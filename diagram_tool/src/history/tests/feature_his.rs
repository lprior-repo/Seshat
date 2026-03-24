//! HIS Feature tests for history module
//!
//! High-level integration tests matching the HIS test specification.

#[cfg(kani)]
use crate::history::History;
#[cfg(kani)]
use diagram_models::document::{
    DiagramDocument, LockState, Node, NodeId, NodeKind, OrderedFloat, Revision,
};

#[cfg(kani)]
struct HistoryDsl {
    history: History,
    doc: DiagramDocument,
    node_id: NodeId,
}

#[cfg(kani)]
impl HistoryDsl {
    fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        let mut doc = DiagramDocument::default();
        doc.revision = Revision::INITIAL;
        let node_id = NodeId::new("node-1".to_string());
        doc.document.nodes.insert(
            node_id.clone(),
            Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: "node-1".to_string(),
                x: OrderedFloat(x),
                y: OrderedFloat(y),
                width: OrderedFloat(w),
                height: OrderedFloat(h),
                font_size: None,
                font_weight: None,
                lock_state: LockState::Unlocked,
                parent: None,
                dag_rank: None,
                tags: im::Vector::new(),
                metadata: im::HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        );
        Self {
            history: History::new().push(doc.clone()),
            doc,
            node_id,
        }
    }

    fn set_pos(&mut self, x: f64, y: f64) {
        if let Some(n) = self.doc.document.nodes.get_mut(&self.node_id) {
            n.x = OrderedFloat(x);
            n.y = OrderedFloat(y);
        }
        self.doc.revision = self.doc.revision.increment();
    }

    fn set_size(&mut self, w: f64, h: f64) {
        if let Some(n) = self.doc.document.nodes.get_mut(&self.node_id) {
            n.width = OrderedFloat(w);
            n.height = OrderedFloat(h);
        }
        self.doc.revision = self.doc.revision.increment();
    }

    fn push(&mut self) {
        self.history = self.history.push(self.doc.clone());
    }

    fn undo(&mut self) -> bool {
        if let Some((restored, new_history)) = self.history.undo(self.doc.clone()) {
            self.history = new_history;
            self.doc = restored;
            true
        } else {
            false
        }
    }

    fn redo(&mut self) -> bool {
        if let Some((restored, new_history)) = self.history.redo(self.doc.clone()) {
            self.history = new_history;
            self.doc = restored;
            true
        } else {
            false
        }
    }

    fn node(&self) -> Node {
        self.doc.document.nodes.get(&self.node_id).unwrap().clone()
    }
}

/// HIS-001: Move node undo restores original position
#[cfg(kani)]
#[kani::proof]
fn given_node_at_position_when_moved_and_undo_then_position_restored() {
    let mut dsl = HistoryDsl::new(100.0, 100.0, 80.0, 40.0);
    dsl.set_pos(200.0, 200.0);
    assert!(dsl.undo(), "undo should succeed");

    let n = dsl.node();
    assert_eq!(n.x.0, 100.0, "x should be restored");
    assert_eq!(n.y.0, 100.0, "y should be restored");
}

/// HIS-002: Resize undo restores exact original dimensions
#[cfg(kani)]
#[kani::proof]
fn given_node_with_dimensions_when_resized_and_undo_then_dimensions_restored() {
    let mut dsl = HistoryDsl::new(100.0, 100.0, 80.0, 40.0);
    dsl.set_size(160.0, 80.0);
    assert!(dsl.undo(), "undo should succeed");

    let n = dsl.node();
    assert_eq!(n.width.0, 80.0, "width should be restored");
    assert_eq!(n.height.0, 40.0, "height should be restored");
}

/// HIS-011: Push after undo clears redo stack
#[cfg(kani)]
#[kani::proof]
fn given_history_with_redo_entries_when_push_then_redo_stack_cleared() {
    let mut dsl = HistoryDsl::new(100.0, 100.0, 80.0, 40.0);

    dsl.set_pos(200.0, 100.0);
    dsl.push();

    dsl.set_pos(300.0, 100.0);
    dsl.push();

    dsl.set_pos(400.0, 100.0);

    assert!(dsl.undo(), "undo should succeed");
    assert!(
        dsl.history.can_redo(),
        "redo stack should have entries after undo"
    );

    dsl.set_pos(500.0, 100.0);
    dsl.push();

    assert!(
        !dsl.history.can_redo(),
        "redo stack should be empty after push"
    );
}

/// HIS-012: Multiple undos walk back through history correctly
#[cfg(kani)]
#[kani::proof]
fn given_history_with_multiple_states_when_undo_multiple_times_then_walks_back_correctly() {
    let mut dsl = HistoryDsl::new(100.0, 100.0, 80.0, 40.0);

    dsl.set_pos(200.0, 100.0);
    dsl.push();

    dsl.set_pos(300.0, 100.0);
    dsl.push();

    dsl.set_pos(400.0, 100.0);

    assert!(dsl.undo(), "first undo should succeed");
    assert_eq!(dsl.node().x.0, 300.0, "first undo should restore x=300");

    assert!(dsl.undo(), "second undo should succeed");
    assert_eq!(dsl.node().x.0, 200.0, "second undo should restore x=200");

    assert!(dsl.undo(), "third undo should succeed");
    assert_eq!(dsl.node().x.0, 100.0, "third undo should restore x=100");
}

/// HIS-013: Redo after multiple undos works correctly
#[cfg(kani)]
#[kani::proof]
fn given_history_after_multiple_undos_when_redo_then_walks_forward_correctly() {
    let mut dsl = HistoryDsl::new(100.0, 100.0, 80.0, 40.0);

    dsl.set_pos(200.0, 100.0);
    dsl.push();

    dsl.set_pos(300.0, 100.0);
    dsl.push();

    dsl.set_pos(400.0, 100.0);

    assert!(dsl.undo()); // -> 300
    assert!(dsl.undo()); // -> 200
    assert_eq!(dsl.node().x.0, 200.0, "should be at state x=200");

    assert!(dsl.redo(), "first redo should succeed");
    assert_eq!(dsl.node().x.0, 300.0, "first redo should restore x=300");

    assert!(dsl.redo(), "second redo should succeed");
    assert_eq!(dsl.node().x.0, 400.0, "second redo should restore x=400");
}
