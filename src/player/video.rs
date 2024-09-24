use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use wasm_bindgen::{prelude::Closure, JsCast};

use super::Error;

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
            .coded_dimensions(video.dimensions.width as _, video.dimensions.height as _)
            .latency_optimized();

        if let Some(description) = video.codec.description() {
            config = config.description(description);
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
        // The callback function that will be called on each requestAnimationFrame.
        // We need two references to it so it can schedule itself.
        let render = Rc::new(RefCell::new(None));
        let render2 = render.clone();

        // A queue of frames waiting to be rendered.
        // We need two refrences, one for the callback and one for the main loop.
        let queue = Rc::new(RefCell::new(
            VecDeque::<web_codecs::video::DecodedFrame>::new(),
        ));
        let queue2 = queue.clone();

        // Actually populate the render closure.
        *render.borrow_mut() = Some(Closure::new(move || {
            // do stuff here
            let mut queue = queue2.borrow_mut();

            let frame = match queue.pop_front() {
                Some(frame) => frame,
                None => return,
            };

            canvas.set_width(frame.display_width());
            canvas.set_height(frame.display_height());

            // TODO error handling lul
            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();

            ctx.draw_image_with_video_frame(&frame, 0.0, 0.0).unwrap();

            if queue.is_empty() {
                return;
            }

            // Schedule ourself for another requestAnimationFrame callback.
            request_animation_frame(render2.borrow().as_ref().unwrap());
        }));

        while let Some(frame) = decoded.next().await? {
            let mut queue = queue.borrow_mut();

            // Start the render loop if the queue is empty
            if queue.is_empty() {
                request_animation_frame(render.borrow().as_ref().unwrap());
            }

            queue.push_back(frame);
        }

        Ok(())
    }
}

// https://rustwasm.github.io/wasm-bindgen/examples/request-animation-frame.html
fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .expect("no global `window` exists")
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
