use wasm_bindgen::prelude::*;
use web_sys::{window, AudioContext, HtmlMediaElement, AudioBufferSourceNode};

use crate::Sound;

use super::interface::AudioBackend;

pub struct WebAudio {
    context: AudioContext,
    audio_element: Option<HtmlMediaElement>
}

impl WebAudio {
    pub fn new(device: Option<u8>) -> Self {
        Self {
            context: AudioContext::new().unwrap(),
            audio_element: None
        }
    }

    pub fn set_device(&mut self, _device: Option<String>) {
        // Stub: non supportato su web
        web_sys::console::log_1(&"set_device not supported on Web".into());
    }
}

impl AudioBackend for WebAudio {
    fn play(&mut self, sound: &Sound) {
        web_sys::console::log_1(&"Playing audio in Web!".into());
        if let Some(path) = &sound.path {
            let document = window().unwrap().document().unwrap();
            let audio = document
                .create_element("audio").unwrap()
                .dyn_into::<HtmlMediaElement>().unwrap();

            audio.set_src(path);
            audio.set_autoplay(true);

            let _ = audio.play(); // Starts playback
            self.audio_element = Some(audio);
        }
    }

    fn stop(&mut self) {
        web_sys::console::log_1(&"Stopping audio".into());
        if let Some(audio) = &self.audio_element {
            let _ = audio.pause();
            audio.set_current_time(0.0);
        }

    }

    fn is_playing(&self) -> bool {
        self.audio_element
            .as_ref()
            .map_or(false, |a| !a.paused())
    }
}

pub fn get_output_devices() -> Vec<String> {
    vec![]
}

pub fn get_default_output_device() -> Option<String> {
    None
}

pub fn get_device_from_name(name: Option<String>) -> Option<String> {
    None
}