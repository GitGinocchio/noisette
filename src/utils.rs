
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowLocation {
    pub row: usize,
}

pub fn shortcut_as_string(sc: &egui::KeyboardShortcut) -> String {
    let mut parts = Vec::new();
    let m = &sc.modifiers;
    if m.ctrl { parts.push("Ctrl"); }
    if m.alt { parts.push("Alt"); }
    if m.shift { parts.push("Shift"); }
    if m.mac_cmd || m.command { parts.push("Cmd"); }
    let key_string = &format!("{:?}", sc.logical_key);
    parts.push(&key_string);
    parts.join("+")
}
