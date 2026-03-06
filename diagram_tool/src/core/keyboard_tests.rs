#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::keyboard::{map_key_to_action, KeyAction};

    #[test]
    fn test_shortcuts_do_not_fire_when_input_has_focus() {
        let action = map_key_to_action("z", true, false, true);
        assert!(matches!(action, KeyAction::None));

        let action2 = map_key_to_action("Delete", false, false, true);
        assert!(matches!(action2, KeyAction::None));
    }

    #[test]
    fn test_undo_and_redo_bindings() {
        assert!(matches!(
            map_key_to_action("z", true, false, false),
            KeyAction::Undo
        ));
        assert!(matches!(
            map_key_to_action("Z", true, false, false),
            KeyAction::Undo
        ));

        // Ctrl+Shift+Z is Redo
        assert!(matches!(
            map_key_to_action("z", true, true, false),
            KeyAction::Redo
        ));
        // Ctrl+Y is Redo
        assert!(matches!(
            map_key_to_action("y", true, false, false),
            KeyAction::Redo
        ));
    }

    #[test]
    fn test_clipboard_bindings() {
        assert!(matches!(
            map_key_to_action("c", true, false, false),
            KeyAction::Copy
        ));
        assert!(matches!(
            map_key_to_action("v", true, false, false),
            KeyAction::Paste
        ));
        assert!(matches!(
            map_key_to_action("d", true, false, false),
            KeyAction::Duplicate
        ));
    }

    #[test]
    fn test_delete_binding() {
        assert!(matches!(
            map_key_to_action("Delete", false, false, false),
            KeyAction::Delete
        ));
        assert!(matches!(
            map_key_to_action("Backspace", false, false, false),
            KeyAction::Delete
        ));
    }
}
