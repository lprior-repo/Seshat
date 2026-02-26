#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::document::DiagramDocument;
use rpds::List;

/// Persistent history using persistent data structures (rpds)
#[derive(Clone, Default)]
pub struct History {
    undo_stack: List<DiagramDocument>,
    redo_stack: List<DiagramDocument>,
}

const MAX_HISTORY: usize = 100;

#[allow(clippy::needless_collect)]
fn truncate_stack(stack: &List<DiagramDocument>) -> List<DiagramDocument> {
    let capped = stack.iter().take(MAX_HISTORY).cloned().collect::<Vec<_>>();
    capped
        .into_iter()
        .rev()
        .fold(List::new(), |acc, entry| acc.push_front(entry))
}

#[allow(clippy::needless_collect)]
fn drop_first(stack: &List<DiagramDocument>) -> List<DiagramDocument> {
    let remainder = stack.iter().skip(1).cloned().collect::<Vec<_>>();
    remainder
        .into_iter()
        .rev()
        .fold(List::new(), |acc, entry| acc.push_front(entry))
}

impl History {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Pure transition to push a new state
    #[must_use]
    pub fn push(&self, doc: DiagramDocument) -> Self {
        Self {
            undo_stack: self.undo_stack.push_front(doc),
            redo_stack: List::new(),
        }
        .tap_history_limit()
    }

    /// Pure transition to undo
    #[must_use]
    pub fn undo(&self, current: DiagramDocument) -> Option<(DiagramDocument, Self)> {
        self.undo_stack.first().map(|prev| {
            (
                prev.clone(),
                Self {
                    undo_stack: drop_first(&self.undo_stack),
                    redo_stack: self.redo_stack.push_front(current),
                }
                .tap_history_limit(),
            )
        })
    }

    /// Pure transition to redo
    #[must_use]
    pub fn redo(&self, current: DiagramDocument) -> Option<(DiagramDocument, Self)> {
        self.redo_stack.first().map(|next| {
            (
                next.clone(),
                Self {
                    undo_stack: self.undo_stack.push_front(current),
                    redo_stack: drop_first(&self.redo_stack),
                }
                .tap_history_limit(),
            )
        })
    }

