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
fn accept_all_success() {
    let mut state = GhostDiffState::new();
    let proposal = make_proposal(3);
    state.receive_proposal(proposal.clone()).unwrap();

    let result = state.accept_all();

    assert_eq!(result, Ok(()));
    assert_eq!(state.mode, GhostDiffStateMode::Applying);
    assert_eq!(state.pending, Some(proposal));
}

#[test]
#[ignore]
fn accept_all_fails_when_idle() {
    let mut state = GhostDiffState::new();

    let result = state.accept_all();

    assert_eq!(
        result,
        Err(GhostDiffError::InvalidStateTransition {
            from: GhostDiffStateMode::Idle,
            action: "accept_all".to_string()
        })
    );
}

#[test]
#[ignore]
fn accept_all_fails_when_applying() {
    let mut state = GhostDiffState::new();
    state.receive_proposal(make_proposal(3)).unwrap();
    state.accept_all().unwrap();

    let result = state.accept_all();

    assert_eq!(
        result,
        Err(GhostDiffError::InvalidStateTransition {
            from: GhostDiffStateMode::Applying,
            action: "accept_all".to_string()
        })
    );
}

#[test]
#[ignore]
fn accept_all_missing_proposal() {
    let mut state = GhostDiffState::new();
    state.mode = GhostDiffStateMode::Reviewing;
    state.pending = None;

    let result = state.accept_all();

    assert_eq!(result, Err(GhostDiffError::NoPendingProposal));
}

#[test]
#[ignore]
fn accept_all_with_empty_proposal() {
    let mut state = GhostDiffState::new();
    let proposal = make_proposal(0);
    state.receive_proposal(proposal.clone()).unwrap();

    let result = state.accept_all();

    assert_eq!(result, Ok(()));
    assert_eq!(state.mode, GhostDiffStateMode::Applying);
    assert_eq!(state.pending, Some(proposal));
}

#[test]
#[ignore]
fn reject_all_success() {
    let mut state = GhostDiffState::new();
    state.receive_proposal(make_proposal(3)).unwrap();

    let result = state.reject_all();

    assert_eq!(result, Ok(()));
    assert_eq!(state.mode, GhostDiffStateMode::Idle);
    assert_eq!(state.pending, None);
    assert!(state.toggled.is_empty());
}

#[test]
#[ignore]
fn reject_all_fails_when_idle() {
    let mut state = GhostDiffState::new();

    let result = state.reject_all();

    assert_eq!(
        result,
        Err(GhostDiffError::InvalidStateTransition {
            from: GhostDiffStateMode::Idle,
            action: "reject_all".to_string()
        })
    );
}

#[test]
#[ignore]
fn reject_all_fails_when_applying() {
    let mut state = GhostDiffState::new();
    state.receive_proposal(make_proposal(3)).unwrap();
    state.accept_all().unwrap();

    let result = state.reject_all();

    assert_eq!(
        result,
        Err(GhostDiffError::InvalidStateTransition {
            from: GhostDiffStateMode::Applying,
            action: "reject_all".to_string()
        })
    );
}

#[test]
#[ignore]
fn reject_all_missing_proposal() {
    let mut state = GhostDiffState::new();
    state.mode = GhostDiffStateMode::Reviewing;
    state.pending = None;

    let result = state.reject_all();

    assert_eq!(result, Err(GhostDiffError::NoPendingProposal));
}

#[test]
#[ignore]
fn reject_all_with_empty_proposal() {
    let mut state = GhostDiffState::new();
    state.receive_proposal(make_proposal(0)).unwrap();

    let result = state.reject_all();

    assert_eq!(result, Ok(()));
    assert_eq!(state.mode, GhostDiffStateMode::Idle);
    assert_eq!(state.pending, None);
}

#[test]
#[ignore]
fn accept_toggled_success_partial() {
    let mut state = GhostDiffState::new();
    state.receive_proposal(make_proposal(3)).unwrap();
    state.toggle_change(1).unwrap();

    let result = state.accept_toggled();

    assert_eq!(result, Ok(()));
    assert_eq!(state.mode, GhostDiffStateMode::Applying);
    assert_eq!(state.pending, Some(make_proposal(2)));
}

#[test]
#[ignore]
fn accept_toggled_success_all_true() {
    let mut state = GhostDiffState::new();
    let proposal = make_proposal(3);
    state.receive_proposal(proposal.clone()).unwrap();

    let result = state.accept_toggled();

    assert_eq!(result, Ok(()));
    assert_eq!(state.mode, GhostDiffStateMode::Applying);
    assert_eq!(state.pending, Some(proposal));
}

#[test]
#[ignore]
fn accept_toggled_success_none_true() {
    let mut state = GhostDiffState::new();
    state.receive_proposal(make_proposal(3)).unwrap();
    state.toggle_change(0).unwrap();
    state.toggle_change(1).unwrap();
    state.toggle_change(2).unwrap();

    let result = state.accept_toggled();

    assert_eq!(result, Ok(()));
    assert_eq!(state.mode, GhostDiffStateMode::Applying);
    assert_eq!(state.pending, Some(make_proposal(0)));
}

#[test]
#[ignore]
fn accept_toggled_empty_proposal() {
    let mut state = GhostDiffState::new();
    let proposal = make_proposal(0);
    state.receive_proposal(proposal.clone()).unwrap();

    let result = state.accept_toggled();

    assert_eq!(result, Ok(()));
    assert_eq!(state.mode, GhostDiffStateMode::Applying);
    assert_eq!(state.pending, Some(proposal));
}

#[test]
#[ignore]
fn accept_toggled_fails_when_idle() {
    let mut state = GhostDiffState::new();

    let result = state.accept_toggled();

    assert_eq!(
        result,
        Err(GhostDiffError::InvalidStateTransition {
            from: GhostDiffStateMode::Idle,
            action: "accept_toggled".to_string()
        })
    );
}

#[test]
#[ignore]
fn accept_toggled_fails_when_applying() {
    let mut state = GhostDiffState::new();
    state.receive_proposal(make_proposal(3)).unwrap();
    state.accept_all().unwrap();

    let result = state.accept_toggled();

    assert_eq!(
        result,
        Err(GhostDiffError::InvalidStateTransition {
            from: GhostDiffStateMode::Applying,
            action: "accept_toggled".to_string()
        })
    );
}

#[test]
#[ignore]
fn accept_toggled_missing_proposal() {
    let mut state = GhostDiffState::new();
    state.mode = GhostDiffStateMode::Reviewing;
    state.pending = None;

    let result = state.accept_toggled();

    assert_eq!(result, Err(GhostDiffError::NoPendingProposal));
}

#[test]
#[ignore]
fn full_lifecycle_receive_toggle_accept_applies_correctly() {
    let mut state = GhostDiffState::new();

    assert_eq!(state.receive_proposal(make_proposal(3)), Ok(()));
    assert_eq!(state.toggle_change(1), Ok(()));
    assert_eq!(state.toggle_change(1), Ok(()));
    assert_eq!(state.toggle_change(2), Ok(()));
    assert_eq!(state.accept_toggled(), Ok(()));

    assert_eq!(state.mode, GhostDiffStateMode::Applying);
    assert_eq!(state.pending, Some(make_proposal(2)));
}
