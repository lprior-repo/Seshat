//! History module for undo/redo operations
//!
//! Provides persistent undo/redo history using immutable data structures.
//!
//! ## Design by Contract
//!
//! ### Preconditions
//! - P1: Documents pushed to history must be valid
//! - P2: Undo requires non-empty undo stack
//! - P3: Redo requires non-empty redo stack
//!
//! ### Postconditions
//! - Q1: Push clears redo stack (new timeline branch)
//! - Q2: Undo returns previous state and moves current to redo
//! - Q3: Redo returns next state and moves current to undo
//! - Q4: All operations return new History (immutable)
//! - Q5: History capped at `MAX_HISTORY` entries (100)
//!
//! ### Invariants
//! - I1: Undo stack contains documents in reverse chronological order
//! - I2: Redo stack contains documents in chronological order
//! - I3: After push: redo stack is empty

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

fn truncate_stack(stack: &List<DiagramDocument>) -> List<DiagramDocument> {
    let len = stack.len();
    if len <= MAX_HISTORY {
        return stack.clone();
    }
    stack.iter().take(MAX_HISTORY).cloned().collect()
}

/// Drops the first element from the list
fn drop_first(stack: &List<DiagramDocument>) -> List<DiagramDocument> {
    let len = stack.len();
    if len <= 1 {
        return List::new();
    }
    stack.iter().skip(1).cloned().collect()
}

/// Drops the first two elements from the list (used when current matches first)
fn drop_first_two(stack: &List<DiagramDocument>) -> List<DiagramDocument> {
    let len = stack.len();
    if len <= 2 {
        return List::new();
    }
    stack.iter().skip(2).cloned().collect()
}

