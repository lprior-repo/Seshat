#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
use crate::core::keyboard::*;

#[test]
fn shortcuts_do_not_fire_when_input_has_focus() {
    let action = map_key_to_action("z", ModifierState::CtrlOrMeta, EditorMode::EditingText);
    assert!(matches!(action, KeyAction::None));

    let action2 = map_key_to_action("Delete", ModifierState::None, EditorMode::EditingText);
    assert!(matches!(action2, KeyAction::None));
}

#[test]
fn undo_and_redo_bindings_map_correctly() {
    assert!(matches!(
        map_key_to_action("z", ModifierState::CtrlOrMeta, EditorMode::Diagram),
        KeyAction::Undo
    ));
    assert!(matches!(
        map_key_to_action("Z", ModifierState::CtrlOrMeta, EditorMode::Diagram),
        KeyAction::Undo
    ));

    // Ctrl+Shift+Z is Redo
    assert!(matches!(
        map_key_to_action("z", ModifierState::CtrlOrMetaAndShift, EditorMode::Diagram),
        KeyAction::Redo
    ));
    // Ctrl+Y is Redo
    assert!(matches!(
        map_key_to_action("y", ModifierState::CtrlOrMeta, EditorMode::Diagram),
        KeyAction::Redo
    ));
}

#[test]
fn clipboard_bindings_map_correctly() {
    assert!(matches!(
        map_key_to_action("c", ModifierState::CtrlOrMeta, EditorMode::Diagram),
        KeyAction::Copy
    ));
    assert!(matches!(
        map_key_to_action("v", ModifierState::CtrlOrMeta, EditorMode::Diagram),
        KeyAction::Paste
    ));
    assert!(matches!(
        map_key_to_action("d", ModifierState::CtrlOrMeta, EditorMode::Diagram),
        KeyAction::Duplicate
    ));
}

#[test]
fn delete_and_backspace_map_to_delete_action() {
    assert!(matches!(
        map_key_to_action("Delete", ModifierState::None, EditorMode::Diagram),
        KeyAction::Delete
    ));
    assert!(matches!(
        map_key_to_action("Backspace", ModifierState::None, EditorMode::Diagram),
        KeyAction::Delete
    ));
}
