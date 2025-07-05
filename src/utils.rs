use crate::shortcut::keycodes::SerializableKeycode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowLocation {
    pub row: usize,
}

pub fn shortcut_as_string(keys: &[SerializableKeycode]) -> String {
    keys.iter()
        .map(|k| format!("{}", k.0).trim_matches('"').to_string())
        .collect::<Vec<_>>()
        .join(" + ")
}


#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Qui dici a Rust: "questa funzione JS esiste e voglio poterla usare"
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    pub fn trigger_file_picker(row : usize);
}
