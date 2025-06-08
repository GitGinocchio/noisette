use egui::{KeyboardShortcut, Key, Modifiers, Response, Ui, Widget};

pub struct ShortcutRecorder<'a> {
    pub shortcut: &'a mut Option<KeyboardShortcut>,
    pub listening_shortcut: &'a mut Option<usize>, // riferimento al listener centrale
    pub id: usize,  // id univoco di questo widget
}

impl<'a> ShortcutRecorder<'a> {
    pub fn new(
        shortcut: &'a mut Option<KeyboardShortcut>,
        listening_shortcut: &'a mut Option<usize>,
        id: usize,
    ) -> Self {
        Self { shortcut, listening_shortcut, id }
    }
}

impl Widget for ShortcutRecorder<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let is_listening = self.listening_shortcut.map_or(false, |active_id| active_id == self.id);

        let label = if is_listening {
            "Type a Shortcut...".to_string()
        } else if let Some(shortcut) = &self.shortcut {
            let mods = format_modifiers(&shortcut.modifiers);
            if mods.is_empty() {
                format!("{:?}", shortcut.logical_key)
            } else {
                format!("{}+{:?}", mods, shortcut.logical_key)
            }
        } else {
            "No Shortcut".to_string()
        };

        let response = ui.button(label);

        if response.clicked() {
            *self.listening_shortcut = Some(self.id);
        }

        if is_listening {
            ui.input(|i| {
                for event in &i.events {
                    if let egui::Event::Key {
                        key,
                        pressed: true,
                        repeat: false,
                        modifiers,
                        physical_key: _,
                    } = event
                    {
                        let mut mods = *modifiers;
                        mods.mac_cmd = false;
                        mods.command = false;

                        *self.shortcut = Some(KeyboardShortcut {
                            modifiers: mods,
                            logical_key: *key,
                        });

                        *self.listening_shortcut = None;
                        break;
                    }
                }
            });
        }

        response
    }
}

fn format_modifiers(mods: &Modifiers) -> String {
    let mut parts = Vec::new();
    if mods.ctrl {
        parts.push("Ctrl");
    }
    if mods.alt {
        parts.push("Alt");
    }
    if mods.shift {
        parts.push("Shift");
    }
    if mods.mac_cmd || mods.command {
        parts.push("Cmd");
    }
    parts.join("+")
}
