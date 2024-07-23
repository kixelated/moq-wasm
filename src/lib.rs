use wasm_bindgen::prelude::*;

mod error;
mod player;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // print pretty errors in wasm https://github.com/rustwasm/console_error_panic_hook
    // This is not needed for tracing_wasm to work, but it is a common tool for getting proper error line numbers for panics.
    console_error_panic_hook::set_once();

    tracing_wasm::set_as_global_default();

    player::register();

    Ok(())
}
