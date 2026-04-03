use proptest::prelude::*;

use super::{GhostDiffState, PendingProposal};

#[derive(Debug, Clone)]
enum GhostDiffAction {
    Receive(usize),
    Toggle(usize),
    AcceptAll,
    RejectAll,
    AcceptToggled,
}

fn action_strategy() -> impl Strategy<Value = GhostDiffAction> {
    prop_oneof![
        (0..100usize).prop_map(GhostDiffAction::Receive),
        (0..150usize).prop_map(GhostDiffAction::Toggle),
        Just(GhostDiffAction::AcceptAll),
        Just(GhostDiffAction::RejectAll),
        Just(GhostDiffAction::AcceptToggled),
    ]
}

fn make_proposal(n: usize) -> PendingProposal {
    PendingProposal {
        change_count: n,
        summary: format!("{n} changes"),
    }
}

proptest! {
    #[test]
    fn fuzz_operation_sequence(actions in prop::collection::vec(action_strategy(), 0..100)) {
        let mut state = GhostDiffState::new();

        for action in actions {
            match action {
                GhostDiffAction::Receive(size) => {
                    let _ = state.receive_proposal(make_proposal(size));
                }
                GhostDiffAction::Toggle(index) => {
                    let _ = state.toggle_change(index);
                }
                GhostDiffAction::AcceptAll => {
                    let _ = state.accept_all();
                }
                GhostDiffAction::RejectAll => {
                    let _ = state.reject_all();
                }
                GhostDiffAction::AcceptToggled => {
                    let _ = state.accept_toggled();
                }
            }
        }
    }
}
