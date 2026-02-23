use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionState {
    #[default]
    Idle,
    Waiting,
    Running,
    Succeeded,
    Failed,
    Skipped,
}

impl fmt::Display for ExecutionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Waiting => write!(f, "waiting"),
            Self::Running => write!(f, "running"),
            Self::Succeeded => write!(f, "succeeded"),
            Self::Failed => write!(f, "failed"),
            Self::Skipped => write!(f, "skipped"),
        }
    }
}

impl ExecutionState {
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Skipped)
    }

    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Running | Self::Waiting)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_six_variants() {
        let states = [
            ExecutionState::Idle,
            ExecutionState::Waiting,
            ExecutionState::Running,
            ExecutionState::Succeeded,
            ExecutionState::Failed,
            ExecutionState::Skipped,
        ];
        assert_eq!(states.len(), 6);
    }

    #[test]
    fn variant_idle_exists() {
        let state = ExecutionState::Idle;
        assert!(!state.is_active());
        assert!(!state.is_terminal());
    }

    #[test]
    fn variant_waiting_exists() {
        let state = ExecutionState::Waiting;
        assert!(state.is_active());
        assert!(!state.is_terminal());
    }

    #[test]
    fn variant_running_exists() {
        let state = ExecutionState::Running;
        assert!(state.is_active());
        assert!(!state.is_terminal());
    }

    #[test]
    fn variant_succeeded_exists() {
        let state = ExecutionState::Succeeded;
        assert!(!state.is_active());
        assert!(state.is_terminal());
    }

    #[test]
    fn variant_failed_exists() {
        let state = ExecutionState::Failed;
        assert!(!state.is_active());
        assert!(state.is_terminal());
    }

    #[test]
    fn variant_skipped_exists() {
        let state = ExecutionState::Skipped;
        assert!(!state.is_active());
        assert!(state.is_terminal());
    }

    #[test]
    fn display_outputs_lowercase() {
        assert_eq!(ExecutionState::Idle.to_string(), "idle");
        assert_eq!(ExecutionState::Waiting.to_string(), "waiting");
        assert_eq!(ExecutionState::Running.to_string(), "running");
        assert_eq!(ExecutionState::Succeeded.to_string(), "succeeded");
        assert_eq!(ExecutionState::Failed.to_string(), "failed");
        assert_eq!(ExecutionState::Skipped.to_string(), "skipped");
    }

    #[test]
    fn is_terminal_returns_true_for_terminal_states() {
        assert!(ExecutionState::Succeeded.is_terminal());
        assert!(ExecutionState::Failed.is_terminal());
        assert!(ExecutionState::Skipped.is_terminal());
    }

    #[test]
    fn is_terminal_returns_false_for_non_terminal_states() {
        assert!(!ExecutionState::Idle.is_terminal());
        assert!(!ExecutionState::Waiting.is_terminal());
        assert!(!ExecutionState::Running.is_terminal());
    }

    #[test]
    fn is_active_returns_true_for_active_states() {
        assert!(ExecutionState::Running.is_active());
        assert!(ExecutionState::Waiting.is_active());
    }

    #[test]
    fn is_active_returns_false_for_inactive_states() {
        assert!(!ExecutionState::Idle.is_active());
        assert!(!ExecutionState::Succeeded.is_active());
        assert!(!ExecutionState::Failed.is_active());
        assert!(!ExecutionState::Skipped.is_active());
    }

    #[test]
    fn default_is_idle() {
        assert_eq!(ExecutionState::default(), ExecutionState::Idle);
    }

    #[test]
    fn serialization_roundtrip() {
        let states = [
            ExecutionState::Idle,
            ExecutionState::Waiting,
            ExecutionState::Running,
            ExecutionState::Succeeded,
            ExecutionState::Failed,
            ExecutionState::Skipped,
        ];

        for state in states {
            let json = serde_json::to_string(&state).expect("serialize");
            let deserialized: ExecutionState = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(state, deserialized);
        }
    }

    #[test]
    fn serialization_uses_lowercase() {
        assert_eq!(
            serde_json::to_string(&ExecutionState::Idle).unwrap(),
            "\"idle\""
        );
        assert_eq!(
            serde_json::to_string(&ExecutionState::Running).unwrap(),
            "\"running\""
        );
        assert_eq!(
            serde_json::to_string(&ExecutionState::Succeeded).unwrap(),
            "\"succeeded\""
        );
    }

    #[test]
    fn deserialization_accepts_lowercase() {
        let state: ExecutionState = serde_json::from_str("\"idle\"").unwrap();
        assert_eq!(state, ExecutionState::Idle);

        let state: ExecutionState = serde_json::from_str("\"running\"").unwrap();
        assert_eq!(state, ExecutionState::Running);

        let state: ExecutionState = serde_json::from_str("\"succeeded\"").unwrap();
        assert_eq!(state, ExecutionState::Succeeded);
    }

    #[test]
    fn transition_idle_to_running_is_valid() {
        let from = ExecutionState::Idle;
        let to = ExecutionState::Running;
        assert!(from != to);
    }

    #[test]
    fn transition_running_to_succeeded_is_valid() {
        let from = ExecutionState::Running;
        let to = ExecutionState::Succeeded;
        assert!(from != to);
    }

    #[test]
    fn transition_running_to_failed_is_valid() {
        let from = ExecutionState::Running;
        let to = ExecutionState::Failed;
        assert!(from != to);
    }

    #[test]
    fn transition_waiting_to_running_is_valid() {
        let from = ExecutionState::Waiting;
        let to = ExecutionState::Running;
        assert!(from != to);
    }

    #[test]
    fn skipped_is_external_transition() {
        let state = ExecutionState::Skipped;
        assert!(state.is_terminal());
    }
}
