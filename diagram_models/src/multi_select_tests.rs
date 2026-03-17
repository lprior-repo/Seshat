use crate::document::{
    DiagramDocument, DocumentData, EditorState, LockState, NodeId, NodeKind, OrderedFloat,
};
use crate::test_utils::builders::{setup_doc, test_node, test_node_builder, DocBuilder};
use im::HashMap;

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_mul031_move_preserves_relative_positions() {
    let mut doc = setup_doc();
    let selection = NonEmptyVec::try_from(vec![
        NodeId::new("A".to_string()),
        NodeId::new("B".to_string()),
    ])
    .unwrap();

    let res = move_selection(&mut doc, selection, Vector2D { x: 5.0, y: 5.0 });
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

#[cfg(kani)]
#[kani::proof]
#[test]
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
    let res = resize_selection(&mut doc, selection, new_bounds);
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

    // A relative to min (10,10) was (0,0). With new min (0,0) and scale 2, new pos is 0,0. new size 100,100.
    assert_eq!(node_a.x.0, 0.0);
    assert_eq!(node_a.width.0, 100.0);

    // B relative to min (10,10) was (10,10). With scale 2, new relative pos is (20,20). new min is 0, so pos is 20,20. new size 60,60.
    assert_eq!(node_b.x.0, 20.0);
    assert_eq!(node_b.width.0, 60.0);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_mul033_delete_removes_all_selected_items() {
    let mut doc = setup_doc();
    let selection = NonEmptyVec::try_from(vec![
        NodeId::new("A".to_string()),
        NodeId::new("B".to_string()),
    ])
    .unwrap();

    let res = delete_selection(&mut doc, selection);
    assert_eq!(res, Ok(()));

    assert!(!doc
        .document
        .nodes
        .contains_key(&NodeId::new("A".to_string())));
    assert!(!doc
        .document
        .nodes
        .contains_key(&NodeId::new("B".to_string())));
    assert!(doc.editor_state.selected_items.is_empty());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_mul034_copy_paste_duplicates_selection_with_offset() {
    let mut doc = setup_doc();
    let selection = NonEmptyVec::try_from(vec![
        NodeId::new("A".to_string()),
        NodeId::new("B".to_string()),
    ])
    .unwrap();

    let clipboard = copy_selection(&doc, selection).unwrap();
    assert_eq!(clipboard.nodes.len(), 2);

    let pasted_ids = paste_selection(&mut doc, &clipboard, Vector2D { x: 10.0, y: 10.0 }).unwrap();
    assert_eq!(pasted_ids.len(), 2);

    let new_a = doc.document.nodes.get(&pasted_ids[0]).unwrap();
    assert_eq!(new_a.x.0, 20.0); // original 10 + 10

    let new_b = doc.document.nodes.get(&pasted_ids[1]).unwrap();
    assert_eq!(new_b.x.0, 30.0); // original 20 + 10

    // Check selection updated
    assert!(doc
        .editor_state
        .selected_items
        .contains(&pasted_ids[0].to_string()));
    assert!(doc
        .editor_state
        .selected_items
        .contains(&pasted_ids[1].to_string()));
    assert!(!doc.editor_state.selected_items.contains("A"));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p2_violation_returns_item_locked_error() {
    let mut doc = setup_doc();
    doc.document
        .nodes
        .get_mut(&NodeId::new("A".to_string()))
        .unwrap()
        .lock_state = LockState::Locked;

    let selection = NonEmptyVec::try_from(vec![
        NodeId::new("A".to_string()),
        NodeId::new("B".to_string()),
    ])
    .unwrap();

    let res = delete_selection(&mut doc, selection.clone());
    assert_eq!(res, Err(Error::ItemLocked));

    let res = move_selection(&mut doc, selection, Vector2D { x: 5.0, y: 5.0 });
    assert_eq!(res, Err(Error::ItemLocked));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p3_violation_returns_invalid_hierarchy_error() {
    let mut doc = setup_doc();
    doc.document
        .nodes
        .get_mut(&NodeId::new("B".to_string()))
        .unwrap()
        .parent = Some(NodeId::new("A".to_string()));

    let selection = NonEmptyVec::try_from(vec![
        NodeId::new("A".to_string()),
        NodeId::new("B".to_string()),
    ])
    .unwrap();

    let res = move_selection(&mut doc, selection, Vector2D { x: 5.0, y: 5.0 });
    assert_eq!(res, Err(Error::InvalidHierarchy));
}
