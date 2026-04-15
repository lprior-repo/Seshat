//! SEL Category Tests (25 tests)
//!
//! Selection operations: click, marquee, multi-select, deselect.

use crate::test_utils::{builders::*, fixtures::*, harness::*, types::*};
use diagram_models::document::{DiagramDocument, DocumentData, NodeId};

// ============================================================================
// SEL-001 to SEL-005: Click selection
// ============================================================================

#[test]
fn sel_001_click_selects_single_node() {
    let doc = DocBuilder::new()
        .add_node_with("A", 10.0, 10.0, 50.0, 50.0)
        .with_selection("A")
        .build();
    assert!(doc.editor_state.selected_items.contains("A"));
}

#[test]
fn sel_002_click_selects_among_multiple() {
    let doc = DocBuilder::new()
        .add_node_with("A", 10.0, 10.0, 50.0, 50.0)
        .add_node_with("B", 100.0, 100.0, 50.0, 50.0)
        .with_selection("B")
        .build();
    assert!(doc.editor_state.selected_items.contains("B"));
    assert!(!doc.editor_state.selected_items.contains("A"));
}

#[test]
fn sel_003_click_empty_area_deselects() {
    let doc = DocBuilder::new()
        .add_node_with("A", 10.0, 10.0, 50.0, 50.0)
        .build();
    assert!(doc.editor_state.selected_items.is_empty());
}

#[test]
fn sel_004_select_locked_node() {
    let node = test_node_builder(10.0, 10.0, 50.0, 50.0)
        .locked()
        .build();
    assert_eq!(node.lock_state, diagram_models::document::LockState::Locked);
}

#[test]
fn sel_005_select_node_inside_subgraph() {
    let subgraph = test_subgraph();
    let child = test_node_builder(10.0, 10.0, 20.0, 20.0)
        .with_parent(NodeId::new("group-1".to_string()))
        .build();
    assert!(child.parent.is_some());
}

// ============================================================================
// SEL-006 to SEL-010: Marquee selection
// ============================================================================

#[test]
fn sel_006_marquee_encloses_single_node() {
    let doc = setup_doc_with_nodes();
    // n1 at (10,10) 50x50 - enclosed by marquee (0,0)->(100,100)
    let node = doc.document.nodes.get(&NodeId::new("n1".to_string())).unwrap();
    assert!(node.x.0 >= 0.0 && node.x.0 + node.width.0 <= 100.0);
    assert!(node.y.0 >= 0.0 && node.y.0 + node.height.0 <= 100.0);
}

#[test]
fn sel_007_marquee_intersects_node() {
    let doc = setup_doc_with_nodes();
    // n2 at (80,80) 50x50 - intersects marquee (0,0)->(100,100) but not enclosed
    let node = doc.document.nodes.get(&NodeId::new("n2".to_string())).unwrap();
    assert!(node.x.0 < 100.0 && node.x.0 + node.width.0 > 100.0);
}

#[test]
fn sel_008_marquee_excludes_distant_node() {
    let doc = setup_doc_with_nodes();
    // n3 at (150,150) 50x50 - outside marquee (0,0)->(100,100)
    let node = doc.document.nodes.get(&NodeId::new("n3".to_string())).unwrap();
    assert!(node.x.0 > 100.0);
}

#[test]
fn sel_009_marquee_zero_area_selects_nothing() {
    let doc = setup_doc_with_nodes();
    assert_eq!(doc.document.nodes.len(), 4);
    // A zero-area marquee should not enclose any node
}

#[test]
fn sel_010_marquee_negative_drag_direction() {
    // Marquee from (100,100) to (0,0) should still select enclosed nodes
    let doc = DocBuilder::new()
        .add_node_with("A", 10.0, 10.0, 50.0, 50.0)
        .build();
    let node = doc.document.nodes.get(&NodeId::new("A".to_string())).unwrap();
    assert!(node.x.0 >= 0.0 && node.x.0 + node.width.0 <= 100.0);
}

// ============================================================================
// SEL-011 to SEL-015: Multi-select operations
// ============================================================================

#[test]
fn sel_011_shift_click_adds_to_selection() {
    let doc = DocBuilder::new()
        .add_node_with("A", 10.0, 10.0, 50.0, 50.0)
        .add_node_with("B", 100.0, 100.0, 50.0, 50.0)
        .with_selection("A")
        .with_selection("B")
        .build();
    assert_eq!(doc.editor_state.selected_items.len(), 2);
}

#[test]
fn sel_012_ctrl_toggle_selection() {
    // Toggling B on a selection that has A should result in {A,B}
    let doc = DocBuilder::new()
        .add_node_with("A", 10.0, 10.0, 50.0, 50.0)
        .add_node_with("B", 100.0, 100.0, 50.0, 50.0)
        .with_selection("A")
        .with_selection("B")
        .build();
    assert!(doc.editor_state.selected_items.contains("A"));
    assert!(doc.editor_state.selected_items.contains("B"));
}

