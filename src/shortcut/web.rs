use crate::shortcut::interface::ShortcutListener;

use wasm_bindgen::prelude::*;
use web_sys::window;

pub struct WebShortcutListener {
}

impl ShortcutListener for WebShortcutListener {
    fn is_pressed(&self, shortcut: &Vec<super::keycodes::SerializableKeycode>) -> bool {
        false
    }
}

impl WebShortcutListener {
    pub fn new() -> Self {
        let window = window().unwrap();
        let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            web_sys::console::log_1(&format!("Key pressed: code = {}, key = {}", event.code(), event.key()).into());
        }) as Box<dyn FnMut(_)>);


        window
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();

        closure.forget(); // Prevent the closure from being dropped

        Self {}
    }

    pub fn update(&mut self) {
    }
}