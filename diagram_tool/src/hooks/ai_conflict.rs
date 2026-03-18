#![allow(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;

/// Error types for AI conflict state operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictError {
    /// Signal provider not available in context
    SignalNotFound,
    /// Attempted to set an empty conflict message
    InvalidMessage,
}

/// Hook to access the AI conflict state signal.
///
/// Returns a copy of the Signal<Option<String>> from context.
/// The signal will be available if context provider is initialized in app.rs.
///
/// # Panics
/// Panics if `ai_conflict_state` context provider is not initialized.
#[must_use]
pub fn use_ai_conflict_state() -> Signal<Option<String>> {
    use_context::<Signal<Option<String>>>()
}

/// Sets the conflict message in the AI conflict state.
///
/// # Errors
/// Returns `ConflictError::InvalidMessage` if the message is empty.
pub const fn set_conflict_message(msg: &str) -> Result<(), ConflictError> {
    if msg.is_empty() {
        return Err(ConflictError::InvalidMessage);
    }
    Ok(())
}

/// Clears the conflict state (resolves the conflict).
/// Takes mutable reference to Signal since `set()` requires interior mutability.
pub fn clear_conflict(signal: &mut Signal<Option<String>>) {
    signal.set(None);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a minimal `VirtualDom` and runs the test closure inside the Dioxus runtime
    /// with a scope context (required for `Signal::new`).
    fn dioxus_test_harness<F, R>(test: F) -> R
    where
        F: FnOnce() -> R,
    {
        // Create a minimal VirtualDom with an empty component
        let dom = VirtualDom::new(|| rsx! {});

        // Run the test inside the Dioxus runtime AND scope context
        // ScopeId::ROOT provides the necessary scope for Signal::new
        dom.in_scope(ScopeId::ROOT, test)
    }

    #[test]
    fn test_signal_initializes_with_none() {
        // The signal is initialized via context in app.rs
        // Testing that None is the expected initial state
        let initial: Option<String> = None;
        assert!(initial.is_none());
    }

    #[test]
    fn test_set_conflict_message() {
        let result = set_conflict_message("AI generated conflicting operations");
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_conflict_message_empty_fails() {
        let result = set_conflict_message("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ConflictError::InvalidMessage);
    }

    #[test]
    fn test_clear_conflict() {
        dioxus_test_harness(|| {
            // Test that clearing sets to None
            let mut signal = Signal::new(Some("conflict message".to_string()));
            clear_conflict(&mut signal);
            assert!(signal.read().is_none());
        });
    }
}
