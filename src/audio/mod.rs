pub mod interface;

#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;
#[cfg(not(target_arch = "wasm32"))]
pub use desktop::DesktopAudioHandler as PlatformAudioHandler;
#[cfg(not(target_arch = "wasm32"))]
pub use desktop::{get_default_output_device, get_output_devices, get_device_from_name};

#[cfg(target_arch = "wasm32")]
pub mod web;
#[cfg(target_arch = "wasm32")]
pub use web::WebAudio as PlatformAudioHandler;
#[cfg(target_arch = "wasm32")]
pub use web::{get_default_output_device, get_output_devices, get_device_from_name};
