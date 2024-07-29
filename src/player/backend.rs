use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
use moq_transfork::runtime::Watch;
use url::Url;

use super::{
    Audio, AudioError, Catalog, CatalogError, Config, Session, SessionError, Video, VideoError,
};

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("session error: {0}")]
    Session(#[from] SessionError),

    #[error("catalog error: {0}")]
    Catalog(#[from] CatalogError),

    #[error("video error: {0}")]
    Video(#[from] VideoError),

    #[error("audio error: {0}")]
    Audio(#[from] AudioError),
}

#[derive(Default)]
pub struct Backend {
    session: Option<Session>,
    catalog: Option<Catalog>,

    video: Option<Video>,
    audio: Option<Audio>,
}

impl Backend {
    async fn run(&mut self, config: Config) -> Result<(), BackendError> {
        // This function can be aborted at any `await` point which is why we use `self.` whenever possible.
        let url = match config.attrs.src.as_ref() {
            Some(url) => url,
            None => return Ok(()),
        };

        let broadcast = match config.attrs.broadcast.as_ref() {
            Some(broadcast) => moq_transfork::Broadcast::new(broadcast),
            None => return Ok(()),
        };

        // Establish a new session if the URL has changed
        self.session = Some(match self.session.take() {
            // Use the existing session
            Some(session) if session.url.as_str() == url => session,

            // Make a new session
            _ => Session::connect(url).await?,
        });

        // Fetch the catalog if the broadcast has changed
        self.catalog = Some(match self.catalog.take() {
            // Use the existing catalog
            Some(catalog) if catalog.broadcast.name == broadcast.name => catalog,
            _ => {
                // Don't try to reuse existing audio and video tracks
                self.audio.take();
                self.video.take();

                // Fetch the catalog
                Catalog::fetch(&mut self.session.as_mut().unwrap().subscriber, broadcast).await?
            }
        });

        if self.video.is_none() {
            if let Some(track) = self.catalog.as_ref().unwrap().video().next() {
                // TODO perform in parallel with audio
                self.video = Some(
                    Video::fetch(
                        &mut self.session.as_mut().unwrap().subscriber,
                        track.clone(),
                    )
                    .await?,
                );
            }
        }

        if self.audio.is_none() {
            if let Some(track) = self.catalog.as_ref().unwrap().audio().next() {
                // TODO perform in parallel with video
                self.audio = Some(
                    Audio::fetch(
                        &mut self.session.as_mut().unwrap().subscriber,
                        track.clone(),
                    )
                    .await?,
                );
            }
        }

        let mut tasks = FuturesUnordered::new();

        if let Some(video) = self.video.as_mut() {
            tasks.push(async move { Ok(video.run(config.canvas).await?) }.boxed_local());
        }

        if let Some(audio) = self.audio.as_mut() {
            tasks.push(async move { Ok(audio.run().await?) }.boxed_local());
        }

        while let Some(res) = tasks.next().await {
            if let Err(err) = res {
                return Err(err);
            }
        }

        Ok(())
    }

    pub async fn watch(&mut self, watch: Watch<Config>) -> Result<(), BackendError> {
        loop {
            let guard = watch.lock();
            let config = guard.clone();

            let mut notify = match guard.changed() {
                Some(notify) => notify,
                None => return Ok(()),
            };

            tracing::info!(?config, "running backend");

            tokio::select! {
                _ = &mut notify => continue,
                res = self.run(config) => {
                    if let Err(err) = res {
                        tracing::error!(?err, "backend error");
                    }
                },
            };

            notify.await;
        }
    }
}