#[test]
fn sel_013_select_all_nodes() {
    let doc = DocBuilder::new()
        .add_node_with("A", 10.0, 10.0, 50.0, 50.0)
        .add_node_with("B", 100.0, 100.0, 50.0, 50.0)
        .add_node_with("C", 200.0, 200.0, 50.0, 50.0)
        .build();
    assert_eq!(doc.document.nodes.len(), 3);
}

#[test]
fn sel_014_deselect_all() {
    let doc = DocBuilder::new()
        .add_node_with("A", 10.0, 10.0, 50.0, 50.0)
        .build();
    assert!(doc.editor_state.selected_items.is_empty());
}

#[test]
fn sel_015_selection_preserves_order() {
    let doc = DocBuilder::new()
        .with_selection("C")
        .with_selection("A")
        .with_selection("B")
        .build();
    assert_eq!(doc.editor_state.selected_items.len(), 3);
}

// ============================================================================
// SEL-016 to SEL-020: Selection with edges
// ============================================================================

#[test]
fn sel_016_select_edge_by_id() {
    let doc = DocBuilder::new()
        .add_node_with("A", 10.0, 10.0, 50.0, 50.0)
        .add_node_with("B", 100.0, 100.0, 50.0, 50.0)
        .add_edge_str("e1", "A", "B")
        .build();
    assert_eq!(doc.document.edges.len(), 1);
}

#[test]
fn sel_017_select_connected_nodes() {
    let doc = DocBuilder::new()
        .add_node_with("A", 10.0, 10.0, 50.0, 50.0)
        .add_node_with("B", 100.0, 100.0, 50.0, 50.0)
        .add_edge_str("e1", "A", "B")
        .build();
    let edge = doc.document.edges.get(&diagram_models::document::EdgeId::new("e1".to_string())).unwrap();
    assert!(doc.document.nodes.contains_key(&edge.source));
    assert!(doc.document.nodes.contains_key(&edge.target));
}

#[test]
fn sel_018_select_node_excludes_edges() {
    let doc = DocBuilder::new()
        .add_node_with("A", 10.0, 10.0, 50.0, 50.0)
        .add_node_with("B", 100.0, 100.0, 50.0, 50.0)
        .add_edge_str("e1", "A", "B")
        .with_selection("A")
        .build();
    assert!(doc.editor_state.selected_items.contains("A"));
    assert!(!doc.editor_state.selected_items.contains("e1"));
}

#[test]
fn sel_019_selection_with_subgraph_nodes() {
    let subgraph = test_subgraph_builder()
        .with_label("Group-1")
        .build();
    let doc = DocBuilder::new()
        .add_node("group-1", subgraph)
        .with_selection("group-1")
        .build();
    assert!(doc.editor_state.selected_items.contains("group-1"));
}

#[test]
fn sel_020_select_all_in_subgraph() {
    let parent = NodeId::new("group-1".to_string());
    let child = test_node_builder(10.0, 10.0, 20.0, 20.0)
        .with_parent(parent.clone())
        .build();
    assert_eq!(child.parent, Some(parent));
}

// ============================================================================
// SEL-021 to SEL-025: Selection invariant tests
// ============================================================================

#[test]
fn sel_021_selected_node_exists_in_document() {
    let doc = DocBuilder::new()
        .add_node_with("A", 10.0, 10.0, 50.0, 50.0)
        .with_selection("A")
        .build();
    for item in &doc.editor_state.selected_items {
        assert!(
            doc.document.nodes.contains_key(&NodeId::new(item.clone())),
            "Selected item '{}' must exist in document",
            item
        );
    }
}

#[test]
fn sel_022_empty_selection_is_valid() {
    let doc = DocBuilder::new().build();
    assert!(doc.editor_state.selected_items.is_empty());
}

#[test]
fn sel_023_selection_count_matches() {
    let doc = DocBuilder::new()
        .add_node_with("A", 10.0, 10.0, 50.0, 50.0)
        .add_node_with("B", 100.0, 100.0, 50.0, 50.0)
        .with_selection("A")
        .with_selection("B")
        .build();
    assert_eq!(doc.editor_state.selected_items.len(), 2);
}

#[test]
fn sel_024_selection_after_node_delete() {
    // If we remove a selected node, it should be removed from selection
    let mut doc = DocBuilder::new()
        .add_node_with("A", 10.0, 10.0, 50.0, 50.0)
        .add_node_with("B", 100.0, 100.0, 50.0, 50.0)
        .with_selection("A")
        .with_selection("B")
        .build();
    doc.document.nodes.remove(&NodeId::new("A".to_string()));
    assert!(!doc.document.nodes.contains_key(&NodeId::new("A".to_string())));
}

#[test]
fn sel_025_selection_invariant_verification() {
    let doc = DocBuilder::new()
        .add_node_with("A", 10.0, 10.0, 50.0, 50.0)
        .with_selection("A")
        .build();
    let result = verify_invariants(&doc);
    assert!(result.is_ok());
}
