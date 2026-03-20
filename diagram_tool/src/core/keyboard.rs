#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierState {
    CtrlOrMeta,
    Shift,
    CtrlOrMetaAndShift,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    EditingText,
    Diagram,
}

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
    modifiers: ModifierState,
    editor_mode: EditorMode,
) -> KeyAction {
    // If the user is currently typing in an input field, do not trigger diagram shortcuts
    if editor_mode == EditorMode::EditingText {
        // Exception: Escape can always cancel editing
        if key == "Escape" {
            return KeyAction::None; // Handled separately by UI
        }
        return KeyAction::None;
    }

    match (key, modifiers) {
        ("z" | "Z", ModifierState::CtrlOrMeta) => KeyAction::Undo,
        ("z" | "Z", ModifierState::CtrlOrMetaAndShift) | ("y" | "Y", ModifierState::CtrlOrMeta) => {
            KeyAction::Redo
        }
        ("c" | "C", ModifierState::CtrlOrMeta) => KeyAction::Copy,
        ("v" | "V", ModifierState::CtrlOrMeta) => KeyAction::Paste,
        ("d" | "D", ModifierState::CtrlOrMeta) => KeyAction::Duplicate,
        ("g" | "G", ModifierState::CtrlOrMeta) => KeyAction::Group,
        ("a" | "A", ModifierState::CtrlOrMeta) => KeyAction::SelectAll,
        ("+" | "=", ModifierState::CtrlOrMeta) => KeyAction::ZoomIn,
        ("-" | "_", ModifierState::CtrlOrMeta) => KeyAction::ZoomOut,
        ("Backspace" | "Delete", ModifierState::None) => KeyAction::Delete,
        _ => KeyAction::None,
    }
}

#[cfg(test)]
#[path = "keyboard_tests.rs"]
mod tests;
