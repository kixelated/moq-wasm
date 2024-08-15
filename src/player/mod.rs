use custom_elements::CustomElement;

mod audio;
mod backend;
mod config;
mod error;
mod frontend;
mod video;

use audio::*;
use backend::*;
use config::*;
use error::*;
use frontend::*;
use video::*;

pub fn register() {
    Frontend::define("moq-player");
}
