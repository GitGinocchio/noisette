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
            *self.last_pressed_keys = Some(Vec::new());
        }

        if is_listening {
            ui.input(|input| {
                // Ottieni la referenza mutabile a last_pressed_keys, se esiste
                if let Some(keys) = self.last_pressed_keys.as_mut() {
                    // Aggiungi modificatori, se premuti e non già presenti
                    if input.modifiers.ctrl {
                        let modifier = SerializableKeycode::LControl;
                        if !keys.contains(&modifier) {
                            keys.push(modifier);
                        }
                    }
                    else {
                        if !keys.is_empty() {
                            *self.shortcut = Some(keys.clone());
                            *self.listening_shortcut = None;
                        }
                    }

                    if input.modifiers.shift {
                        let modifier = SerializableKeycode::LShift;
                        if !keys.contains(&modifier) {
                            keys.push(modifier);
                        }
                    }
                    else {
                        if !keys.is_empty() {
                            *self.shortcut = Some(keys.clone());
                            *self.listening_shortcut = None;
                        }
                    }


                    if input.modifiers.alt {
                        let modifier = SerializableKeycode::LAlt;
                        if !keys.contains(&modifier) {
                            keys.push(modifier);
                        }
                    }
                    else {
                        if !keys.is_empty() {
                            *self.shortcut = Some(keys.clone());
                            *self.listening_shortcut = None;
                        }
                    }
                    
                    if input.modifiers.mac_cmd {
                        let modifier = SerializableKeycode::LMeta;
                        if !keys.contains(&modifier) {
                            keys.push(modifier);
                        }
                    }
                    else {
                        if !keys.is_empty() {
                            *self.shortcut = Some(keys.clone());
                            *self.listening_shortcut = None;
                        }
                    }

                    // Gestisci gli eventi Key
                    for event in &input.events {
                        if let egui::Event::Key { key, pressed, .. } = event {
                            let key = SerializableKeycode::from(*key);
                            if *pressed {
                                if !keys.contains(&key) {
                                    keys.push(key);
                                }
                            } else {
                                if !keys.is_empty() {
                                    *self.shortcut = Some(keys.clone());
                                    *self.listening_shortcut = None;
                                    *self.last_pressed_keys = None;
                                    break;
                                }
                            }
                        }
                    }
                }
            });
        }

        response
    }
}