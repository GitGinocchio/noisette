use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use device_query::{DeviceQuery, DeviceState};

use crate::shortcut::keycodes::SerializableKeycode;
use crate::Sound;

pub struct DesktopShortcutListener {
    pressed_keys: Arc<Mutex<Vec<SerializableKeycode>>>
}

impl DesktopShortcutListener {
    pub fn new(
        sounds: Arc<Mutex<Vec<Sound>>>,
        sender: mpsc::Sender<usize>,
    ) -> Self {
        let keys = Arc::new(Mutex::new(Vec::new()));
        let keys_clone = Arc::clone(&keys);
        let sounds = Arc::clone(&sounds);

        thread::spawn(move || {
            let device_state = DeviceState::new();
            let mut last_played: Option<Vec<SerializableKeycode>> = None;

            loop {
                let current_keys: Vec<SerializableKeycode> = device_state
                    .get_keys()
                    .into_iter()
                    .map(SerializableKeycode)
                    .collect();

                if let Ok(mut keys_lock) = keys_clone.lock() {
                    *keys_lock = current_keys.clone();
                }

                if let Ok(sounds) = sounds.lock() {
                    for (idx, sound) in sounds.iter().enumerate() {
                        if let Some(shortcut) = &sound.shortcut {
                            let all_pressed = shortcut.iter().all(|k| current_keys.contains(k));
                            let already_played = Some(shortcut.clone()) == last_played;

                            if all_pressed && !already_played {
                                let _ = sender.send(idx); // invia messaggio di play
                                last_played = Some(shortcut.clone());
                            }

                            if current_keys.is_empty() {
                                last_played = None;
                            }
                        }
                    }
                }

                thread::sleep(Duration::from_millis(50));
            }
        });

        Self {
            pressed_keys: keys
        }
    }
}
