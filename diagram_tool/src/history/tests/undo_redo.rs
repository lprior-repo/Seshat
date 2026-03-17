//! Undo/Redo tests for history module
//!
//! Tests for the core undo and redo functionality.

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

/// Direct test of undo: returns correct document
#[cfg(kani)]
#[kani::proof]
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
#[cfg(kani)]
#[kani::proof]
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
        let undo_count = new_history.undo_stack_len();
        assert_eq!(
            undo_count, 2,
            "undo_stack should have 2 elements after undo"
        );

        // And redo_stack should have 1 element
        let redo_count = new_history.redo_stack_len();
        assert_eq!(redo_count, 1, "redo_stack should have 1 element after undo");
    }
}

/// Direct test of undo on empty history
#[cfg(kani)]
#[kani::proof]
#[test]
fn given_empty_history_when_undo_then_returns_none() {
    let history = History::new();
    let current = doc_with_revision(1);

    let result = history.undo(current);

    assert!(result.is_none(), "undo on empty history should return None");
}

/// Direct test of redo: returns correct document
#[cfg(kani)]
#[kani::proof]
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
#[cfg(kani)]
#[kani::proof]
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
#[cfg(kani)]
#[kani::proof]
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
#[cfg(kani)]
#[kani::proof]
#[test]
fn given_undone_state_when_push_then_redo_stack_is_cleared() {
    let history = History::new()
        .push(doc_with_revision(1))
        .push(doc_with_revision(2));

    let Some((_, after_undo)) = history.undo(doc_with_revision(3)) else {
        panic!("undo should succeed");
    };

    assert_eq!(
        after_undo.redo_stack_len(),
        1,
        "after undo, redo stack should have 1 element"
    );

    let after_push = after_undo.push(doc_with_revision(4));

    assert!(
        after_push.can_redo() == false,
        "push should clear redo stack"
    );
}

/// Verify undo returns correct document for multiple pushes (no loops)
#[cfg(kani)]
#[kani::proof]
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
#[cfg(kani)]
#[kani::proof]
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

/// Test can_undo returns false for fresh history
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_can_undo_returns_false_for_fresh_history() {
    let history = History::new();
    assert!(!history.can_undo());
}

/// Test can_undo returns true after push
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_can_undo_returns_true_after_push() {
    let history = History::new().push(doc_with_revision(1));
    assert!(history.can_undo());
}

/// Test can_redo returns false for fresh history
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_can_redo_returns_false_for_fresh_history() {
    let history = History::new();
    assert!(!history.can_redo());
}

/// Test can_redo returns true after undo
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_can_redo_returns_true_after_undo() {
    let history = History::new().push(doc_with_revision(1));
    let Some((_, after_undo)) = history.undo(doc_with_revision(100)) else {
        panic!("undo should succeed");
    };
    assert!(after_undo.can_redo());
}

/// Given multiple entries when undo then it walks back in order
#[cfg(kani)]
#[kani::proof]
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

/// Given cap boundary when undo and redo then round trip is sane
#[cfg(kani)]
#[kani::proof]
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
