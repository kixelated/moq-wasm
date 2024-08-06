use std::pin::Pin;

use futures::{stream::FuturesUnordered, FutureExt, StreamExt};

use super::{CatalogError, Session};

#[derive(Debug, thiserror::Error)]
pub enum VideoError {
    #[error("catalog error: {0}")]
    Catalog(#[from] CatalogError),

    #[error("transfork error: {0}")]
    Transfork(#[from] moq_transfork::Error),

    #[error("missing namespace")]
    MissingNamespace,
}

pub struct Video {
    info: moq_warp::catalog::Track,
    reader: moq_transfork::TrackReader,
    tasks: FuturesUnordered<Pin<Box<dyn std::future::Future<Output = Result<(), VideoError>>>>>,
}

impl Video {
    pub async fn fetch(
        session: &mut Session,
        info: moq_warp::catalog::Track,
    ) -> Result<Self, VideoError> {
        tracing::info!("fetching video track: {:?}", info);
        let broadcast = info
            .namespace
            .as_ref()
            .ok_or(VideoError::MissingNamespace)?;
        let track = moq_transfork::Track::build(&info.name, 2).into();

        let reader = session.subscribe(broadcast, track).await?;
        let tasks = FuturesUnordered::new();

        Ok(Self {
            info,
            reader,
            tasks,
        })
    }

    // NOTE: Can be called multiple times
    pub async fn run(
        &mut self,
        canvas: Option<web_sys::HtmlCanvasElement>,
    ) -> Result<(), VideoError> {
        loop {
            tokio::select! {
                Ok(Some(group)) = self.reader.next_group() => {
                    self.tasks.push(Self::run_group(group).boxed_local());
                },
                Some(res) = self.tasks.next() => {
                    if let Err(err) = res {
                        tracing::warn!(?err, "failed to run group")
                    }
                },
                else => return Ok(self.reader.closed().await?),
            }
        }
    }

    async fn run_group(mut group: moq_transfork::GroupReader) -> Result<(), VideoError> {
        while let Some(frame) = group.read_frame().await? {
            tracing::info!("video frame: {:?}", frame.len());
        }

        Ok(())
    }
}
