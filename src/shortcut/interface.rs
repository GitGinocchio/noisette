use crate::shortcut::keycodes::SerializableKeycode;

pub trait ShortcutListener {
    fn is_pressed(&mut self, shortcut: &Vec<SerializableKeycode>) -> bool;
}