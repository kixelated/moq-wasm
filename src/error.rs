use web_sys::js_sys;

#[derive(Clone, Debug, thiserror::Error)]
#[error("web error: {0:?}")]
pub struct WebError(js_sys::Error);

impl From<js_sys::Error> for WebError {
    fn from(e: js_sys::Error) -> Self {
        Self(e)
    }
}

impl From<wasm_bindgen::JsValue> for WebError {
    fn from(e: wasm_bindgen::JsValue) -> Self {
        Self(e.into())
    }
}

pub trait WebErrorExt<T> {
    fn throw(self) -> Result<T, WebError>;
}

impl<T, E: Into<WebError>> WebErrorExt<T> for Result<T, E> {
    fn throw(self) -> Result<T, WebError> {
        self.map_err(Into::into)
    }
}

impl<T> WebErrorExt<T> for Option<T> {
    fn throw(self) -> Result<T, WebError> {
        self.ok_or(WebError(js_sys::Error::new("unwrapped None")))
    }
}
