use egui::KeyboardShortcut;




#[derive(serde::Deserialize, serde::Serialize)]
pub struct Sound {
    pub name: Option<String>,
    pub path: Option<String>,
    pub shortcut: Option<KeyboardShortcut>,
    pub editing: bool
}

impl Default for Sound {
    fn default() -> Self {
        Self {
            name: None,
            path: None,
            shortcut: None,
            editing: true
        }
    }
}