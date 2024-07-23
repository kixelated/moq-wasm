use custom_elements::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::*;
use web_sys::{window, HtmlCanvasElement, HtmlElement, Node};

use moq_transfork::util::State;

use crate::error::{WebError, WebErrorExt};

use super::{Attributes, Backend, Config};

#[derive(Default)]
pub struct Frontend {
    config: State<Config>,
}

impl Frontend {
    pub fn view(&mut self) -> Result<Node, WebError> {
        let document = window().throw()?.document().throw()?;

        let canvas: HtmlCanvasElement = document.create_element("canvas").throw()?.unchecked_into();

        let div = document.create_element("div").throw()?;
        div.append_child(&canvas).throw()?;

        self.config.lock_mut().throw()?.canvas = Some(canvas);

        Ok(div.into())
    }
}

impl CustomElement for Frontend {
    fn constructor(&mut self, _this: &HtmlElement) {
        tracing::info!("constructor");

        let config = self.config.split();
        let backend = Backend::new(config);

        spawn_local(async move { backend.run().await.unwrap_throw() });
    }

    fn inject_children(&mut self, this: &HtmlElement) {
        let node = self.view().unwrap_throw();
        this.append_child(&node).unwrap_throw();
    }

    fn observed_attributes() -> &'static [&'static str] {
        Attributes::names()
    }

    fn attribute_changed_callback(
        &mut self,
        _this: &HtmlElement,
        name: String,
        _old_value: Option<String>,
        new_value: Option<String>,
    ) {
        let mut state = self.config.lock_mut().unwrap_throw();
        state.attrs.update(name, new_value);
    }

    fn connected_callback(&mut self, _this: &HtmlElement) {
        tracing::info!("connected");
    }

    fn disconnected_callback(&mut self, _this: &HtmlElement) {
        tracing::info!("disconnected");
    }

    fn adopted_callback(&mut self, _this: &HtmlElement) {
        tracing::info!("adopted");
    }
}
