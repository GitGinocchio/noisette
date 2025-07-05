#![warn(clippy::all, rust_2018_idioms)]

mod sound;
pub use sound::Sound;

mod app;
pub use app::Noisette;

mod widgets;

mod utils;
pub use utils::*;

pub mod audio;

pub mod shortcut;