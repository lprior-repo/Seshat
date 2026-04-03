use super::{GhostDiffState, GhostDiffStateMode, PendingProposal};

fn make_proposal(n: usize) -> PendingProposal {
    PendingProposal {
        change_count: n,
        summary: format!("{n} changes"),
    }
}

#[kani::proof]
fn kani_no_panics_on_state_machine_transitions() {
    let mut state = GhostDiffState::new();

    let op: u8 = kani::any();
    let size: usize = kani::any();
    let idx: usize = kani::any();

    kani::assume(size <= 10);
    kani::assume(idx <= 10);

    let initial_mode: u8 = kani::any();
    match initial_mode % 3 {
        0 => state.mode = GhostDiffStateMode::Idle,
        1 => {
            state.mode = GhostDiffStateMode::Reviewing;
            state.pending = Some(make_proposal(size));
        }
        _ => {
            state.mode = GhostDiffStateMode::Applying;
            state.pending = Some(make_proposal(size));
        }
    }

    match op % 5 {
        0 => {
            let _ = state.receive_proposal(make_proposal(size));
        }
        1 => {
            let _ = state.toggle_change(idx);
        }
        2 => {
            let _ = state.accept_all();
        }
        3 => {
            let _ = state.reject_all();
        }
        _ => {
            let _ = state.accept_toggled();
        }
    }
}

#[kani::proof]
fn kani_exhaustive_invariant_preservation() {
    let mut state = GhostDiffState::new();

    let initial_mode: u8 = kani::any();
    let size: usize = kani::any();
    kani::assume(size <= 5);

    match initial_mode % 3 {
        0 => {
            state.mode = GhostDiffStateMode::Idle;
            state.pending = None;
            state.toggled.clear();
        }
        1 => {
            state.mode = GhostDiffStateMode::Reviewing;
            state.pending = Some(make_proposal(size));
            for i in 0..size {
                state.toggled.insert(i, true);
            }
        }
        _ => {
            state.mode = GhostDiffStateMode::Applying;
            state.pending = Some(make_proposal(size));
        }
    }

    let op: u8 = kani::any();
    let idx: usize = kani::any();
    kani::assume(idx <= 10);

    match op % 5 {
        0 => {
            let _ = state.receive_proposal(make_proposal(size));
        }
        1 => {
            let _ = state.toggle_change(idx);
        }
        2 => {
            let _ = state.accept_all();
        }
        3 => {
            let _ = state.reject_all();
        }
        _ => {
            let _ = state.accept_toggled();
        }
    }

    match state.mode {
        GhostDiffStateMode::Idle => {
            assert!(state.pending.is_none());
            assert!(state.toggled.is_empty());
        }
        GhostDiffStateMode::Reviewing | GhostDiffStateMode::Applying => {
            assert!(state.pending.is_some());
            if let Some(ref pending) = state.pending {
                for key in state.toggled.keys() {
                    assert!(*key < pending.change_count);
                }
            }
        }
    }
}
