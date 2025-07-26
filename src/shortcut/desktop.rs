use std::sync::{Arc, Mutex};

use device_query::{DeviceQuery, DeviceState};

use crate::shortcut::keycodes::SerializableKeycode;
use crate::shortcut::interface::ShortcutListener;

pub struct DesktopShortcutListener {
    pressed_keys: Arc<Mutex<Vec<SerializableKeycode>>>
}

impl DesktopShortcutListener {
    pub fn new() -> Self {
        let keys = Arc::new(Mutex::new(Vec::new()));
        //let keys_clone = Arc::clone(&keys);

        Self {
            pressed_keys: keys
        }
    }

    pub fn update(&mut self) {
        let device_state = DeviceState::new();
        let keys_clone = Arc::clone(&self.pressed_keys);

        let current_keys: Vec<SerializableKeycode> = device_state
            .get_keys()
            .into_iter()
            .map(|k| SerializableKeycode::from(k))
            .collect();

        if let Ok(mut keys_lock) = keys_clone.lock() {
            *keys_lock = current_keys.clone();
        }
    }

    pub fn get_pressed_keys(&self) -> Vec<SerializableKeycode> {
        if let Ok(keys_guard) = self.pressed_keys.lock() {
            keys_guard.clone()
        } else {
            Vec::new()
        }
    }
}

impl ShortcutListener for DesktopShortcutListener {
    fn is_pressed(&self, shortcut: &Vec<SerializableKeycode>) -> bool {
        if let Ok(pressed) = self.pressed_keys.lock() {
            shortcut.iter().all(|k| pressed.contains(k))
        } else {
            false // non riesce a ottenere il lock? -> non premuto
        }
    }
}
