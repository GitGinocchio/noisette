#![warn(clippy::all, rust_2018_idioms)]

mod sound;
pub use sound::Sound;

mod app;
pub use app::Noisette;

mod widgets;
pub use widgets::shortcut;

mod utils;
pub use utils::*;