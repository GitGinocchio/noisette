use egui::KeyboardShortcut;




#[derive(serde::Deserialize, serde::Serialize)]
pub struct Sound {
    pub name: Option<String>,
    pub path: Option<String>,
    pub shortcut: Option<KeyboardShortcut>,
    pub editing: bool,
    pub data: Option<Vec<u8>>,
}

impl Default for Sound {
    fn default() -> Self {
        Self {
            name: None,
            path: None,
            shortcut: None,
            editing: true,
            data: None
        }
    }
}