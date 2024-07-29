use custom_elements::CustomElement;

mod audio;
mod backend;
mod catalog;
mod config;
mod frontend;
mod session;
mod video;

use audio::*;
use backend::*;
use catalog::*;
use config::*;
use frontend::*;
use session::*;
use video::*;

pub fn register() {
    Frontend::define("moq-player");
}
