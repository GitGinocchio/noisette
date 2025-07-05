use device_query::{DeviceQuery, DeviceState};
use egui::{Response, Ui, Widget};

use crate::shortcut::keycodes::SerializableKeycode;
use crate::utils::shortcut_as_string;

pub struct ShortcutRecorder<'a> {
    pub shortcut: &'a mut Option<Vec<SerializableKeycode>>,
    pub listening_shortcut: &'a mut Option<usize>,
    pub last_pressed_keys: &'a mut Option<Vec<SerializableKeycode>>,
    pub id: usize,
}

impl<'a> ShortcutRecorder<'a> {
    pub fn new(
        shortcut: &'a mut Option<Vec<SerializableKeycode>>,
        listening_shortcut: &'a mut Option<usize>,
        last_pressed_keys: &'a mut Option<Vec<SerializableKeycode>>,
        id: usize,
    ) -> Self {
        Self {
            shortcut,
            listening_shortcut,
            last_pressed_keys,
            id,
        }
    }
}

impl Widget for ShortcutRecorder<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let is_listening = self
            .listening_shortcut
            .map_or(false, |active_id| active_id == self.id);
        
        let label = if is_listening {
            if let Some(keys) = &self.last_pressed_keys {
                if keys.is_empty() {
                    "Type a Shortcut...".to_string()
                } else {
                    shortcut_as_string(keys)
                }
            } else {
                "Type a Shortcut...".to_string()
            }
        } else if let Some(keys) = &self.shortcut {
            shortcut_as_string(keys)
        } else {
            "No Shortcut".to_string()
        };


        let response = ui.button(label);

        if response.clicked() {
            *self.listening_shortcut = Some(self.id);
            *self.last_pressed_keys = None;
        }

        if is_listening {
            let device_state = DeviceState::new();
            let current_keys: Vec<SerializableKeycode> = device_state
                .get_keys()
                .into_iter()
                .map(SerializableKeycode)
                .collect();

            if let Some(previous_keys) = &self.last_pressed_keys {
                let released = previous_keys
                    .iter()
                    .any(|prev_key| !current_keys.contains(prev_key));

                if released && !previous_keys.is_empty() {
                    *self.shortcut = Some(previous_keys.clone());
                    *self.listening_shortcut = None;
                    *self.last_pressed_keys = None;
                } else {
                    *self.last_pressed_keys = Some(current_keys);
                }
            } else {
                *self.last_pressed_keys = Some(current_keys);
            }
        }

        response
    }
}
