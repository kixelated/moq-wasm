use super::Error;

mod renderer;
use renderer::*;

pub struct Video {
    broadcast: moq_warp::media::BroadcastConsumer,
    canvas: web_sys::HtmlCanvasElement,
}

impl Video {
    pub fn new(
        broadcast: moq_warp::media::BroadcastConsumer,
        canvas: web_sys::HtmlCanvasElement,
    ) -> Self {
        Self { broadcast, canvas }
    }

    pub async fn run(self) -> Result<(), Error> {
        let video = match self.broadcast.catalog().video.first() {
            Some(track) => track,
            None => return Ok(()),
        };

        let (decoder, decoded) = web_codecs::video::decoder();

        tracing::info!("configuring video decoder: {:?}", video.codec);

        let mut config = web_codecs::video::DecoderConfig::new(video.codec.to_string())
            .coded_dimensions(video.resolution.width as _, video.resolution.height as _)
            .latency_optimized();

        if !video.description.is_empty() {
            config = config.description(video.description.clone().into());
        }

        decoder.configure(&config)?;

        tracing::info!("fetching video track: {:?}", video);
        let track = self.broadcast.subscribe(video.track.clone()).await?;

        tokio::select! {
            Err(err) = Self::run_decoder(track, decoder) => Err(err),
            Err(err) = Self::run_renderer(self.canvas, decoded) => Err(err),
            else => Ok(()),
        }
    }

    async fn run_decoder(
        mut track: moq_warp::media::TrackConsumer,
        decoder: web_codecs::video::Decoder,
    ) -> Result<(), Error> {
        while let Some(frame) = track.read().await? {
            let frame = web_codecs::video::EncodedFrame {
                payload: frame.payload,
                timestamp: frame.timestamp.as_micros() as _,
                keyframe: frame.keyframe,
            };
            decoder.decode(frame)?;
        }

        Ok(())
    }

    async fn run_renderer(
        canvas: web_sys::HtmlCanvasElement,
        mut decoded: web_codecs::video::Decoded,
    ) -> Result<(), Error> {
        let mut renderer = Renderer::new(canvas);

        while let Some(frame) = decoded.next().await? {
            renderer.push(frame);
        }

        Ok(())
    }
}
