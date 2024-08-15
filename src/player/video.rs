use super::Error;

pub struct Video {
    broadcast: moq_warp::fmp4::BroadcastConsumer,
    canvas: web_sys::HtmlCanvasElement,
}

impl Video {
    pub fn new(
        broadcast: moq_warp::fmp4::BroadcastConsumer,
        canvas: web_sys::HtmlCanvasElement,
    ) -> Self {
        Self { broadcast, canvas }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        let video = match self.broadcast.catalog.video.first() {
            Some(track) => track,
            None => return Ok(()),
        };

        let decoder = web_codecs::video::Decoder::new()?;
        let config = web_codecs::video::DecoderConfig::new(video.codec.to_string())
            .coded_dimensions(video.dimensions.width as _, video.dimensions.height as _)
            .latency_optimized();

        decoder.configure(&config)?;

        tracing::info!("fetching video track: {:?}", video);
        let mut track = self.broadcast.subscribe(video.track.clone()).await?;

        while let Some(frame) = track.read().await? {
            tracing::debug!(?frame, "video frame");
        }
        Ok(())
    }
}
