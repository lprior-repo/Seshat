#![allow(clippy::unwrap_used, clippy::panic, clippy::module_inception, clippy::let_unit_value, clippy::redundant_pattern_matching, unused_variables, unused_imports)]
use im::HashMap;

use super::{apply_rubber_band_release, fit_icon_side, subgraph_release_bounds};
use crate::ui::grid::GridSize;
use diagram_models::document::{
    DiagramDocument, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
};

fn node_at(x: f64, y: f64) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: String::from("N"),
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        width: OrderedFloat(50.0),
        height: OrderedFloat(50.0),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent: None,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        z_index: 0,
        style: Some(NodeStyle::default()),
        collapsed: None,
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_rubber_band_release_when_applied_then_selection_is_committed() {
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new(String::from("n1"));
    doc.document.nodes = doc
        .document
        .nodes
        .update(node_id.clone(), node_at(10.0, 10.0));

    apply_rubber_band_release(&mut doc, (0.0, 0.0), (80.0, 80.0), false);

    assert!(doc
        .editor_state
        .selected_items
        .contains(&node_id.to_string()));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_noop_rubber_band_when_released_then_selection_is_preserved() {
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new(String::from("n1"));
    doc.document.nodes = doc
        .document
        .nodes
        .update(node_id.clone(), node_at(10.0, 10.0));
    doc.editor_state.selected_items = doc.editor_state.selected_items.update(node_id.to_string());

    apply_rubber_band_release(&mut doc, (10.0, 10.0), (10.0, 10.0), false);

    assert!(doc
        .editor_state
        .selected_items
        .contains(&node_id.to_string()));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_existing_selection_when_rubber_band_released_then_selection_is_cleared() {
    let mut doc = DiagramDocument::default();
    let node1_id = NodeId::new(String::from("n1"));
    let node2_id = NodeId::new(String::from("n2"));
    doc.document.nodes = doc
        .document
        .nodes
        .update(node1_id.clone(), node_at(10.0, 10.0))
        .update(node2_id.clone(), node_at(100.0, 100.0));
    doc.editor_state.selected_items = doc.editor_state.selected_items.update(node1_id.to_string());

    apply_rubber_band_release(&mut doc, (50.0, 50.0), (150.0, 150.0), false);

    assert!(!doc
        .editor_state
        .selected_items
        .contains(&node1_id.to_string()));
    assert!(doc
        .editor_state
        .selected_items
        .contains(&node2_id.to_string()));
}

#[cfg(kani)]
#[kani::proof]
#[test]
#[allow(clippy::unwrap_used)]
fn given_subgraph_release_bounds_when_drag_too_small_then_none() {
    let grid = GridSize::new(20.0).unwrap();
    let result = subgraph_release_bounds((0.0, 0.0), (10.0, 10.0), false, grid);
    assert!(result.is_none());
}

#[cfg(kani)]
#[kani::proof]
#[test]
#[allow(clippy::unwrap_used)]
fn given_subgraph_release_bounds_when_drag_valid_then_bounds_returned() {
    let grid = GridSize::new(20.0).unwrap();
    let result = subgraph_release_bounds((5.0, 10.0), (60.0, 70.0), false, grid);
    assert_eq!(result, Some((5.0, 10.0, 55.0, 60.0)));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_icon_side_when_too_small_then_fit_never_panics_and_stays_non_negative() {
    let result = fit_icon_side(19.68);
    assert!(result >= 0.0);
    assert!(result <= 11.68);
}
