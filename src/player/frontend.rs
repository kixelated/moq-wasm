use custom_elements::*;
use tokio::sync::watch;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::*;
use web_sys::{window, HtmlCanvasElement, HtmlElement, Node};

use crate::error::{WebError, WebErrorExt};

use super::{Attributes, Backend, Config};

#[derive(Default)]
pub struct Frontend {
    config: watch::Sender<Config>,
}

impl Frontend {
    pub fn view(&mut self) -> Result<Node, WebError> {
        let document = window().throw()?.document().throw()?;

        let canvas: HtmlCanvasElement = document.create_element("canvas").throw()?.unchecked_into();

        let div = document.create_element("div").throw()?;
        div.append_child(&canvas).throw()?;

        self.config
            .send_modify(|state| state.canvas = Some(canvas.clone()));

        Ok(div.into())
    }
}

impl CustomElement for Frontend {
    fn constructor(&mut self, _this: &HtmlElement) {
        let config = self.config.subscribe();
        let mut backend = Backend::default();
        spawn_local(async move { backend.watch(config).await.unwrap_throw() });
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
        self.config
            .send_if_modified(|state| state.attrs.update(name, new_value));
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
