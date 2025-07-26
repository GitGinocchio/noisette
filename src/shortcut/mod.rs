pub mod interface;

pub mod keycodes;

#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;
#[cfg(not(target_arch = "wasm32"))]
pub use desktop::DesktopShortcutListener as PlatformShortcutListener;

#[cfg(target_arch = "wasm32")]
pub mod web;
#[cfg(target_arch = "wasm32")]
pub use web::WebShortcutListener as PlatformShortcutListener;
