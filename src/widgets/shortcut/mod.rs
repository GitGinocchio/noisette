

#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;
#[cfg(not(target_arch = "wasm32"))]
pub use desktop::ShortcutRecorder as PlatformShortcutRecorder;

#[cfg(target_arch = "wasm32")]
pub mod web;
#[cfg(target_arch = "wasm32")]
pub use web::ShortcutRecorder as PlatformShortcutRecorder;