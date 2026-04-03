#![allow(clippy::unwrap_used, unused_variables)]

use super::{GhostDiffError, GhostDiffState, GhostDiffStateMode, PendingProposal};

fn make_proposal(n: usize) -> PendingProposal {
    PendingProposal {
        change_count: n,
        summary: format!("{n} changes"),
    }
}

#[test]
#[ignore]
fn toggle_change_success_at_index_0() {
    let mut state = GhostDiffState::new();
    state.receive_proposal(make_proposal(3)).unwrap();

    let result = state.toggle_change(0);

    assert_eq!(result, Ok(()));
    assert_eq!(state.mode, GhostDiffStateMode::Reviewing);
    assert_eq!(state.toggled.get(&0), Some(&false));
    assert_eq!(state.toggled.get(&1), Some(&true));
    assert_eq!(state.toggled.get(&2), Some(&true));
}

#[test]
#[ignore]
fn toggle_change_success_at_max_valid_index() {
    let mut state = GhostDiffState::new();
    state.receive_proposal(make_proposal(3)).unwrap();

    let result = state.toggle_change(2);

    assert_eq!(result, Ok(()));
    assert_eq!(state.mode, GhostDiffStateMode::Reviewing);
    assert_eq!(state.toggled.get(&0), Some(&true));
    assert_eq!(state.toggled.get(&1), Some(&true));
    assert_eq!(state.toggled.get(&2), Some(&false));
}

#[test]
#[ignore]
fn toggle_change_double_toggle() {
    let mut state = GhostDiffState::new();
    state.receive_proposal(make_proposal(3)).unwrap();

    state.toggle_change(1).unwrap();
    let result = state.toggle_change(1);

    assert_eq!(result, Ok(()));
    assert_eq!(state.toggled.get(&1), Some(&true));
}

#[test]
#[ignore]
fn toggle_change_fails_exact_out_of_bounds() {
    let mut state = GhostDiffState::new();
    state.receive_proposal(make_proposal(2)).unwrap();

    let result = state.toggle_change(2);

    assert_eq!(
        result,
        Err(GhostDiffError::InvalidProposalIndex {
            index: 2,
            max_valid: 1
        })
    );
}

#[test]
#[ignore]
fn toggle_change_fails_far_out_of_bounds() {
    let mut state = GhostDiffState::new();
    state.receive_proposal(make_proposal(2)).unwrap();

    let result = state.toggle_change(10);

    assert_eq!(
        result,
        Err(GhostDiffError::InvalidProposalIndex {
            index: 10,
            max_valid: 1
        })
    );
}

#[test]
#[ignore]
fn toggle_change_fails_at_absolute_maximum_bounds() {
    let mut state = GhostDiffState::new();
    state.receive_proposal(make_proposal(2)).unwrap();

    let result = state.toggle_change(usize::MAX);

    assert_eq!(
        result,
        Err(GhostDiffError::InvalidProposalIndex {
            index: usize::MAX,
            max_valid: 1
        })
    );
}

#[test]
#[ignore]
fn toggle_change_fails_with_empty_proposal() {
    let mut state = GhostDiffState::new();
    state.receive_proposal(make_proposal(0)).unwrap();

    let result = state.toggle_change(0);

    assert_eq!(
        result,
        Err(GhostDiffError::InvalidProposalIndex {
            index: 0,
            max_valid: 0
        })
    );
}

#[test]
#[ignore]
fn toggle_change_fails_when_idle() {
    let mut state = GhostDiffState::new();

    let result = state.toggle_change(0);

    assert_eq!(
        result,
        Err(GhostDiffError::InvalidStateTransition {
            from: GhostDiffStateMode::Idle,
            action: "toggle_change".to_string()
        })
    );
}

#[test]
#[ignore]
fn toggle_change_fails_when_applying() {
    let mut state = GhostDiffState::new();
    state.receive_proposal(make_proposal(1)).unwrap();
    state.accept_all().unwrap();

    let result = state.toggle_change(0);

    assert_eq!(
        result,
        Err(GhostDiffError::InvalidStateTransition {
            from: GhostDiffStateMode::Applying,
            action: "toggle_change".to_string()
        })
    );
}

#[test]
#[ignore]
fn toggle_change_maintains_others() {
    let mut state = GhostDiffState::new();
    state.receive_proposal(make_proposal(5)).unwrap();

    let result = state.toggle_change(3);

    assert_eq!(result, Ok(()));
    assert_eq!(state.toggled.get(&3), Some(&false));
    assert_eq!(state.toggled.get(&0), Some(&true));
    assert_eq!(state.toggled.get(&1), Some(&true));
    assert_eq!(state.toggled.get(&2), Some(&true));
    assert_eq!(state.toggled.get(&4), Some(&true));
}
