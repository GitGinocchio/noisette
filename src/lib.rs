#![warn(clippy::all, rust_2018_idioms)]

mod sound;
pub use sound::Sound;

mod app;
pub use app::Noisette;

mod widgets;

mod icons;
pub use icons::*;

mod utils;
pub use utils::*;

pub mod helpers;
pub use helpers::*;

pub mod audio;

pub mod shortcut;