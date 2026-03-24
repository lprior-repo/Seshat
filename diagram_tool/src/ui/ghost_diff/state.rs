#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    fn make_proposal(n: usize) -> PendingProposal {
        PendingProposal {
            change_count: n,
            summary: format!("{} changes", n),
        }
    }

    #[test]
    fn has_pending_returns_false_when_none() {
        let state = GhostDiffState::new();
        assert!(!state.has_pending());
    }

    #[test]
    fn has_pending_returns_true_when_some() {
        let state = GhostDiffState {
            pending: Some(make_proposal(3)),
            toggled: HashMap::new(),
        };
        assert!(state.has_pending());
    }

    #[test]
    fn change_count_returns_zero_when_no_pending() {
        let state = GhostDiffState::new();
        assert_eq!(state.change_count(), 0);
    }

    #[test]
    fn change_count_returns_pending_count() {
        let state = GhostDiffState {
            pending: Some(make_proposal(5)),
            toggled: HashMap::new(),
        };
        assert_eq!(state.change_count(), 5);
    }

    #[test]
    fn accepted_indices_empty_when_no_toggles() {
        let state = GhostDiffState {
            pending: Some(make_proposal(3)),
            toggled: HashMap::new(),
        };
        assert!(state.accepted_indices().is_empty());
    }

    #[test]
    fn accepted_indices_returns_only_true_values() {
        let mut toggled = HashMap::new();
        toggled.insert(0, true);
        toggled.insert(1, false);
        toggled.insert(2, true);

        let state = GhostDiffState {
            pending: Some(make_proposal(3)),
            toggled,
        };

        let accepted: Vec<usize> = state.accepted_indices();
        assert_eq!(accepted.len(), 2);
        assert!(accepted.contains(&0));
        assert!(accepted.contains(&2));
    }

    #[test]
    fn accepted_indices_sorted_ascending() {
        let mut toggled = HashMap::new();
        toggled.insert(5, true);
        toggled.insert(1, true);
        toggled.insert(3, true);

        let state = GhostDiffState {
            pending: Some(make_proposal(6)),
            toggled,
        };

        let accepted: Vec<usize> = state.accepted_indices();
        assert_eq!(accepted, vec![1, 3, 5]);
    }

    #[test]
    fn new_creates_empty_state() {
        let state = GhostDiffState::new();
        assert!(state.pending.is_none());
        assert!(state.toggled.is_empty());
        assert!(!state.has_pending());
        assert_eq!(state.change_count(), 0);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingProposal {
    pub change_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GhostDiffState {
    pub pending: Option<PendingProposal>,
    pub toggled: HashMap<usize, bool>,
}

impl GhostDiffState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending: None,
            toggled: HashMap::new(),
        }
    }

    #[must_use]
    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    #[must_use]
    pub fn change_count(&self) -> usize {
        self.pending.as_ref().map_or(0, |p| p.change_count)
    }

    #[must_use]
    pub fn accepted_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = self
            .toggled
            .iter()
            .filter(|(_, &accepted)| accepted)
            .map(|(&idx, _)| idx)
            .collect();
        indices.sort_unstable();
        indices
    }
}

impl Default for GhostDiffState {
    fn default() -> Self {
        Self::new()
    }
}
