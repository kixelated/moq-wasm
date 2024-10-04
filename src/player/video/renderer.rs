use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use wasm_bindgen::{prelude::Closure, JsCast};
use web_codecs::video::DecodedFrame;

#[derive(Clone)]
pub struct Renderer {
    state: Rc<RefCell<RenderState>>,
}

struct RenderState {
    scheduled: bool,
    canvas: web_sys::HtmlCanvasElement,
    queue: VecDeque<DecodedFrame>,
    render: Option<Closure<dyn FnMut()>>,
}

impl Renderer {
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Self {
        let state = Rc::new(RefCell::new(RenderState {
            scheduled: false,
            canvas,
            queue: Default::default(),
            render: None,
        }));

        let renderer = Renderer {
            state: state.clone(),
        };

        let mut renderer_clone = renderer.clone();
        let f = Closure::wrap(Box::new(move || {
            renderer_clone.render();
        }) as Box<dyn FnMut()>);

        state.borrow_mut().render = Some(f);

        Self { state }
    }

    fn render(&mut self) {
        let mut state = self.state.borrow_mut();
        state.scheduled = false;

        let frame = match state.queue.pop_front() {
            Some(frame) => frame,
            None => return,
        };

        let canvas = &mut state.canvas;
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
    }

    fn schedule(&mut self) {
        let mut state = self.state.borrow_mut();
        if state.scheduled {
            return;
        }

        let render = state.render.as_ref().unwrap();
        request_animation_frame(render);

        state.scheduled = true;
    }

    pub fn push(&mut self, frame: DecodedFrame) {
        self.state.borrow_mut().queue.push_back(frame);
        self.schedule();
    }
}

// https://rustwasm.github.io/wasm-bindgen/examples/request-animation-frame.html
fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .expect("no global `window` exists")
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
