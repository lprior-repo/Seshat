use std::collections::HashMap;

use super::{GhostDiffState, GhostDiffStateMode, PendingProposal};

fn make_proposal(n: usize) -> PendingProposal {
    PendingProposal {
        change_count: n,
        summary: format!("{n} changes"),
    }
}

#[test]
#[ignore]
fn has_pending_returns_false_when_none() {
    let state = GhostDiffState::new();
    assert!(!state.has_pending());
}

#[test]
#[ignore]
fn has_pending_returns_true_when_some() {
    let state = GhostDiffState {
        mode: GhostDiffStateMode::Reviewing,
        pending: Some(make_proposal(3)),
        toggled: HashMap::new(),
    };
    assert!(state.has_pending());
}

#[test]
#[ignore]
fn change_count_returns_zero_when_no_pending() {
    let state = GhostDiffState::new();
    assert_eq!(state.change_count(), 0);
}

#[test]
#[ignore]
fn change_count_returns_pending_count() {
    let state = GhostDiffState {
        mode: GhostDiffStateMode::Reviewing,
        pending: Some(make_proposal(5)),
        toggled: HashMap::new(),
    };
    assert_eq!(state.change_count(), 5);
}

#[test]
#[ignore]
fn accepted_indices_empty_when_no_toggles() {
    let state = GhostDiffState {
        mode: GhostDiffStateMode::Reviewing,
        pending: Some(make_proposal(3)),
        toggled: HashMap::new(),
    };
    assert!(state.accepted_indices().is_empty());
}

#[test]
#[ignore]
fn accepted_indices_returns_only_true_values() {
    let toggled = HashMap::from([(0, true), (1, false), (2, true)]);
    let state = GhostDiffState {
        mode: GhostDiffStateMode::Reviewing,
        pending: Some(make_proposal(3)),
        toggled,
    };

    let accepted: Vec<usize> = state.accepted_indices();
    assert_eq!(accepted.len(), 2);
    assert!(accepted.contains(&0));
    assert!(accepted.contains(&2));
}

#[test]
#[ignore]
fn accepted_indices_sorted_ascending() {
    let toggled = HashMap::from([(5, true), (1, true), (3, true)]);
    let state = GhostDiffState {
        mode: GhostDiffStateMode::Reviewing,
        pending: Some(make_proposal(6)),
        toggled,
    };

    let accepted: Vec<usize> = state.accepted_indices();
    assert_eq!(accepted, vec![1, 3, 5]);
}

#[test]
#[ignore]
fn new_creates_empty_state() {
    let state = GhostDiffState::new();
    assert!(state.pending.is_none());
    assert!(state.toggled.is_empty());
    assert!(!state.has_pending());
    assert_eq!(state.change_count(), 0);
}
