use crate::shortcut::keycodes::SerializableKeycode;

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Clone, Debug)]
pub struct Sound {
    pub name: Option<String>,
    pub path: Option<String>,
    pub shortcut: Option<Vec<SerializableKeycode>>,
    pub editing: bool,
    pub playing: bool,
    pub data: Option<Vec<u8>>,
}

impl Default for Sound {
    fn default() -> Self {
        Self {
            name: None,
            path: None,
            shortcut: None,
            editing: true,
            playing: false,
            data: None
        }
    }
}