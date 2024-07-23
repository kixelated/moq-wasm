use custom_elements::*;
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlElement, Node};

#[derive(Default)]
pub struct MoqComponent {}

impl MoqComponent {
	pub fn view(&self) -> Node {
		let document = window().unwrap_throw().document().unwrap_throw();
		let p = document.create_element("p").unwrap_throw();
		let text = document.create_text_node("Hello, moq-web!");
		p.append_child(&text).unwrap_throw();
		p.into()
	}
}

impl CustomElement for MoqComponent {
	fn inject_children(&mut self, this: &HtmlElement) {
		inject_style(&this, "p { color: green; }");
		let node = self.view();
		this.append_child(&node).unwrap_throw();
	}

	fn observed_attributes() -> &'static [&'static str] {
		&["url"]
	}

	fn attribute_changed_callback(
		&mut self,
		_this: &HtmlElement,
		name: String,
		_old_value: Option<String>,
		new_value: Option<String>,
	) {
		if name == "url" { /* do something... */ }
	}

	fn connected_callback(&mut self, _this: &HtmlElement) {
		log("connected");
	}

	fn disconnected_callback(&mut self, _this: &HtmlElement) {
		log("disconnected");
	}

	fn adopted_callback(&mut self, _this: &HtmlElement) {
		log("adopted");
	}
}

pub fn init() {
	MoqComponent::define("moq-web");
}

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console)]
	fn log(s: &str);
}
