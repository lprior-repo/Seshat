//! Truncate stack tests for history module
//!
//! Tests for the history size limiting behavior via the public API.

#[cfg(kani)]
use crate::history::History;
#[cfg(kani)]
use diagram_models::document::{DiagramDocument, Revision};

#[cfg(kani)]
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

/// Direct test: empty history stays empty
#[cfg(kani)]
#[kani::proof]
#[test]
fn given_empty_history_then_undo_stack_is_empty() {
    let history = History::new();
    assert!(
        !history.can_undo(),
        "new history should have empty undo stack"
    );
}

/// Direct test: push creates undo entry
#[cfg(kani)]
#[kani::proof]
fn given_single_push_then_can_undo() {
    let history = History::new().push(doc_with_revision(1));
    assert!(history.can_undo(), "history with push should allow undo");
}

/// Direct test: more than cap when pushing, undo stack is capped at 100
#[cfg(kani)]
#[kani::proof]
#[test]
fn given_more_than_cap_when_pushing_then_undo_stack_is_capped_at_100() {
    let history = (0..105_u64).fold(History::new(), |acc, step| {
        acc.push(doc_with_revision(step))
    });

    // Safety: verify stack size is exactly 100 (not more)
    assert_eq!(
        history.undo_stack_len(),
        100,
        "undo_stack should be capped at 100"
    );
}

/// Direct test: capped history when undo all, exactly 100 undos succeed
#[cfg(kani)]
#[kani::proof]
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
