//! Undo/Redo tests for history module
//!
//! Tests for the core undo and redo functionality.

#[cfg(kani)]
use crate::history::History;
#[cfg(kani)]
use diagram_models::document::{DiagramDocument, Revision};

#[cfg(kani)]
fn doc(steps: u64) -> DiagramDocument {
    let mut revision = Revision::INITIAL;
    for _ in 0..steps {
        revision = revision.increment();
    }
    DiagramDocument {
        revision,
        ..DiagramDocument::default()
    }
}

#[cfg(kani)]
fn push_docs(count: u64) -> History {
    (1..=count).fold(History::new(), |h, i| h.push(doc(i)))
}

#[cfg(kani)]
#[kani::proof]
fn undo_returns_correct_document() {
    let (restored, _) = push_docs(1).undo(doc(20)).unwrap();
    assert_eq!(restored.revision, doc(1).revision);
}

#[cfg(kani)]
#[kani::proof]
fn undo_updates_history_stacks() {
    let (_, h) = push_docs(3).undo(doc(100)).unwrap();
    assert_eq!(h.undo_stack_len(), 2);
    assert_eq!(h.redo_stack_len(), 1);
}

#[cfg(kani)]
#[kani::proof]
fn undo_on_empty_returns_none() {
    assert!(History::new().undo(doc(1)).is_none());
}

#[cfg(kani)]
#[kani::proof]
fn redo_returns_correct_document() {
    let (_, after_undo) = push_docs(1).undo(doc(20)).unwrap();
    let (restored, _) = after_undo.redo(doc(10)).unwrap();
    assert_eq!(restored.revision, doc(20).revision);
}

#[cfg(kani)]
#[kani::proof]
fn redo_on_fresh_returns_none() {
    assert!(push_docs(1).redo(doc(2)).is_none());
}

#[cfg(kani)]
#[kani::proof]
fn undo_then_redo_round_trip() {
    let start = doc(999);
    let (undo_doc, after_undo) = push_docs(1).undo(start.clone()).unwrap();
    assert_eq!(undo_doc.revision, doc(1).revision);

    let (redo_doc, _) = after_undo.redo(undo_doc).unwrap();
    assert_eq!(redo_doc.revision, start.revision);
}

#[cfg(kani)]
#[kani::proof]
fn push_clears_redo_stack() {
    let (_, after_undo) = push_docs(2).undo(doc(3)).unwrap();
    assert_eq!(after_undo.redo_stack_len(), 1);
    assert!(!after_undo.push(doc(4)).can_redo());
}

#[cfg(kani)]
#[kani::proof]
fn multiple_pushes_undo_order() {
    let (first, h1) = push_docs(3).undo(doc(100)).unwrap();
    assert_eq!(first.revision, doc(3).revision);

    let (second, _) = h1.undo(first).unwrap();
    assert_eq!(second.revision, doc(2).revision);
}

#[cfg(kani)]
#[kani::proof]
fn can_undo_redo_states() {
    assert!(!History::new().can_undo());
    assert!(push_docs(1).can_undo());
    assert!(!History::new().can_redo());

    let (_, after_undo) = push_docs(1).undo(doc(100)).unwrap();
    assert!(after_undo.can_redo());
}

#[cfg(kani)]
#[kani::proof]
fn multiple_entries_walks_back_in_order() {
    let h = push_docs(3);
    let (d1, h) = h.undo(doc(4)).unwrap();
    let (d2, h) = h.undo(d1.clone()).unwrap();
    let (d3, _) = h.undo(d2.clone()).unwrap();

    assert_eq!(d1.revision, doc(3).revision);
    assert_eq!(d2.revision, doc(2).revision);
    assert_eq!(d3.revision, doc(1).revision);
}

#[cfg(kani)]
#[kani::proof]
fn cap_boundary_round_trip() {
    let h = push_docs(100);
    let current = doc(500);
    let (latest, h) = h.undo(current.clone()).unwrap();
    let (restored, _) = h.redo(latest.clone()).unwrap();

    assert_eq!(latest.revision, doc(100).revision);
    assert_eq!(restored.revision, current.revision);
}