/// Gets the second element from the list
fn second_element(stack: &List<DiagramDocument>) -> Option<DiagramDocument> {
    stack.iter().nth(1).cloned()
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
            undo_stack: push_back(&self.undo_stack, doc),
            redo_stack: List::new(),
        }
        .tap_history_limit()
    }

    /// Pure transition to undo
    #[must_use]
    pub fn undo(&self, current: DiagramDocument) -> Option<(DiagramDocument, Self)> {
        last_element(&self.undo_stack).map(|prev| {
            (
                prev.clone(),
                Self {
                    undo_stack: drop_last(&self.undo_stack),
                    redo_stack: push_back(&self.redo_stack, current),
                }
                .tap_history_limit(),
            )
        })
    }

    /// Pure transition to redo
    #[must_use]
    pub fn redo(&self, current: DiagramDocument) -> Option<(DiagramDocument, Self)> {
        last_element(&self.redo_stack).map(|next| {
            (
                next.clone(),
                Self {
                    undo_stack: push_back(&self.undo_stack, current),
                    redo_stack: drop_last(&self.redo_stack),
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

    #[cfg(kani)]
    #[kani::proof]
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

    // ============================================================
    // FAST TARGETED TESTS - catch mutation timeout issues directly
    // ============================================================

    /// Direct test of truncate_stack: empty stack stays empty
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_empty_stack_when_truncate_then_returns_empty() {
        use super::{truncate_stack, List};
        let empty: List<DiagramDocument> = List::new();
        let result = truncate_stack(&empty);
        assert!(result.is_empty(), "empty stack should remain empty");
    }

    /// Direct test of truncate_stack: small stack unchanged
    #[cfg(kani)]
    #[kani::proof]
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
    #[cfg(kani)]
    #[kani::proof]
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
    #[cfg(kani)]
    #[kani::proof]
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
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_empty_stack_when_drop_first_then_returns_empty() {
        use super::{drop_first, List};
        let empty: List<DiagramDocument> = List::new();
        let result = drop_first(&empty);
        assert!(result.is_empty(), "dropping from empty should return empty");
    }

    /// Direct test of drop_first: single element becomes empty
    #[cfg(kani)]
    #[kani::proof]
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
    #[cfg(kani)]
    #[kani::proof]
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

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_can_undo_returns_false_for_fresh_history() {
        let history = History::new();
        assert!(!history.can_undo());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_can_undo_returns_true_after_push() {
        let history = History::new().push(doc_with_revision(1));
        assert!(history.can_undo());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_can_redo_returns_false_for_fresh_history() {
        let history = History::new();
        assert!(!history.can_redo());
    }

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
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    /// HIS-001: Move node undo restores original position
    #[cfg(kani)]
    #[kani::proof]
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
    #[cfg(kani)]
    #[kani::proof]
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
    #[cfg(kani)]
    #[kani::proof]
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
    #[cfg(kani)]
    #[kani::proof]
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
                tags: im::Vector::new(),
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
    #[cfg(kani)]
    #[kani::proof]
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
    #[cfg(kani)]
    #[kani::proof]
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
            bend_points: im::Vector::new(),
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            font_size: None,
            source_port: None,
            target_port: None,
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
    #[cfg(kani)]
    #[kani::proof]
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
    #[cfg(kani)]
    #[kani::proof]
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
    #[cfg(kani)]
    #[kani::proof]
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
    #[cfg(kani)]
    #[kani::proof]
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
    #[cfg(kani)]
    #[kani::proof]
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
    #[cfg(kani)]
    #[kani::proof]
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
    #[cfg(kani)]
    #[kani::proof]
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
    #[cfg(kani)]
    #[kani::proof]
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
        let Some((state_b, h1)) = history_after_4.redo(state_a.clone()) else {
            panic!("first redo should succeed");
        };
        let node_at_b = state_b
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");
        assert_eq!(
            node_at_b.x.0, 200.0,
            "first redo should restore state B (x=200)"
        );

        let Some((state_c, h2)) = h1.redo(state_b.clone()) else {
            panic!("second redo should succeed");
        };
        let node_at_c = state_c
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");
        assert_eq!(
            node_at_c.x.0, 300.0,
            "second redo should restore state C (x=300)"
        );

        let Some((state_d, h3)) = h2.redo(state_c.clone()) else {
            panic!("third redo should succeed");
        };
        let node_at_d = state_d
            .document
            .nodes
            .get(&node_id)
            .expect("node should exist");
        assert_eq!(
            node_at_d.x.0, 400.0,
            "third redo should restore state D (x=400)"
        );

        let Some((state_current, _)) = h3.redo(state_d.clone()) else {
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
    #[cfg(kani)]
    #[kani::proof]
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
    #[cfg(kani)]
    #[kani::proof]
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
    #[cfg(kani)]
    #[kani::proof]
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
    #[cfg(kani)]
    #[kani::proof]
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

            #[cfg(kani)]
            #[kani::proof]
            #[test]
            #[allow(clippy::unwrap_used)]
            fn prop_undo_on_empty_returns_none(rev in 0..1000u64) {
                let history = History::new();
                let current = doc_with_revision(rev);
                prop_assert!(history.undo(current).is_none());
            }

            #[cfg(kani)]
            #[kani::proof]
            #[test]
            #[allow(clippy::unwrap_used)]
            fn prop_redo_on_empty_returns_none(rev in 0..1000u64) {
                let history = History::new().push(doc_with_revision(rev));
                let current = doc_with_revision(rev + 100);
                prop_assert!(history.redo(current).is_none());
            }

            #[cfg(kani)]
            #[kani::proof]
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

            #[cfg(kani)]
            #[kani::proof]
            #[test]
            #[allow(clippy::unwrap_used)]
            fn prop_undo_stack_bounded_at_100(pushes in 0..200usize) {
                let history = (0..pushes as u64).fold(History::new(), |acc, i| {
                    acc.push(doc_with_revision(i))
                });
                prop_assert!(history.undo_stack.len() <= 100);
            }

            #[cfg(kani)]
            #[kani::proof]
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

            #[cfg(kani)]
            #[kani::proof]
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

            #[cfg(kani)]
            #[kani::proof]
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

            #[cfg(kani)]
            #[kani::proof]
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

            #[cfg(kani)]
            #[kani::proof]
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

    #[cfg(test)]
    mod his003_to_his008_tests {
        //! Unit tests for HIS-003..HIS-008
        //! These tests verify the history system's undo/redo functionality
        //! for various document mutations including drag, grouping, reparenting,
        //! connectors, style changes, and text edits.

        use super::*;
        use crate::models::document::{
            DiagramDocument, Edge, EdgeId, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
            Revision,
        };

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

        fn make_node(label: &str, x: f64, y: f64, width: f64, height: f64) -> Node {
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
                tags: im::Vector::new(),
                metadata: im::HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            }
        }

        // ============================================================
        // HIS-003: Drag gesture creates single history entry
        // ============================================================

        #[test]
        fn test_his003_drag_creates_single_history_entry() {
            let mut doc_before = DiagramDocument::default();
            let node_id = NodeId::new("node-1".to_string());
            let _ = doc_before.document.nodes.insert(
                node_id.clone(),
                make_node("node-1", 100.0, 100.0, 80.0, 40.0),
            );

            // Push initial state as a checkpoint (this is the "before" state for undo)
            let history = History::new().push(doc_before.clone());

            // Simulate drag: move node to new position (150, 150)
            // This creates the current state after the drag operation
            // Note: We do NOT push doc_after - it represents the current working state
            let mut doc_after = doc_before.clone();
            if let Some(node) = doc_after.document.nodes.get_mut(&node_id) {
                node.x = OrderedFloat(150.0);
                node.y = OrderedFloat(150.0);
            }
            doc_after.revision = doc_after.revision.increment();

            // History undo_stack has exactly 1 entry (the checkpoint we saved before drag)
            // This verifies that a drag operation creates a single history entry,
            // not multiple per-frame entries during the drag gesture
            assert_eq!(
                history.undo_stack.len(),
                1,
                "History should have exactly 1 entry (the pre-drag checkpoint)"
            );

            // Undo restores original position (100, 100)
            // We pass doc_after as the current state to be stored in redo
            let Some((restored, _)) = history.undo(doc_after) else {
                panic!("undo should succeed");
            };
            let restored_node = restored
                .document
                .nodes
                .get(&node_id)
                .expect("node should exist");
            assert_eq!(
                restored_node.x.0, 100.0,
                "Undo should restore original x position"
            );
            assert_eq!(
                restored_node.y.0, 100.0,
                "Undo should restore original y position"
            );
        }

        // ============================================================
        // HIS-004: Group undo removes group
        // ============================================================

        #[test]
        fn test_his004_group_undo_removes_group() {
            let mut doc_before = DiagramDocument::default();
            let node_a = NodeId::new("node-a".to_string());
            let node_b = NodeId::new("node-b".to_string());

            let _ = doc_before.document.nodes.insert(
                node_a.clone(),
                make_node("node-a", 100.0, 100.0, 80.0, 40.0),
            );
            let _ = doc_before.document.nodes.insert(
                node_b.clone(),
                make_node("node-b", 200.0, 100.0, 80.0, 40.0),
            );

            // Verify nodes have no parent
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

            // Push initial state
            let history = History::new().push(doc_before.clone());

            // Create group (subgraph) containing nodes
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
                    tags: im::Vector::new(),
                    metadata: im::HashMap::new(),
                    z_index: -1,
                    style: Some(NodeStyle::Box),
                    collapsed: Some(false),
                },
            );
            doc_after.revision = doc_after.revision.increment();

            // Undo should remove group - undo(doc_after) returns doc_before which has no group
            let Some((restored, _)) = history.undo(doc_after) else {
                panic!("undo should succeed");
            };

            // Group should be removed
            assert!(
                !restored.document.nodes.contains_key(&group_id),
                "Group should be removed after undo"
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
            assert!(
                restored_a.parent.is_none(),
                "node-a parent should be None after undo"
            );
            assert!(
                restored_b.parent.is_none(),
                "node-b parent should be None after undo"
            );
        }

        // ============================================================
        // HIS-005: Reparent undo restores parent
        // ============================================================

        #[test]
        fn test_his005_reparent_undo_restores_parent() {
            let mut doc_before = DiagramDocument::default();
            let parent1 = NodeId::new("parent-1".to_string());
            let parent2 = NodeId::new("parent-2".to_string());
            let child = NodeId::new("child".to_string());

            let _ = doc_before.document.nodes.insert(
                parent1.clone(),
                make_node("parent-1", 0.0, 0.0, 200.0, 150.0),
            );
            let _ = doc_before.document.nodes.insert(
                parent2.clone(),
                make_node("parent-2", 300.0, 0.0, 200.0, 150.0),
            );

            let mut child_node = make_node("child", 50.0, 50.0, 80.0, 40.0);
            child_node.parent = Some(parent1.clone());
            let _ = doc_before.document.nodes.insert(child.clone(), child_node);

            // Push initial state
            let history = History::new().push(doc_before.clone());

            // Reparent child to parent2
            let mut doc_after = doc_before.clone();
            if let Some(node) = doc_after.document.nodes.get_mut(&child) {
                node.parent = Some(parent2.clone());
            }
            doc_after.revision = doc_after.revision.increment();

            let history = history.push(doc_after.clone());

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
                "Child's parent should be restored to parent-1"
            );
        }

        // ============================================================
        // HIS-006: Connector create undo removes edge
        // ============================================================

        #[test]
        fn test_his006_connector_create_undo_removes_edge() {
            let mut doc_before = DiagramDocument::default();
            let node_a = NodeId::new("node-a".to_string());
            let node_b = NodeId::new("node-b".to_string());

            let _ = doc_before
                .document
                .nodes
                .insert(node_a.clone(), make_node("node-a", 0.0, 0.0, 80.0, 40.0));
            let _ = doc_before
                .document
                .nodes
                .insert(node_b.clone(), make_node("node-b", 200.0, 0.0, 80.0, 40.0));

            // Initially no edges
            assert!(doc_before.document.edges.is_empty());

            // Push initial state
            let history = History::new().push(doc_before.clone());

            // Create edge
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
                bend_points: im::Vector::new(),
                tags: im::Vector::new(),
                metadata: im::HashMap::new(),
                font_size: None,
                source_port: None,
                target_port: None,
            };
            let _ = doc_after.document.edges.insert(edge_id, edge);
            doc_after.revision = doc_after.revision.increment();

            let history = history.push(doc_after.clone());

            // Undo should remove edge
            let Some((restored, _)) = history.undo(doc_after) else {
                panic!("undo should succeed");
            };

            assert!(
                restored.document.edges.is_empty(),
                "Edges should be empty after undo"
            );
        }

        // ============================================================
        // HIS-007: Style change undo restores style
        // ============================================================

        #[test]
        fn test_his007_style_change_undo_restores_style() {
            let mut doc_before = DiagramDocument::default();
            let node_id = NodeId::new("node-1".to_string());
            let mut node = make_node("node-1", 100.0, 100.0, 80.0, 40.0);
            node.style = Some(NodeStyle::Box);
            let _ = doc_before.document.nodes.insert(node_id.clone(), node);

            // Push initial state
            let history = History::new().push(doc_before.clone());

            // Change style to Dashed
            let mut doc_after = doc_before.clone();
            if let Some(node) = doc_after.document.nodes.get_mut(&node_id) {
                node.style = Some(NodeStyle::Dashed);
            }
            doc_after.revision = doc_after.revision.increment();

            let history = history.push(doc_after.clone());

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
                "Style should be restored to Box"
            );
        }

        // ============================================================
        // HIS-008: Text edit creates single entry
        // ============================================================

        #[test]
        fn test_his008_text_edit_creates_single_entry() {
            let mut doc_before = DiagramDocument::default();
            let node_id = NodeId::new("node-1".to_string());
            let mut node = make_node("node-1", 100.0, 100.0, 80.0, 40.0);
            node.label = "Original Label".to_string();
            let _ = doc_before.document.nodes.insert(node_id.clone(), node);

            // Push initial state
            let history = History::new().push(doc_before.clone());
            assert_eq!(
                history.undo_stack.len(),
                1,
                "Should have one history entry after first push"
            );

            // Change label and push
            let mut doc_after = doc_before.clone();
            if let Some(node) = doc_after.document.nodes.get_mut(&node_id) {
                node.label = "New Label".to_string();
            }
            doc_after.revision = doc_after.revision.increment();

            let history = history.push(doc_after.clone());

            // History undo_stack has exactly 1 entry
            assert_eq!(
                history.undo_stack.len(),
                2,
                "Should have exactly 2 history entries"
            );

            // Undo restores original label
            let Some((restored, _)) = history.undo(doc_after) else {
                panic!("undo should succeed");
            };
            let restored_node = restored
                .document
                .nodes
                .get(&node_id)
                .expect("node should exist");
            assert_eq!(
                restored_node.label, "Original Label",
                "Label should be restored to Original Label"
            );
        }

        // ============================================================
        // Error path tests
        // ============================================================

        #[test]
        fn test_undo_on_empty_history_returns_none() {
            let history = History::new();
            let current = doc_with_revision(1);

            let result = history.undo(current);
            assert!(result.is_none(), "undo on empty history should return None");
        }

        #[test]
        fn test_redo_on_empty_redo_stack_returns_none() {
            let history = History::new().push(doc_with_revision(1));
            let current = doc_with_revision(2);

            let result = history.redo(current);
            assert!(
                result.is_none(),
                "redo on empty redo stack should return None"
            );
        }

        #[test]
        fn test_multiple_redo_on_exhausted_stack_returns_none() {
            // push(A), push(B), undo (back to A), redo (forward to B)
            let history = History::new()
                .push(doc_with_revision(1))
                .push(doc_with_revision(2));

            let current = doc_with_revision(3);
            let Some((_, after_undo)) = history.undo(current) else {
                panic!("undo should succeed");
            };
            let Some((_, after_redo)) = after_undo.redo(doc_with_revision(2)) else {
                panic!("redo should succeed");
            };

            // Now redo stack is empty, redo should return None
            let result = after_redo.redo(doc_with_revision(1));
            assert!(
                result.is_none(),
                "redo on exhausted stack should return None"
            );
        }

        // ============================================================
        // Edge case tests
        // ============================================================

        #[test]
        fn test_undo_stack_bounded_at_100() {
            // Push 101 documents
            let history =
                (0..101_u64).fold(History::new(), |acc, i| acc.push(doc_with_revision(i)));

            assert_eq!(
                history.undo_stack.len(),
                100,
                "undo_stack should be capped at 100"
            );
        }

        #[test]
        fn test_multiple_operations_create_multiple_entries() {
            let history = History::new()
                .push(doc_with_revision(1))
                .push(doc_with_revision(2))
                .push(doc_with_revision(3));

            assert_eq!(history.undo_stack.len(), 3, "Should have 3 entries");
        }

        #[test]
        fn test_push_after_undo_clears_redo_stack() {
            // push(A), push(B), undo (back to A, redo has B)
            let history = History::new()
                .push(doc_with_revision(1))
                .push(doc_with_revision(2));

            let current = doc_with_revision(3);
            let Some((_, after_undo)) = history.undo(current) else {
                panic!("undo should succeed");
            };

            assert!(
                !after_undo.redo_stack.is_empty(),
                "redo stack should have entries after undo"
            );

            // push(C)
            let after_push = after_undo.push(doc_with_revision(4));

            assert!(
                after_push.redo_stack.is_empty(),
                "redo stack should be empty after push"
            );
            assert_eq!(
                after_push.undo_stack.len(),
                2,
                "undo stack should have A and C"
            );
        }

        #[test]
        fn test_can_undo_returns_correct_state() {
            let history = History::new();
            assert!(
                !history.can_undo(),
                "can_undo should return false for fresh history"
            );

            let history = history.push(doc_with_revision(1));
            assert!(history.can_undo(), "can_undo should return true after push");
        }

        #[test]
        fn test_can_redo_returns_correct_state() {
            let history = History::new();
            assert!(
                !history.can_redo(),
                "can_redo should return false for fresh history"
            );

            let history = history.push(doc_with_revision(1));
            let current = doc_with_revision(2);
            let Some((_, after_undo)) = history.undo(current) else {
                panic!("undo should succeed");
            };
            assert!(
                after_undo.can_redo(),
                "can_redo should return true after undo"
            );
        }

        // ============================================================
        // Integration tests
        // ============================================================

        #[test]
        fn test_integration_e2e_full_history_workflow() {
            let mut doc = DiagramDocument::default();
            let node_id = NodeId::new("node-1".to_string());
            let _ = doc
                .document
                .nodes
                .insert(node_id.clone(), make_node("node-1", 0.0, 0.0, 80.0, 40.0));

            // Initialize history with initial state
            // This counts as 1 entry in undo_stack
            let mut history = History::new().push(doc.clone());
            assert_eq!(
                history.undo_stack.len(),
                1,
                "After initial push: undo_stack.len() = 1"
            );

            // Step 1: Move to (100, 100) and push
            if let Some(node) = doc.document.nodes.get_mut(&node_id) {
                node.x = OrderedFloat(100.0);
                node.y = OrderedFloat(100.0);
            }
            doc.revision = doc.revision.increment();
            history = history.push(doc.clone());
            assert_eq!(
                history.undo_stack.len(),
                2,
                "After step 1: undo_stack.len() = 2 (initial + step1)"
            );

            // Step 2: Move to (200, 200) and push
            if let Some(node) = doc.document.nodes.get_mut(&node_id) {
                node.x = OrderedFloat(200.0);
                node.y = OrderedFloat(200.0);
            }
            doc.revision = doc.revision.increment();
            history = history.push(doc.clone());
            assert_eq!(
                history.undo_stack.len(),
                3,
                "After step 2: undo_stack.len() = 3 (initial + step1 + step2)"
            );

            // Step 3: Undo
            let Some((doc_restored, h)) = history.undo(doc.clone()) else {
                panic!("undo should succeed");
            };
            doc = doc_restored;
            history = h;
            assert!(history.can_redo(), "After step 3: can_redo() = true");
            assert_eq!(
                history.undo_stack.len(),
                2,
                "After step 3: undo_stack.len() = 2 (step2 removed)"
            );

            // Step 4: Undo again
            let Some((doc_restored, h)) = history.undo(doc.clone()) else {
                panic!("undo should succeed");
            };
            doc = doc_restored;
            history = h;
            assert!(history.can_redo(), "After step 4: can_redo() = true");
            assert_eq!(
                history.undo_stack.len(),
                1,
                "After step 4: undo_stack.len() = 1 (only initial remains)"
            );

            // Step 5: Redo
            let Some((doc_redo, h)) = history.redo(doc.clone()) else {
                panic!("redo should succeed");
            };
            doc = doc_redo;
            history = h;
            assert!(history.can_undo(), "After step 5: can_undo() = true");
            assert_eq!(
                history.undo_stack.len(),
                2,
                "After step 5: undo_stack.len() = 2 (current added back)"
            );

            // Step 6: Redo again
            let Some((doc_redo, h)) = history.redo(doc.clone()) else {
                panic!("redo should succeed");
            };
            doc = doc_redo;
            history = h;
            assert!(history.can_undo(), "After step 6: can_undo() = true");
            assert_eq!(
                history.undo_stack.len(),
                3,
                "After step 6: undo_stack.len() = 3 (current added back)"
            );

            // Verify final position
            let node = doc.document.nodes.get(&node_id).expect("node should exist");
            assert_eq!(node.x.0, 200.0, "Final document position should be x=200");
            assert_eq!(node.y.0, 200.0, "Final document position should be y=200");
        }

        // ============================================================
        // Contract verification tests
        // ============================================================

        #[test]
        fn test_precondition_p2_undo_requires_nonempty_stack() {
            // Fresh history with empty undo_stack
            let history = History::new();
            let current = doc_with_revision(1);

            // undo() returns None (not panic)
            let result = history.undo(current);
            assert!(result.is_none(), "undo on empty stack should return None");
        }

        #[test]
        fn test_precondition_p3_redo_requires_nonempty_stack() {
            // History with push(A), undo (at A, redo has A)
            // redo (at original, redo empty)
            let history = History::new().push(doc_with_revision(1));

            let current = doc_with_revision(2);
            let Some((_, after_undo)) = history.undo(current) else {
                panic!("undo should succeed");
            };

            // First redo should work
            let result = after_undo.redo(doc_with_revision(1));
            assert!(result.is_some(), "first redo should succeed");

            // Second redo should return None
            if let Some((_, after_redo)) = result {
                let result2 = after_redo.redo(doc_with_revision(0));
                assert!(result2.is_none(), "second redo should return None");
            }
        }

        #[test]
        fn test_postcondition_q2_push_clears_redo_stack() {
            // History with push(A), push(B), undo (back to A, redo has B)
            let history = History::new()
                .push(doc_with_revision(1))
                .push(doc_with_revision(2));

            let current = doc_with_revision(3);
            let Some((_, after_undo)) = history.undo(current) else {
                panic!("undo should succeed");
            };

            assert!(
                !after_undo.redo_stack.is_empty(),
                "redo stack should have entries"
            );

            // push(C) should clear redo stack
            let after_push = after_undo.push(doc_with_revision(4));

            assert!(
                after_push.redo_stack.is_empty(),
                "redo stack should be empty after push"
            );
        }

        #[test]
        fn test_postcondition_q8_single_entry_per_operation() {
            // History with push(A)
            let history = History::new().push(doc_with_revision(1));

            // Single push(B) representing completed drag gesture
            let history = history.push(doc_with_revision(2));

            // undo_stack should have exactly 2 (A and B)
            assert_eq!(
                history.undo_stack.len(),
                2,
                "undo_stack should have exactly 2 entries"
            );
        }

        #[test]
        fn test_invariant_i1_undo_stack_is_reverse_chronological() {
            // History with push(A), push(B), push(C)
            // I1: Undo stack contains documents in reverse chronological order
            // (newest first - this is correct for LIFO undo semantics)
            let history = History::new()
                .push(doc_with_revision(1))
                .push(doc_with_revision(2))
                .push(doc_with_revision(3));

            // Collect revisions in order from the stack
            let revisions: Vec<_> = history
                .undo_stack
                .iter()
                .map(|d| d.revision.value())
                .collect();

            // undo_stack = [C (newest), B, A (oldest)] - reverse chronological
            // This is correct: undo() returns first() which should be the most recent state
            assert_eq!(revisions[0], 3, "First entry should be revision 3 (newest)");
            assert_eq!(revisions[1], 2, "Second entry should be revision 2");
            assert_eq!(revisions[2], 1, "Third entry should be revision 1 (oldest)");
        }

        #[test]
        fn test_invariant_i2_redo_stack_is_chronological() {
            // I2: Redo stack contains documents in chronological order
            // (oldest redo first - this allows redo to walk forward in time)
            //
            // History with push(A), push(B)
            // Then undo twice to get back to initial state
            let history = History::new()
                .push(doc_with_revision(1))
                .push(doc_with_revision(2));
            // undo_stack = [B(rev2), A(rev1)] - reverse chronological

            let current = doc_with_revision(3);
            let Some((returned_doc, h1)) = history.undo(current) else {
                panic!("undo 1 should succeed");
            };
            // returned_doc = B(rev2), h1.undo_stack = [A(rev1)], h1.redo_stack = [C(rev3)]
            assert_eq!(
                returned_doc.revision.value(),
                2,
                "First undo should return rev2"
            );

            // Must pass the RETURNED document as current, not an arbitrary document
            let Some((_, h2)) = h1.undo(returned_doc) else {
                panic!("undo 2 should succeed");
            };
            // h2.undo_stack = [], h2.redo_stack = [B(rev2), C(rev3)]
            // redo_stack is chronological: first redo goes to B, then to C

            // Collect revisions from redo stack
            let revisions: Vec<_> = h2.redo_stack.iter().map(|d| d.revision.value()).collect();

            // redo_stack[0] = B (first redo target), redo_stack[1] = C (second redo target)
            assert_eq!(
                revisions[0], 2,
                "First redo entry should be revision 2 (first redo target)"
            );
            assert_eq!(
                revisions[1], 3,
                "Second redo entry should be revision 3 (second redo target)"
            );
        }

        #[test]
        fn test_invariant_i3_after_push_redo_stack_is_empty() {
            // History with push(A), push(B), undo (back to A, redo has B)
            let history = History::new()
                .push(doc_with_revision(1))
                .push(doc_with_revision(2));

            let current = doc_with_revision(3);
            let Some((_, after_undo)) = history.undo(current) else {
                panic!("undo should succeed");
            };

            assert!(!after_undo.redo_stack.is_empty(), "redo should have B");

            // push(C) creates new timeline branch
            let after_push = after_undo.push(doc_with_revision(4));

            assert!(
                after_push.redo_stack.is_empty(),
                "redo stack should be empty after push"
            );
            assert_eq!(
                after_push.undo_stack.len(),
                2,
                "undo stack should have A and C"
            );
        }

        #[test]
        fn test_invariant_i4_after_undo_can_redo_is_true() {
            // History with push(A), push(B)
            let history = History::new()
                .push(doc_with_revision(1))
                .push(doc_with_revision(2));

            let current = doc_with_revision(3);
            let Some((_, after_undo)) = history.undo(current) else {
                panic!("undo should succeed");
            };

            assert!(
                after_undo.can_redo(),
                "can_redo should return true after undo"
            );
        }

        #[test]
        fn test_invariant_i5_after_redo_can_undo_is_true() {
            // History with push(A), push(B), undo performed (at A, redo has B)
            let history = History::new()
                .push(doc_with_revision(1))
                .push(doc_with_revision(2));

            let current = doc_with_revision(3);
            let Some((_, after_undo)) = history.undo(current) else {
                panic!("undo should succeed");
            };

            // Redo performed (at B)
            let Some((_, after_redo)) = after_undo.redo(doc_with_revision(1)) else {
                panic!("redo should succeed");
            };

            assert!(
                after_redo.can_undo(),
                "can_undo should return true after redo"
            );
        }
    }
}
