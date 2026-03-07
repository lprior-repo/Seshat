
pub enum KeyAction {
    Undo,
    Redo,
    Delete,
    Copy,
    Paste,
    Duplicate,
    Group,
    SelectAll,
    ZoomIn,
    ZoomOut,
    None,
}

#[must_use]
pub fn map_key_to_action(
    key: &str,
    ctrl_or_meta: bool,
    shift: bool,
    is_editing_text: bool,
) -> KeyAction {
    // If the user is currently typing in an input field, do not trigger diagram shortcuts
    if is_editing_text {
        // Exception: Escape can always cancel editing
        if key == "Escape" {
            return KeyAction::None; // Handled separately by UI
        }
        return KeyAction::None;
    }

    match (key, ctrl_or_meta, shift) {
        ("z" | "Z", true, false) => KeyAction::Undo,
        ("z" | "Z", true, true) | ("y" | "Y", true, false) => KeyAction::Redo,
        ("c" | "C", true, false) => KeyAction::Copy,
        ("v" | "V", true, false) => KeyAction::Paste,
        ("d" | "D", true, false) => KeyAction::Duplicate,
        ("g" | "G", true, false) => KeyAction::Group,
        ("a" | "A", true, false) => KeyAction::SelectAll,
        ("+" | "=", true, _) => KeyAction::ZoomIn,
        ("-" | "_", true, _) => KeyAction::ZoomOut,
        ("Backspace" | "Delete", false, false) => KeyAction::Delete,
        _ => KeyAction::None,
    }
}

#[cfg(test)]
#[path = "keyboard_tests.rs"]
mod tests;
