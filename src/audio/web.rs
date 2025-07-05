#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, AudioBufferSourceNode};

use crate::Sound;

use super::interface::AudioBackend;

pub struct WebAudio {
    context: AudioContext,
}

impl WebAudio {
    pub fn new() -> Self {
        Self {
            context: AudioContext::new().unwrap(),
        }
    }
}

impl AudioBackend for WebAudio {
    fn play(&mut self, sound: &mut Sound) {
        web_sys::console::log_1(&"Playing audio in Web!".into());
        // Qui andrebbe caricato e suonato un audio buffer (es. via fetch API).
    }

    fn stop(&mut self) {
        web_sys::console::log_1(&"Stopping audio (not yet implemented)".into());
    }

    fn is_playing(&self) -> bool {
        false // Demo: serve logica interna
    }
}
