#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
use crate::clipboard::{calculate_paste, copy_selection, ClipboardData};
use crate::document::{DiagramDocument, LockState, NodeId, OrderedFloat};
use crate::multi_select::{
    delete_selection, move_selection, resize_selection, Error, NonEmptyVec, Rect, Vector2D,
};
use crate::test_utils::setup_doc;

fn paste_selection(
    doc: &mut DiagramDocument,
    clipboard: &ClipboardData,
    _delta: Vector2D,
) -> Result<Vec<NodeId>, Error> {
    let result = calculate_paste(clipboard, doc);
    let mut ids: Vec<NodeId> = result
        .selected
        .iter()
        .map(|s| NodeId::new(s.clone()))
        .collect();
    doc.document.nodes = result.nodes;
    doc.document.edges = result.edges;
    doc.editor_state.selected_items = result.selected;
    ids.sort();
    Ok(ids)
}

#[cfg_attr(kani, kani::proof)]
fn test_mul031_move_preserves_relative_positions() {
    let mut doc = setup_doc();
    let selection = NonEmptyVec::try_from(vec![
        NodeId::new("A".to_string()),
        NodeId::new("B".to_string()),
    ])
    .unwrap();

    let res = move_selection(&mut doc, &selection, Vector2D { x: 5.0, y: 5.0 });
    assert_eq!(res, Ok(()));

    let node_a = doc
        .document
        .nodes
        .get(&NodeId::new("A".to_string()))
        .unwrap();
    let node_b = doc
        .document
        .nodes
        .get(&NodeId::new("B".to_string()))
        .unwrap();

    assert_eq!(node_a.x.0, 15.0);
    assert_eq!(node_a.y.0, 15.0);
    assert_eq!(node_b.x.0, 25.0);
    assert_eq!(node_b.y.0, 25.0);

    // Relative distance preserved: 25 - 15 = 10 (same as 20 - 10)
    assert_eq!(node_b.x.0 - node_a.x.0, 10.0);
}

#[cfg_attr(kani, kani::proof)]
fn test_mul032_resize_scales_items_proportionally() {
    let mut doc = setup_doc();
    let selection = NonEmptyVec::try_from(vec![
        NodeId::new("A".to_string()),
        NodeId::new("B".to_string()),
    ])
    .unwrap();

    // original bounds of A and B:
    // A: 10,10 to 60,60
    // B: 20,20 to 50,50
    // combined bounds: 10,10 to 60,60. width=50, height=50
    // resize to 100x100 at 0,0 -> scale=2.0, translation
    let new_bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    };
    let res = resize_selection(&mut doc, &selection, new_bounds);
    assert_eq!(res, Ok(()));

    let node_a = doc
        .document
        .nodes
        .get(&NodeId::new("A".to_string()))
        .unwrap();
    let node_b = doc
        .document
        .nodes
        .get(&NodeId::new("B".to_string()))
        .unwrap();

    // A: (10,10) -> (0,0), size 50 -> 100
    assert_eq!(node_a.x.0, 0.0);
    assert_eq!(node_a.y.0, 0.0);
    assert_eq!(node_a.width.0, 100.0);

    // B: (20,20) -> relative (10,10) scaled by 2.0 -> (20,20)
    // Wait, if anchor is 10,10 and we resize to 0,0 100x100.
    // Scale is 2.0. New origin is 0,0.
    // node B original x is 20. rel_x = 20 - 10 = 10.
    // scaled_rel_x = 10 * 2.0 = 20.
    // new_x = 0 + 20 = 20. Correct.
    assert_eq!(node_b.x.0, 20.0);
    assert_eq!(node_b.y.0, 20.0);
    assert_eq!(node_b.width.0, 60.0);
}

#[test]
fn test_mul033_delete_removes_from_document() {
    let mut doc = setup_doc();
    let selection = NonEmptyVec::try_from(vec![NodeId::new("A".to_string())]).unwrap();

    let res = delete_selection(&mut doc, &selection);
    assert_eq!(res, Ok(()));
    assert!(!doc
        .document
        .nodes
        .contains_key(&NodeId::new("A".to_string())));
    assert!(doc
        .document
        .nodes
        .contains_key(&NodeId::new("B".to_string())));
}

#[test]
fn test_mul034_copy_paste_clones_selection() {
    let mut doc = setup_doc();
    let selection = vec![NodeId::new("A".to_string())];

    let clipboard = copy_selection(&doc, &selection).unwrap();
    let pasted_ids = paste_selection(&mut doc, &clipboard, Vector2D { x: 0.0, y: 0.0 }).unwrap();

    assert_eq!(pasted_ids.len(), 1);
    assert_ne!(pasted_ids[0], NodeId::new("A".to_string()));
    assert!(doc.document.nodes.contains_key(&pasted_ids[0]));
}

#[test]
fn test_mul035_paste_applies_offset() {
    let mut doc = setup_doc();
    let selection = vec![NodeId::new("A".to_string())];
    let clipboard = copy_selection(&doc, &selection).unwrap();

    let pasted_ids = paste_selection(&mut doc, &clipboard, Vector2D { x: 0.0, y: 0.0 }).unwrap();
    let pasted_node = doc.document.nodes.get(&pasted_ids[0]).unwrap();

    // Default offset is 20px
    assert_eq!(pasted_node.x.0, 30.0); // 10 + 20
}
