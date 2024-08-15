use super::Error;

pub struct Audio {
    broadcast: moq_warp::fmp4::BroadcastConsumer,
}

impl Audio {
    pub fn new(broadcast: moq_warp::fmp4::BroadcastConsumer) -> Self {
        Self { broadcast }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        let audio = match self.broadcast.catalog.audio.first() {
            Some(track) => track,
            None => return Ok(()),
        };

        tracing::info!("fetching audio track: {:?}", audio);
        let mut track = self.broadcast.subscribe(audio.track.clone()).await?;

        while let Some(frame) = track.read().await? {
            tracing::debug!(?frame, "audio frame");
        }

        Ok(())
    }
}
