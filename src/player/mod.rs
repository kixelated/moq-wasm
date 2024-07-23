use custom_elements::CustomElement;

mod backend;
mod config;
mod frontend;

use backend::*;
use config::*;
use frontend::*;

pub fn register() {
    Frontend::define("moq-player");
}