    #[must_use]
    pub fn tap_history_limit(self) -> Self {
        Self {
            undo_stack: truncate_stack(&self.undo_stack),
            redo_stack: truncate_stack(&self.redo_stack),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{History, List};
    use crate::models::document::{DiagramDocument, Revision};

    fn doc_with_revision(steps: u64) -> DiagramDocument {
        let mut revision = Revision::INITIAL;
        for _ in 0..steps {
            revision = revision.increment();
        }
        DiagramDocument {
            revision,
            ..DiagramDocument::default()
        }
    }

    #[test]
    fn given_more_than_cap_when_pushing_then_undo_stack_is_capped_at_100() {
        let history = (0..105_u64).fold(History::new(), |acc, step| {
            acc.push(doc_with_revision(step))
        });

        // Safety: verify stack size is exactly 100 (not more)
        assert_eq!(
            history.undo_stack.len(),
            100,
            "undo_stack should be capped at 100"
        );
    }

    #[test]
    fn given_capped_history_when_undo_all_then_exactly_100_undos_succeed() {
        let history = (0..105_u64).fold(History::new(), |acc, step| {
            acc.push(doc_with_revision(step))
        });
        let current = doc_with_revision(10_000);

        // Use explicit counter with safety limit to avoid infinite loops
        let mut next = history;
        let mut undo_count = 0_usize;
        const MAX_UNDOS: usize = 200; // Safety limit

        while undo_count < MAX_UNDOS {
            match next.undo(current.clone()) {
                Some((_, h)) => {
                    undo_count += 1;
                    next = h;
                }
                None => break,
            }
        }

        assert_eq!(undo_count, 100, "should have exactly 100 undos");
        assert!(undo_count < MAX_UNDOS, "should not hit safety limit");
    }

    #[test]
    fn given_multiple_entries_when_undo_then_it_walks_back_in_order() {
        let history = History::new()
            .push(doc_with_revision(1))
            .push(doc_with_revision(2))
            .push(doc_with_revision(3));

        let current = doc_with_revision(4);
        let first_undo = history.undo(current);
        assert!(first_undo.is_some());
        let Some((first, history)) = first_undo else {
            return;
        };

        let second_undo = history.undo(first.clone());
        assert!(second_undo.is_some());
        let Some((second, history)) = second_undo else {
            return;
        };

        let third_undo = history.undo(second.clone());
        assert!(third_undo.is_some());
        let Some((third, _history)) = third_undo else {
            return;
        };

        assert_eq!(first.revision, doc_with_revision(3).revision);
        assert_eq!(second.revision, doc_with_revision(2).revision);
        assert_eq!(third.revision, doc_with_revision(1).revision);
    }

    #[test]
    fn given_cap_boundary_when_undo_and_redo_then_round_trip_is_sane() {
        let history = (0..100_u64).fold(History::new(), |acc, step| {
            acc.push(doc_with_revision(step))
        });
        let current = doc_with_revision(500);

        let undo_result = history.undo(current.clone());
        assert!(undo_result.is_some());
        let Some((latest, after_undo)) = undo_result else {
            return;
        };

        let redo_result = after_undo.redo(latest.clone());
        assert!(redo_result.is_some());
        let Some((restored, _after_redo)) = redo_result else {
            return;
        };

        assert_eq!(latest.revision, doc_with_revision(99).revision);
        assert_eq!(restored.revision, current.revision);
    }

    // ============================================================
    // FAST TARGETED TESTS - catch mutation timeout issues directly
    // ============================================================

    /// Direct test of truncate_stack: empty stack stays empty
    #[test]
    fn given_empty_stack_when_truncate_then_returns_empty() {
        use super::{truncate_stack, List};
        let empty: List<DiagramDocument> = List::new();
        let result = truncate_stack(&empty);
        assert!(result.is_empty(), "empty stack should remain empty");
    }

    /// Direct test of truncate_stack: small stack unchanged
    #[test]
    fn given_small_stack_when_truncate_then_returns_same_elements() {
        use super::{truncate_stack, List};
        let stack = List::new()
            .push_front(doc_with_revision(1))
            .push_front(doc_with_revision(2))
            .push_front(doc_with_revision(3));

        let result = truncate_stack(&stack);

        // Verify all elements preserved in order
        let revisions: Vec<_> = result.iter().map(|d| d.revision).collect();
        assert_eq!(revisions.len(), 3, "small stack should not be truncated");
        assert_eq!(revisions[0], doc_with_revision(3).revision);
        assert_eq!(revisions[1], doc_with_revision(2).revision);
        assert_eq!(revisions[2], doc_with_revision(1).revision);
    }

    /// Direct test of truncate_stack: exact boundary (100 elements)
    #[test]
    fn given_exactly_100_elements_when_truncate_then_all_preserved() {
        use super::{truncate_stack, List};
        let stack = (0..100_u64).fold(List::new(), |acc: List<DiagramDocument>, i| {
            acc.push_front(doc_with_revision(i))
        });

        let result = truncate_stack(&stack);

        assert_eq!(
            result.len(),
            100,
            "exactly 100 elements should all be preserved"
        );
    }

    /// Direct test of truncate_stack: over limit gets truncated to 100
    #[test]
    fn given_105_elements_when_truncate_then_exactly_100_preserved() {
        use super::{truncate_stack, List};
        // Push 105 docs: first pushed has revision 0, last has revision 104
        let stack = (0..105_u64).fold(List::new(), |acc: List<DiagramDocument>, i| {
            acc.push_front(doc_with_revision(i))
        });

        let result = truncate_stack(&stack);

        assert_eq!(result.len(), 100, "should truncate to exactly 100");

        // Most recent (first in list) should be revision 104
        let first = result.iter().next();
        assert!(first.is_some(), "truncated stack should have elements");
        if let Some(doc) = first {
            assert_eq!(doc.revision, doc_with_revision(104).revision);
        }
    }

    /// Direct test of drop_first: empty stack stays empty
    #[test]
    fn given_empty_stack_when_drop_first_then_returns_empty() {
        use super::{drop_first, List};
        let empty: List<DiagramDocument> = List::new();
        let result = drop_first(&empty);
        assert!(result.is_empty(), "dropping from empty should return empty");
    }

    /// Direct test of drop_first: single element becomes empty
    #[test]
    fn given_single_element_when_drop_first_then_returns_empty() {
        use super::{drop_first, List};
        let stack = List::new().push_front(doc_with_revision(42));
        let result = drop_first(&stack);
        assert!(
            result.is_empty(),
            "dropping only element should return empty"
        );
    }

    /// Direct test of drop_first: removes first, preserves rest
    #[test]
    fn given_three_elements_when_drop_first_then_two_remain_in_order() {
        use super::{drop_first, List};
        // Stack: [rev3, rev2, rev1] (front to back)
        let stack = List::new()
            .push_front(doc_with_revision(1))
            .push_front(doc_with_revision(2))
            .push_front(doc_with_revision(3));

        let result = drop_first(&stack);

        let revisions: Vec<_> = result.iter().map(|d| d.revision).collect();
        assert_eq!(revisions.len(), 2, "should have 2 elements after drop");
        assert_eq!(revisions[0], doc_with_revision(2).revision);
        assert_eq!(revisions[1], doc_with_revision(1).revision);
    }

    /// Direct test of undo: returns correct document
    #[test]
    fn given_history_with_one_state_when_undo_then_returns_that_document() {
        let history = History::new().push(doc_with_revision(10));
        let current = doc_with_revision(20);

        let result = history.undo(current);

        assert!(result.is_some(), "undo should return Some");
        if let Some((restored_doc, _new_history)) = result {
            assert_eq!(
                restored_doc.revision,
                doc_with_revision(10).revision,
                "undo should return the pushed document"
            );
        }
    }

    /// Direct test of undo: returns correct new history state
    #[test]
    fn given_history_with_states_when_undo_then_new_history_has_dropped_first() {
        let history = History::new()
            .push(doc_with_revision(1))
            .push(doc_with_revision(2))
            .push(doc_with_revision(3));
        let current = doc_with_revision(100);

        let result = history.undo(current);

        assert!(result.is_some());
        if let Some((_doc, new_history)) = result {
            // After undo, the undo_stack should have 2 elements (dropped first)
            let undo_count = new_history.undo_stack.len();
            assert_eq!(
                undo_count, 2,
                "undo_stack should have 2 elements after undo"
            );

            // And redo_stack should have 1 element
            let redo_count = new_history.redo_stack.len();
            assert_eq!(redo_count, 1, "redo_stack should have 1 element after undo");
        }
    }

    /// Direct test of undo on empty history
    #[test]
    fn given_empty_history_when_undo_then_returns_none() {
        let history = History::new();
        let current = doc_with_revision(1);

        let result = history.undo(current);

        assert!(result.is_none(), "undo on empty history should return None");
    }

    /// Direct test of redo: returns correct document
    #[test]
    fn given_history_with_redo_state_when_redo_then_returns_that_document() {
        // Create history with one undo available
        let history = History::new().push(doc_with_revision(10));
        let current = doc_with_revision(20);

        let Some((_, after_undo)) = history.undo(current.clone()) else {
            panic!("undo should succeed");
        };

        let result = after_undo.redo(doc_with_revision(10));

        assert!(result.is_some(), "redo should return Some");
        if let Some((restored_doc, _new_history)) = result {
            assert_eq!(
                restored_doc.revision, current.revision,
                "redo should return the document that was current when undo was called"
            );
        }
    }

    /// Direct test of redo on empty redo stack
    #[test]
    fn given_fresh_history_when_redo_then_returns_none() {
        let history = History::new().push(doc_with_revision(1));
        let current = doc_with_revision(2);

        let result = history.redo(current);

        assert!(
            result.is_none(),
            "redo on fresh history (no undo done) should return None"
        );
    }

    /// Test undo then redo round trip with single element
    #[test]
    fn given_single_push_when_undo_then_redo_then_returns_to_current() {
        let original_current = doc_with_revision(999);
        let history = History::new().push(doc_with_revision(100));

        // Undo
        let Some((undo_doc, after_undo)) = history.undo(original_current.clone()) else {
            panic!("undo should succeed");
        };
        assert_eq!(undo_doc.revision, doc_with_revision(100).revision);

        // Redo
        let Some((redo_doc, _after_redo)) = after_undo.redo(undo_doc) else {
            panic!("redo should succeed");
        };
        assert_eq!(
            redo_doc.revision, original_current.revision,
            "redo should restore the original current document"
        );
    }

    /// Test that push clears redo stack
    #[test]
    fn given_undone_state_when_push_then_redo_stack_is_cleared() {
        let history = History::new()
            .push(doc_with_revision(1))
            .push(doc_with_revision(2));

        let Some((_, after_undo)) = history.undo(doc_with_revision(3)) else {
            panic!("undo should succeed");
        };

        assert_eq!(
            after_undo.redo_stack.len(),
            1,
            "after undo, redo stack should have 1 element"
        );

        let after_push = after_undo.push(doc_with_revision(4));

        assert!(
            after_push.redo_stack.is_empty(),
            "push should clear redo stack"
        );
    }

    /// Verify undo returns correct document for multiple pushes (no loops)
    #[test]
    fn given_three_pushes_when_undo_once_then_returns_most_recent_push() {
        let history = History::new()
            .push(doc_with_revision(1))
            .push(doc_with_revision(2))
            .push(doc_with_revision(3));

        let result = history.undo(doc_with_revision(100));

        assert!(result.is_some());
        if let Some((doc, _)) = result {
            assert_eq!(
                doc.revision,
                doc_with_revision(3).revision,
                "first undo should return most recently pushed document"
            );
        }
    }

    /// Verify undo order for second undo
    #[test]
    fn given_three_pushes_when_undo_twice_then_returns_second_push() {
        let history = History::new()
            .push(doc_with_revision(1))
            .push(doc_with_revision(2))
            .push(doc_with_revision(3));

        let Some((first, after_first)) = history.undo(doc_with_revision(100)) else {
            panic!("first undo should succeed");
        };
        assert_eq!(first.revision, doc_with_revision(3).revision);

        let Some((second, _)) = after_first.undo(first) else {
            panic!("second undo should succeed");
        };
        assert_eq!(
            second.revision,
            doc_with_revision(2).revision,
            "second undo should return second-to-last pushed document"
        );
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use crate::models::document::{DiagramDocument, Revision};
    use proptest::prelude::*;

    fn doc_with_revision(steps: u64) -> DiagramDocument {
        let mut revision = Revision::INITIAL;
        for _ in 0..steps {
            revision = revision.increment();
        }
        DiagramDocument {
            revision,
            ..DiagramDocument::default()
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_undo_on_empty_returns_none(rev in 0..1000u64) {
            let history = History::new();
            let current = doc_with_revision(rev);
            prop_assert!(history.undo(current).is_none());
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_redo_on_empty_returns_none(rev in 0..1000u64) {
            let history = History::new().push(doc_with_revision(rev));
            let current = doc_with_revision(rev + 100);
            prop_assert!(history.redo(current).is_none());
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_push_undo_roundtrip(current_rev in 0..1000u64, push_rev in 0..1000u64) {
            let history = History::new().push(doc_with_revision(push_rev));
            let current = doc_with_revision(current_rev);

            let (restored, after_undo) = history.undo(current.clone()).unwrap();
            prop_assert_eq!(restored.revision, doc_with_revision(push_rev).revision);

            let (redo_doc, _) = after_undo.redo(restored.clone()).unwrap();
            prop_assert_eq!(redo_doc.revision, current.revision);
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_undo_stack_bounded_at_100(pushes in 0..200usize) {
            let history = (0..pushes as u64).fold(History::new(), |acc, i| {
                acc.push(doc_with_revision(i))
            });
            prop_assert!(history.undo_stack.len() <= 100);
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_redo_stack_bounded_at_100(undos in 1..150usize) {
            let total_pushes = undos + 10;
            let history = (0..total_pushes as u64).fold(History::new(), |acc, i| {
                acc.push(doc_with_revision(i))
            });
            let current = doc_with_revision(10_000);

            let final_history = (0..undos).fold(Some((current, history)), |state, _| {
                state.and_then(|(curr, h)| h.undo(curr))
            }).map(|(_, h)| h);

            if let Some(h) = final_history {
                prop_assert!(h.redo_stack.len() <= 100);
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_sequential_pushes_recoverable(pushes in 1..50usize) {
            let history = (0..pushes as u64).fold(History::new(), |acc, i| {
                acc.push(doc_with_revision(i))
            });

            let mut current = doc_with_revision(10_000);
            let mut h = history;

            for expected_rev in (0..pushes as u64).rev() {
                let (restored, new_h) = h.undo(current.clone()).unwrap();
                prop_assert_eq!(restored.revision, doc_with_revision(expected_rev).revision);
                current = restored;
                h = new_h;
            }
            prop_assert!(h.undo(current).is_none());
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_capacity_maintained_after_many_ops(ops in 0..300usize) {
            let mut history = History::new();
            for i in 0..ops {
                history = history.push(doc_with_revision(i as u64));
            }
            prop_assert!(history.undo_stack.len() <= 100);
            prop_assert!(history.redo_stack.len() <= 100);
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_push_clears_redo_stack(initial_pushes in 1..20usize, undos in 1..10usize) {
            let history = (0..initial_pushes as u64).fold(History::new(), |acc, i| {
                acc.push(doc_with_revision(i))
            });

            let undos_to_do = undos.min(initial_pushes - 1).max(1);
            let current = doc_with_revision(10_000);

            let (_, after_undo) = (0..undos_to_do).fold(
                (current.clone(), history),
                |(curr, h), _| h.undo(curr).unwrap()
            );

            prop_assert!(!after_undo.redo_stack.is_empty());

            let after_push = after_undo.push(doc_with_revision(999));
            prop_assert!(after_push.redo_stack.is_empty());
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_undo_redo_idempotent(push_rev in 0..100u64, current_rev in 0..100u64) {
            let history = History::new().push(doc_with_revision(push_rev));
            let current = doc_with_revision(current_rev);

            let (undo_doc, after_undo) = history.undo(current.clone()).unwrap();
            let (redo_doc, _) = after_undo.redo(undo_doc.clone()).unwrap();

            let (undo_doc2, after_undo2) = history.undo(current.clone()).unwrap();
            let (redo_doc2, _) = after_undo2.redo(undo_doc2.clone()).unwrap();

            prop_assert_eq!(redo_doc.revision, redo_doc2.revision);
            prop_assert_eq!(undo_doc.revision, undo_doc2.revision);
        }
    }
}
