use crate::shortcut::keycodes::SerializableKeycode;

pub trait ShortcutListener {
    fn is_pressed(&self, shortcut: &Vec<SerializableKeycode>) -> bool;
}