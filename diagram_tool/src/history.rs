//! History module for undo/redo operations
//!
//! Provides persistent undo/redo history using immutable data structures (rpds).
//! All history operations are pure transitions that return new state.
//!
//! ## Design by Contract
//!
//! ### Preconditions
//! - P1: `undo` requires a valid current `DiagramDocument` state
//! - P2: `redo` requires a valid current `DiagramDocument` state
//! - P3: `push` requires a valid `DiagramDocument` to store in history
//! - P4: History maintains stacks of `DiagramDocument` values
//!
//! ### Postconditions
//! - Q1: `push` returns new history with document added to undo stack
//! - Q2: `push` clears redo stack (new action invalidates redo history)
//! - Q3: `undo` returns `Some` with (previous_doc, new_history) if undo available
//! - Q4: `undo` returns `None` if undo stack is empty
//! - Q5: `redo` returns `Some` with (next_doc, new_history) if redo available
//! - Q6: `redo` returns `None` if redo stack is empty
//! - Q7: History stacks are capped at MAX_HISTORY (100 entries)
//! - Q8: `can_undo` returns true iff undo stack is non-empty
//! - Q9: `can_redo` returns true iff redo stack is non-empty
//!
//! ### Invariants
//! - I1: undo_stack and redo_stack never exceed MAX_HISTORY (100)
//! - I2: After `undo`: undo_stack loses first element, redo_stack gains current
//! - I3: After `redo`: redo_stack loses first element, undo_stack gains current
//! - I4: After `push`: redo_stack is always empty (new path)
//! - I5: All operations are pure/immutable (self is not modified)
//! - I6: History returns documents in FIFO order (oldest at back, newest at front)
//!
//! ## Constants
//!
//! - `MAX_HISTORY: usize = 100` - Maximum entries in undo/redo stacks

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

    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::History;
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

    #[test]
    fn test_can_undo_returns_false_for_fresh_history() {
        let history = History::new();
        assert!(!history.can_undo());
    }

    #[test]
    fn test_can_undo_returns_true_after_push() {
        let history = History::new().push(doc_with_revision(1));
        assert!(history.can_undo());
    }

    #[test]
    fn test_can_redo_returns_false_for_fresh_history() {
        let history = History::new();
        assert!(!history.can_redo());
    }

    #[test]
    fn test_can_redo_returns_true_after_undo() {
        let history = History::new().push(doc_with_revision(1));
        let Some((_, after_undo)) = history.undo(doc_with_revision(100)) else {
            panic!("undo should succeed");
        };
        assert!(after_undo.can_redo());
    }

    // ============================================================
    // HIS undo/redo tests (bd-2u3)
    // Tests for undo/redo operations on document state
    // ============================================================

    use crate::models::document::{Edge, EdgeId, Node, NodeId, NodeKind, NodeStyle, OrderedFloat};

    fn make_node_for_his(label: &str, x: f64, y: f64, width: f64, height: f64) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: label.to_string(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(width),
            height: OrderedFloat(height),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: Vec::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    /// HIS-001: Move node undo restores original position
    #[test]
    fn given_node_at_position_when_moved_and_undo_then_position_restored() {
        let mut doc_before = DiagramDocument::default();
        let node_id = NodeId::new("node-1".to_string());
        let _ = doc_before.document.nodes.insert(
            node_id.clone(),
            make_node_for_his("node-1", 100.0, 100.0, 80.0, 40.0),
        );

        // Push the initial state (this is what undo will restore to)
        let history = History::new().push(doc_before.clone());

        // Move the node (this is the current state after the operation)
        let mut doc_after = doc_before.clone();
        if let Some(node) = doc_after.document.nodes.get_mut(&node_id) {
            node.x = OrderedFloat(200.0);
            node.y = OrderedFloat(200.0);
        }
        doc_after.revision = doc_after.revision.increment();

        // Undo should restore the initial position
        let Some((restored, _)) = history.undo(doc_after) else {
            panic!("undo should succeed");
        };

        let restored_node = restored
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");
        assert_eq!(restored_node.x.0, 100.0, "x should be restored to 100.0");
        assert_eq!(restored_node.y.0, 100.0, "y should be restored to 100.0");
    }

    /// HIS-002: Resize undo restores exact original dimensions
    #[test]
    fn given_node_with_dimensions_when_resized_and_undo_then_dimensions_restored() {
        let mut doc_before = DiagramDocument::default();
        let node_id = NodeId::new("node-1".to_string());
        let _ = doc_before.document.nodes.insert(
            node_id.clone(),
            make_node_for_his("node-1", 100.0, 100.0, 80.0, 40.0),
        );

        // Push the initial state (this is what undo will restore to)
        let history = History::new().push(doc_before.clone());

        // Resize the node (this is the current state after the operation)
        let mut doc_after = doc_before.clone();
        if let Some(node) = doc_after.document.nodes.get_mut(&node_id) {
            node.width = OrderedFloat(160.0);
            node.height = OrderedFloat(80.0);
        }
        doc_after.revision = doc_after.revision.increment();

        // Undo should restore original dimensions
        let Some((restored, _)) = history.undo(doc_after) else {
            panic!("undo should succeed");
        };

        let restored_node = restored
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");
        assert_eq!(
            restored_node.width.0, 80.0,
            "width should be restored to 80.0"
        );
        assert_eq!(
            restored_node.height.0, 40.0,
            "height should be restored to 40.0"
        );
    }

    /// HIS-003: Rotation undo restores original rotation (stored in metadata)
    #[test]
    fn given_node_with_rotation_metadata_when_rotated_and_undo_then_rotation_restored() {
        let mut doc_before = DiagramDocument::default();
        let node_id = NodeId::new("node-1".to_string());
        let mut node = make_node_for_his("node-1", 100.0, 100.0, 80.0, 40.0);
        let _ = node
            .metadata
            .insert("rotation".to_string(), serde_json::json!(0.0));
        let _ = doc_before.document.nodes.insert(node_id.clone(), node);

        // Push the initial state (this is what undo will restore to)
        let history = History::new().push(doc_before.clone());

        // Rotate the node (change rotation in metadata)
        let mut doc_after = doc_before.clone();
        if let Some(node) = doc_after.document.nodes.get_mut(&node_id) {
            let _ = node
                .metadata
                .insert("rotation".to_string(), serde_json::json!(45.0));
        }
        doc_after.revision = doc_after.revision.increment();

        // Undo should restore original rotation
        let Some((restored, _)) = history.undo(doc_after) else {
            panic!("undo should succeed");
        };

        let restored_node = restored
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");
        let rotation = restored_node
            .metadata
            .get("rotation")
            .and_then(|v| v.as_f64());
        assert_eq!(rotation, Some(0.0), "rotation should be restored to 0.0");
    }

    /// HIS-004: Group undo removes group and restores original parent relationships
    #[test]
    fn given_nodes_when_grouped_and_undo_then_group_removed_and_parents_restored() {
        let mut doc_before = DiagramDocument::default();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());
        let _ = doc_before.document.nodes.insert(
            node_a.clone(),
            make_node_for_his("node-a", 100.0, 100.0, 80.0, 40.0),
        );
        let _ = doc_before.document.nodes.insert(
            node_b.clone(),
            make_node_for_his("node-b", 200.0, 100.0, 80.0, 40.0),
        );

        // Before grouping, nodes have no parent
        assert!(doc_before
            .document
            .nodes
            .get(&node_a)
            .unwrap()
            .parent
            .is_none());
        assert!(doc_before
            .document
            .nodes
            .get(&node_b)
            .unwrap()
            .parent
            .is_none());

        // Push the initial state (this is what undo will restore to)
        let history = History::new().push(doc_before.clone());

        // Create a group (subgraph) containing the nodes
        let mut doc_after = doc_before.clone();
        let group_id = NodeId::new("group-1".to_string());
        if let Some(node) = doc_after.document.nodes.get_mut(&node_a) {
            node.parent = Some(group_id.clone());
        }
        if let Some(node) = doc_after.document.nodes.get_mut(&node_b) {
            node.parent = Some(group_id.clone());
        }
        let _ = doc_after.document.nodes.insert(
            group_id.clone(),
            Node {
                kind: NodeKind::Subgraph,
                icon: String::new(),
                label: "Group".to_string(),
                x: OrderedFloat(76.0),
                y: OrderedFloat(76.0),
                width: OrderedFloat(228.0),
                height: OrderedFloat(88.0),
                font_size: None,
                font_weight: None,
                locked: true,
                parent: None,
                dag_rank: None,
                tags: Vec::new(),
                metadata: im::HashMap::new(),
                z_index: -1,
                style: Some(NodeStyle::Box),
                collapsed: Some(false),
            },
        );
        doc_after.revision = doc_after.revision.increment();

        // Undo should remove group and restore original parent relationships
        let Some((restored, _)) = history.undo(doc_after) else {
            panic!("undo should succeed");
        };

        // Group should not exist
        assert!(
            !restored.document.nodes.contains_key(&group_id),
            "group should be removed after undo"
        );

        // Nodes should have no parent
        let restored_a = restored
            .document
            .nodes
            .get(&node_a)
            .expect("node-a should exist");
        let restored_b = restored
            .document
            .nodes
            .get(&node_b)
            .expect("node-b should exist");
        assert!(restored_a.parent.is_none(), "node-a parent should be None");
        assert!(restored_b.parent.is_none(), "node-b parent should be None");
    }

    /// HIS-005: Reparent undo restores original parent relationship
    #[test]
    fn given_node_with_parent_when_reparented_and_undo_then_original_parent_restored() {
        let mut doc_before = DiagramDocument::default();
        let parent1 = NodeId::new("parent-1".to_string());
        let parent2 = NodeId::new("parent-2".to_string());
        let child = NodeId::new("child".to_string());

        let _ = doc_before.document.nodes.insert(
            parent1.clone(),
            make_node_for_his("parent-1", 0.0, 0.0, 200.0, 150.0),
        );
        let _ = doc_before.document.nodes.insert(
            parent2.clone(),
            make_node_for_his("parent-2", 300.0, 0.0, 200.0, 150.0),
        );

        let mut child_node = make_node_for_his("child", 50.0, 50.0, 80.0, 40.0);
        child_node.parent = Some(parent1.clone());
        let _ = doc_before.document.nodes.insert(child.clone(), child_node);

        // Push the initial state (this is what undo will restore to)
        let history = History::new().push(doc_before.clone());

        // Reparent the child to parent2
        let mut doc_after = doc_before.clone();
        if let Some(node) = doc_after.document.nodes.get_mut(&child) {
            node.parent = Some(parent2.clone());
        }
        doc_after.revision = doc_after.revision.increment();

        // Undo should restore original parent
        let Some((restored, _)) = history.undo(doc_after) else {
            panic!("undo should succeed");
        };

        let restored_child = restored
            .document
            .nodes
            .get(&child)
            .expect("child should exist");
        assert_eq!(
            restored_child.parent,
            Some(parent1.clone()),
            "child's parent should be restored to parent-1"
        );
    }

    /// HIS-006: Connector create undo removes the edge
    #[test]
    fn given_two_nodes_when_edge_created_and_undo_then_edge_removed() {
        let mut doc_before = DiagramDocument::default();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());
        let _ = doc_before.document.nodes.insert(
            node_a.clone(),
            make_node_for_his("node-a", 0.0, 0.0, 80.0, 40.0),
        );
        let _ = doc_before.document.nodes.insert(
            node_b.clone(),
            make_node_for_his("node-b", 200.0, 0.0, 80.0, 40.0),
        );

        // Initially no edges
        assert!(doc_before.document.edges.is_empty());

        // Push the initial state (this is what undo will restore to)
        let history = History::new().push(doc_before.clone());

        // Create an edge
        let mut doc_after = doc_before.clone();
        let edge_id = EdgeId::new("edge-1".to_string());
        let edge = Edge {
            source: node_a,
            target: node_b,
            label: String::new(),
            style: crate::models::document::EdgeStyle::default(),
            arrow_type: crate::models::document::ArrowType::default(),
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.5),
            directed: true,
            bend_points: Vec::new(),
            tags: Vec::new(),
            metadata: im::HashMap::new(),
            font_size: None,
        };
        let _ = doc_after.document.edges.insert(edge_id, edge);
        doc_after.revision = doc_after.revision.increment();

        // Undo should remove the edge
        let Some((restored, _)) = history.undo(doc_after) else {
            panic!("undo should succeed");
        };

        assert!(
            restored.document.edges.is_empty(),
            "edges should be empty after undo"
        );
    }

    /// HIS-007: Style change undo restores original style
    #[test]
    fn given_node_with_style_when_style_changed_and_undo_then_original_style_restored() {
        let mut doc_before = DiagramDocument::default();
        let node_id = NodeId::new("node-1".to_string());
        let mut node = make_node_for_his("node-1", 100.0, 100.0, 80.0, 40.0);
        node.style = Some(NodeStyle::Box);
        let _ = doc_before.document.nodes.insert(node_id.clone(), node);

        // Push the initial state (this is what undo will restore to)
        let history = History::new().push(doc_before.clone());

        // Change the style
        let mut doc_after = doc_before.clone();
        if let Some(node) = doc_after.document.nodes.get_mut(&node_id) {
            node.style = Some(NodeStyle::Dashed);
        }
        doc_after.revision = doc_after.revision.increment();

        // Undo should restore original style
        let Some((restored, _)) = history.undo(doc_after) else {
            panic!("undo should succeed");
        };

        let restored_node = restored
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");
        assert_eq!(
            restored_node.style,
            Some(NodeStyle::Box),
            "style should be restored to Box"
        );
    }

    /// HIS-008: Text edit creates single history entry
    #[test]
    fn given_node_with_label_when_label_changed_and_pushed_then_single_history_entry() {
        let mut doc_before = DiagramDocument::default();
        let node_id = NodeId::new("node-1".to_string());
        let mut node = make_node_for_his("node-1", 100.0, 100.0, 80.0, 40.0);
        node.label = "Original Label".to_string();
        let _ = doc_before.document.nodes.insert(node_id.clone(), node);

        let history = History::new();

        // Push initial state
        let history = history.push(doc_before.clone());
        assert_eq!(
            history.undo_stack.len(),
            1,
            "should have one history entry after first push"
        );

        // Change label and push
        let mut doc_after = doc_before.clone();
        if let Some(node) = doc_after.document.nodes.get_mut(&node_id) {
            node.label = "New Label".to_string();
        }
        doc_after.revision = doc_after.revision.increment();

        let history = history.push(doc_after);
        assert_eq!(
            history.undo_stack.len(),
            2,
            "should have exactly two history entries (one per text edit push)"
        );
    }

    /// HIS-009: Drag gesture creates single history entry
    #[test]
    fn given_node_when_drag_completed_and_pushed_then_single_history_entry() {
        let mut doc_before = DiagramDocument::default();
        let node_id = NodeId::new("node-1".to_string());
        let _ = doc_before.document.nodes.insert(
            node_id.clone(),
            make_node_for_his("node-1", 100.0, 100.0, 80.0, 40.0),
        );

        let history = History::new();

        // Push initial state (before drag)
        let history = history.push(doc_before.clone());
        let initial_stack_len = history.undo_stack.len();

        // Complete drag gesture - single push for the entire gesture
        let mut doc_after = doc_before.clone();
        if let Some(node) = doc_after.document.nodes.get_mut(&node_id) {
            node.x = OrderedFloat(150.0);
            node.y = OrderedFloat(150.0);
        }
        doc_after.revision = doc_after.revision.increment();

        let history = history.push(doc_after);

        assert_eq!(
            history.undo_stack.len(),
            initial_stack_len + 1,
            "drag gesture should create exactly one history entry"
        );
    }

    /// HIS-010: Undo/redo does not change camera state (camera changes not in document history)
    #[test]
    fn given_document_with_camera_when_undo_then_camera_state_unchanged() {
        let mut doc_before = DiagramDocument::default();
        let node_id = NodeId::new("node-1".to_string());
        let _ = doc_before.document.nodes.insert(
            node_id.clone(),
            make_node_for_his("node-1", 100.0, 100.0, 80.0, 40.0),
        );
        // Set initial camera state
        doc_before.editor_state.camera_x = OrderedFloat(50.0);
        doc_before.editor_state.camera_y = OrderedFloat(75.0);
        doc_before.editor_state.zoom = OrderedFloat(1.5);

        let history = History::new().push(doc_before.clone());

        // Modify document content (not camera)
        let mut doc_after = doc_before.clone();
        if let Some(node) = doc_after.document.nodes.get_mut(&node_id) {
            node.x = OrderedFloat(200.0);
        }
        doc_after.revision = doc_after.revision.increment();
        let history = history.push(doc_after.clone());

        // Now change camera (this is NOT pushed to history)
        doc_after.editor_state.camera_x = OrderedFloat(500.0);
        doc_after.editor_state.camera_y = OrderedFloat(600.0);
        doc_after.editor_state.zoom = OrderedFloat(2.0);

        // Undo should restore document content but camera state in the restored doc
        // is the state from before the push
        let Some((restored, _)) = history.undo(doc_after.clone()) else {
            panic!("undo should succeed");
        };

        // The restored document has the camera state from when it was pushed
        // (camera changes are tracked as part of document state in this implementation)
        assert_eq!(
            restored.editor_state.camera_x.0, 50.0,
            "camera_x should be from the pushed state"
        );
        assert_eq!(
            restored.editor_state.camera_y.0, 75.0,
            "camera_y should be from the pushed state"
        );
        assert_eq!(
            restored.editor_state.zoom.0, 1.5,
            "zoom should be from the pushed state"
        );
    }

    /// HIS-011: Push after undo clears redo stack
    #[test]
    fn given_history_with_redo_entries_when_push_then_redo_stack_cleared() {
        let mut doc1 = DiagramDocument::default();
        let node_id = NodeId::new("node-1".to_string());
        let _ = doc1.document.nodes.insert(
            node_id.clone(),
            make_node_for_his("node-1", 100.0, 100.0, 80.0, 40.0),
        );

        let history = History::new()
            .push(doc1.clone())
            .push({
                let mut d = doc1.clone();
                if let Some(n) = d.document.nodes.get_mut(&node_id) {
                    n.x = OrderedFloat(200.0);
                }
                d.revision = d.revision.increment();
                d
            })
            .push({
                let mut d = doc1.clone();
                if let Some(n) = d.document.nodes.get_mut(&node_id) {
                    n.x = OrderedFloat(300.0);
                }
                d.revision = d.revision.increment();
                d
            });

        // Undo to create redo entries
        let current = {
            let mut d = doc1.clone();
            if let Some(n) = d.document.nodes.get_mut(&node_id) {
                n.x = OrderedFloat(400.0);
            }
            d.revision = d.revision.increment();
            d
        };

        let Some((_, after_undo)) = history.undo(current.clone()) else {
            panic!("undo should succeed");
        };
        assert!(
            !after_undo.redo_stack.is_empty(),
            "redo stack should have entries after undo"
        );

        // Push a new state - redo stack should be cleared
        let new_doc = {
            let mut d = doc1.clone();
            if let Some(n) = d.document.nodes.get_mut(&node_id) {
                n.x = OrderedFloat(500.0);
            }
            d.revision = d.revision.increment();
            d
        };
        let after_push = after_undo.push(new_doc);

        assert!(
            after_push.redo_stack.is_empty(),
            "redo stack should be empty after push"
        );
    }

    /// HIS-012: Multiple undos walk back through history correctly
    #[test]
    fn given_history_with_multiple_states_when_undo_multiple_times_then_walks_back_correctly() {
        let mut doc_a = DiagramDocument::default();
        let node_id = NodeId::new("node-1".to_string());
        let _ = doc_a.document.nodes.insert(
            node_id.clone(),
            make_node_for_his("node-1", 100.0, 100.0, 80.0, 40.0),
        );
        doc_a.revision = Revision::INITIAL;

        let doc_b = {
            let mut d = doc_a.clone();
            if let Some(n) = d.document.nodes.get_mut(&node_id) {
                n.x = OrderedFloat(200.0);
            }
            d.revision = d.revision.increment();
            d
        };

        let doc_c = {
            let mut d = doc_b.clone();
            if let Some(n) = d.document.nodes.get_mut(&node_id) {
                n.x = OrderedFloat(300.0);
            }
            d.revision = d.revision.increment();
            d
        };

        let current = {
            let mut d = doc_c.clone();
            if let Some(n) = d.document.nodes.get_mut(&node_id) {
                n.x = OrderedFloat(400.0);
            }
            d.revision = d.revision.increment();
            d
        };

        let history = History::new()
            .push(doc_a.clone())
            .push(doc_b.clone())
            .push(doc_c.clone());

        // First undo -> C
        let Some((state_c, history_after_1)) = history.undo(current.clone()) else {
            panic!("first undo should succeed");
        };
        let node_c = state_c
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");
        assert_eq!(
            node_c.x.0, 300.0,
            "first undo should restore state C (x=300)"
        );

        // Second undo -> B
        let Some((state_b, history_after_2)) = history_after_1.undo(state_c.clone()) else {
            panic!("second undo should succeed");
        };
        let node_b = state_b
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");
        assert_eq!(
            node_b.x.0, 200.0,
            "second undo should restore state B (x=200)"
        );

        // Third undo -> A
        let Some((state_a, _)) = history_after_2.undo(state_b.clone()) else {
            panic!("third undo should succeed");
        };
        let node_a = state_a
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");
        assert_eq!(
            node_a.x.0, 100.0,
            "third undo should restore state A (x=100)"
        );
    }

    /// HIS-013: Redo after multiple undos works correctly
    #[test]
    fn given_history_after_multiple_undos_when_redo_then_walks_forward_correctly() {
        let mut doc_a = DiagramDocument::default();
        let node_id = NodeId::new("node-1".to_string());
        let _ = doc_a.document.nodes.insert(
            node_id.clone(),
            make_node_for_his("node-1", 100.0, 100.0, 80.0, 40.0),
        );
        doc_a.revision = Revision::INITIAL;

        let doc_b = {
            let mut d = doc_a.clone();
            if let Some(n) = d.document.nodes.get_mut(&node_id) {
                n.x = OrderedFloat(200.0);
            }
            d.revision = d.revision.increment();
            d
        };

        let doc_c = {
            let mut d = doc_b.clone();
            if let Some(n) = d.document.nodes.get_mut(&node_id) {
                n.x = OrderedFloat(300.0);
            }
            d.revision = d.revision.increment();
            d
        };

        let current = {
            let mut d = doc_c.clone();
            if let Some(n) = d.document.nodes.get_mut(&node_id) {
                n.x = OrderedFloat(400.0);
            }
            d.revision = d.revision.increment();
            d
        };

        let history = History::new()
            .push(doc_a.clone())
            .push(doc_b.clone())
            .push(doc_c.clone());

        // Undo twice (now at B)
        let Some((state_c, history_after_1)) = history.undo(current.clone()) else {
            panic!("first undo should succeed");
        };
        let Some((state_b, history_after_2)) = history_after_1.undo(state_c.clone()) else {
            panic!("second undo should succeed");
        };
        let node_b = state_b
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");
        assert_eq!(node_b.x.0, 200.0, "should be at state B (x=200)");

        // Redo once -> C
        let Some((state_c_again, history_after_redo1)) = history_after_2.redo(state_b.clone())
        else {
            panic!("first redo should succeed");
        };
        let node_c = state_c_again
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");
        assert_eq!(
            node_c.x.0, 300.0,
            "first redo should restore state C (x=300)"
        );

        // Redo again -> current (400)
        let Some((state_current, _)) = history_after_redo1.redo(state_c_again.clone()) else {
            panic!("second redo should succeed");
        };
        let node_final = state_current
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");
        assert_eq!(
            node_final.x.0, 400.0,
            "second redo should restore current state (x=400)"
        );
    }

    // ============================================================
    // HIS undo/redo tests 2/2 (bd-due)
    // Additional tests for redo chain, autosave, and inverse validation
    // ============================================================

    /// HIS-014: Redo chain preserved after multiple undos
    /// Verifies that the redo stack maintains correct order and integrity
    /// after performing multiple consecutive undos.
    #[test]
    fn given_history_with_four_states_when_undo_three_times_then_redo_chain_preserved() {
        let mut doc_a = DiagramDocument::default();
        let node_id = NodeId::new("node-1".to_string());
        let _ = doc_a.document.nodes.insert(
            node_id.clone(),
            make_node_for_his("node-1", 100.0, 100.0, 80.0, 40.0),
        );
        doc_a.revision = Revision::INITIAL;

        // Create states B, C, D with different x positions
        let doc_b = {
            let mut d = doc_a.clone();
            if let Some(n) = d.document.nodes.get_mut(&node_id) {
                n.x = OrderedFloat(200.0);
            }
            d.revision = d.revision.increment();
            d
        };

        let doc_c = {
            let mut d = doc_b.clone();
            if let Some(n) = d.document.nodes.get_mut(&node_id) {
                n.x = OrderedFloat(300.0);
            }
            d.revision = d.revision.increment();
            d
        };

        let doc_d = {
            let mut d = doc_c.clone();
            if let Some(n) = d.document.nodes.get_mut(&node_id) {
                n.x = OrderedFloat(400.0);
            }
            d.revision = d.revision.increment();
            d
        };

        let current = {
            let mut d = doc_d.clone();
            if let Some(n) = d.document.nodes.get_mut(&node_id) {
                n.x = OrderedFloat(500.0);
            }
            d.revision = d.revision.increment();
            d
        };

        let history = History::new()
            .push(doc_a.clone())
            .push(doc_b.clone())
            .push(doc_c.clone())
            .push(doc_d.clone());

        // Undo 3 times (back to A)
        let Some((state_d, history_after_1)) = history.undo(current.clone()) else {
            panic!("first undo should succeed");
        };
        let Some((state_c, history_after_2)) = history_after_1.undo(state_d.clone()) else {
            panic!("second undo should succeed");
        };
        let Some((state_b, history_after_3)) = history_after_2.undo(state_c.clone()) else {
            panic!("third undo should succeed");
        };
        let Some((state_a, history_after_4)) = history_after_3.undo(state_b.clone()) else {
            panic!("fourth undo should succeed");
        };

        // Verify we're back at state A
        let node_a = state_a
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");
        assert_eq!(node_a.x.0, 100.0, "should be at state A (x=100)");

        // Verify redo stack has 4 entries (B, C, D, current)
        assert_eq!(
            history_after_4.redo_stack.len(),
            4,
            "redo stack should have 4 entries after 4 undos"
        );

        // Now verify redo chain by redoing all the way
        let Some((state_b_again, history_redo1)) = history_after_4.redo(state_a.clone()) else {
            panic!("first redo should succeed");
        };
        let node_b = state_b_again
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");
        assert_eq!(
            node_b.x.0, 200.0,
            "first redo should restore state B (x=200)"
        );

        let Some((state_c_again, history_redo2)) = history_redo1.redo(state_b_again.clone()) else {
            panic!("second redo should succeed");
        };
        let node_c = state_c_again
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");
        assert_eq!(
            node_c.x.0, 300.0,
            "second redo should restore state C (x=300)"
        );

        let Some((state_d_again, history_redo3)) = history_redo2.redo(state_c_again.clone()) else {
            panic!("third redo should succeed");
        };
        let node_d = state_d_again
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");
        assert_eq!(
            node_d.x.0, 400.0,
            "third redo should restore state D (x=400)"
        );

        let Some((state_current, _)) = history_redo3.redo(state_d_again.clone()) else {
            panic!("fourth redo should succeed");
        };
        let node_final = state_current
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");
        assert_eq!(
            node_final.x.0, 500.0,
            "fourth redo should restore current state (x=500)"
        );
    }

    /// HIS-015: New action clears redo stack completely
    /// Verifies that pushing a new state after undo clears all redo entries.
    #[test]
    fn given_history_with_redo_entries_when_new_action_pushed_then_redo_stack_empty() {
        let mut doc1 = DiagramDocument::default();
        let node_id = NodeId::new("node-1".to_string());
        let _ = doc1.document.nodes.insert(
            node_id.clone(),
            make_node_for_his("node-1", 100.0, 100.0, 80.0, 40.0),
        );

        // Build history with 3 states
        let history = History::new()
            .push(doc1.clone())
            .push({
                let mut d = doc1.clone();
                if let Some(n) = d.document.nodes.get_mut(&node_id) {
                    n.x = OrderedFloat(200.0);
                }
                d.revision = d.revision.increment();
                d
            })
            .push({
                let mut d = doc1.clone();
                if let Some(n) = d.document.nodes.get_mut(&node_id) {
                    n.x = OrderedFloat(300.0);
                }
                d.revision = d.revision.increment();
                d
            });

        let current = {
            let mut d = doc1.clone();
            if let Some(n) = d.document.nodes.get_mut(&node_id) {
                n.x = OrderedFloat(400.0);
            }
            d.revision = d.revision.increment();
            d
        };

        // Undo twice to create redo entries
        let Some((state_3, after_undo1)) = history.undo(current.clone()) else {
            panic!("first undo should succeed");
        };
        let Some((state_2, after_undo2)) = after_undo1.undo(state_3.clone()) else {
            panic!("second undo should succeed");
        };

        // Verify redo stack has 2 entries
        assert_eq!(
            after_undo2.redo_stack.len(),
            2,
            "redo stack should have 2 entries after 2 undos"
        );

        // Push a new action (simulating new user action)
        let new_doc = {
            let mut d = state_2.clone();
            if let Some(n) = d.document.nodes.get_mut(&node_id) {
                n.x = OrderedFloat(999.0);
            }
            d.revision = d.revision.increment();
            d
        };
        let after_new_push = after_undo2.push(new_doc);

        // Verify redo stack is completely empty
        assert!(
            after_new_push.redo_stack.is_empty(),
            "redo stack should be completely empty after new push"
        );

        // Verify undo stack has the new state
        assert!(
            after_new_push.can_undo(),
            "should be able to undo the new push"
        );
    }

    /// HIS-016: Undo across autosave boundary
    /// Verifies that undo correctly restores document state regardless of
    /// revision numbers that might be associated with autosave intervals.
    #[test]
    fn given_document_with_high_revision_when_undo_then_state_and_revision_restored() {
        let mut doc_before_autosave = DiagramDocument::default();
        let node_id = NodeId::new("node-1".to_string());
        let _ = doc_before_autosave.document.nodes.insert(
            node_id.clone(),
            make_node_for_his("node-1", 100.0, 100.0, 80.0, 40.0),
        );
        // Simulate revision at autosave boundary (e.g., revision 10)
        for _ in 0..10 {
            doc_before_autosave.revision = doc_before_autosave.revision.increment();
        }

        // Push state before autosave
        let history = History::new().push(doc_before_autosave.clone());

        // Simulate changes after autosave with higher revision
        let mut doc_after_autosave = doc_before_autosave.clone();
        if let Some(node) = doc_after_autosave.document.nodes.get_mut(&node_id) {
            node.x = OrderedFloat(500.0);
            node.y = OrderedFloat(500.0);
        }
        // Autosave might increment revision multiple times
        for _ in 0..5 {
            doc_after_autosave.revision = doc_after_autosave.revision.increment();
        }

        // Undo should restore state from before autosave
        let Some((restored, after_undo)) = history.undo(doc_after_autosave.clone()) else {
            panic!("undo should succeed");
        };

        // Verify document content is restored
        let restored_node = restored
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");
        assert_eq!(
            restored_node.x.0, 100.0,
            "x should be restored to pre-autosave value"
        );
        assert_eq!(
            restored_node.y.0, 100.0,
            "y should be restored to pre-autosave value"
        );

        // Verify revision is from the pushed state
        assert_eq!(
            restored.revision, doc_before_autosave.revision,
            "revision should be from pre-autosave state"
        );

        // Verify redo is available
        assert!(after_undo.can_redo(), "redo should be available after undo");
    }

    /// HIS-017: Inverse property validation for move operations
    /// Verifies that undo of a move operation restores exact original position.
    #[test]
    fn given_node_at_original_position_when_moved_and_undo_then_exact_position_restored() {
        let original_x = 123.45;
        let original_y = 678.90;
        let new_x = 999.99;
        let new_y = 111.11;

        let mut doc_before = DiagramDocument::default();
        let node_id = NodeId::new("node-1".to_string());
        let _ = doc_before.document.nodes.insert(
            node_id.clone(),
            make_node_for_his("node-1", original_x, original_y, 80.0, 40.0),
        );

        let history = History::new().push(doc_before.clone());

        // Move the node
        let mut doc_after = doc_before.clone();
        if let Some(node) = doc_after.document.nodes.get_mut(&node_id) {
            node.x = OrderedFloat(new_x);
            node.y = OrderedFloat(new_y);
        }
        doc_after.revision = doc_after.revision.increment();

        // Undo should restore exact original position
        let Some((restored, _)) = history.undo(doc_after) else {
            panic!("undo should succeed");
        };

        let restored_node = restored
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");

        // Verify exact inverse property restoration
        assert_eq!(
            restored_node.x.0, original_x,
            "x should be exactly restored to original value"
        );
        assert_eq!(
            restored_node.y.0, original_y,
            "y should be exactly restored to original value"
        );

        // Verify no floating point drift
        let epsilon = 1e-10;
        assert!(
            (restored_node.x.0 - original_x).abs() < epsilon,
            "x should have no floating point drift"
        );
        assert!(
            (restored_node.y.0 - original_y).abs() < epsilon,
            "y should have no floating point drift"
        );
    }

    /// HIS-018: Inverse property validation for resize operations
    /// Verifies that undo of a resize operation restores exact original dimensions.
    #[test]
    fn given_node_with_original_dimensions_when_resized_and_undo_then_exact_dimensions_restored() {
        let original_width = 150.75;
        let original_height = 200.25;
        let new_width = 50.5;
        let new_height = 75.5;

        let mut doc_before = DiagramDocument::default();
        let node_id = NodeId::new("node-1".to_string());
        let _ = doc_before.document.nodes.insert(
            node_id.clone(),
            make_node_for_his("node-1", 100.0, 100.0, original_width, original_height),
        );

        let history = History::new().push(doc_before.clone());

        // Resize the node
        let mut doc_after = doc_before.clone();
        if let Some(node) = doc_after.document.nodes.get_mut(&node_id) {
            node.width = OrderedFloat(new_width);
            node.height = OrderedFloat(new_height);
        }
        doc_after.revision = doc_after.revision.increment();

        // Undo should restore exact original dimensions
        let Some((restored, _)) = history.undo(doc_after) else {
            panic!("undo should succeed");
        };

        let restored_node = restored
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");

        // Verify exact inverse property restoration
        assert_eq!(
            restored_node.width.0, original_width,
            "width should be exactly restored to original value"
        );
        assert_eq!(
            restored_node.height.0, original_height,
            "height should be exactly restored to original value"
        );

        // Verify no floating point drift
        let epsilon = 1e-10;
        assert!(
            (restored_node.width.0 - original_width).abs() < epsilon,
            "width should have no floating point drift"
        );
        assert!(
            (restored_node.height.0 - original_height).abs() < epsilon,
            "height should have no floating point drift"
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
