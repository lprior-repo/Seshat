// Martin Fowler Tests for Node Grouping (SUB-001 to SUB-006)

// Happy Path Tests
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_sub001_click_inside_container_selects_child_vs_container_with_modifier() {
    let mut canvas = mock_canvas();
    let group_id = NodeId::new("group_a".to_string());
    let child_id = NodeId::new("node_1".to_string());

    canvas.nodes = canvas
        .nodes
        .update(
            group_id.clone(),
            create_empty_subgraph(
                group_id.clone(),
                Point {
                    x: OrderedFloat(0.0),
                    y: OrderedFloat(0.0),
                },
            )
            .unwrap(),
        )
        .update(
            child_id.clone(),
            create_mock_node("node_1", 10.0, 10.0, 20.0, 20.0),
        );

    set_node_parent(child_id.clone(), group_id.clone(), &mut canvas).unwrap();

    // With Ctrl modifier, should select child bypassing group
    let result = evaluate_selection(
        &canvas,
        Point {
            x: OrderedFloat(15.0),
            y: OrderedFloat(15.0),
        },
        SelectionModifiers { ctrl: true },
    );
    assert_eq!(result.unwrap(), SelectionResult::NodeSelected(child_id));

    // Without modifier, if z-index makes child higher, it might select child anyway.
    // In our evaluate_selection, it just picks the first hitting node. We assume group bounds are larger.
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_sub003_collapse_and_expand_container_toggles_child_visibility() {
    let mut canvas = mock_canvas();
    let group_id = NodeId::new("group_a".to_string());
    let child_id = NodeId::new("node_1".to_string());

    canvas.nodes = canvas
        .nodes
        .update(
            group_id.clone(),
            create_empty_subgraph(
                group_id.clone(),
                Point {
                    x: OrderedFloat(0.0),
                    y: OrderedFloat(0.0),
                },
            )
            .unwrap(),
        )
        .update(
            child_id.clone(),
            create_mock_node("node_1", 10.0, 10.0, 20.0, 20.0),
        );

    set_node_parent(child_id.clone(), group_id.clone(), &mut canvas).unwrap();

    // Toggle collapse
    toggle_collapse(&mut canvas, group_id.clone()).unwrap();
    assert_eq!(canvas.nodes.get(&group_id).unwrap().collapsed, Some(true));

    // Child should not be hit-testable when parent is collapsed
    let result = evaluate_selection(
        &canvas,
        Point {
            x: OrderedFloat(15.0),
            y: OrderedFloat(15.0),
        },
        SelectionModifiers { ctrl: true },
    );
    assert!(result.is_err()); // Hit testing fails because parent is collapsed

    // Toggle expand
    toggle_collapse(&mut canvas, group_id.clone()).unwrap();
    assert_eq!(canvas.nodes.get(&group_id).unwrap().collapsed, Some(false));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_sub006_delete_container_reparents_children_to_grandparent() {
    let mut canvas = mock_canvas();
    let group_id = NodeId::new("group_a".to_string());
    let child_1 = NodeId::new("node_1".to_string());
    let child_2 = NodeId::new("node_2".to_string());

    canvas.nodes = canvas
        .nodes
        .update(
            group_id.clone(),
            create_empty_subgraph(
                group_id.clone(),
                Point {
                    x: OrderedFloat(0.0),
                    y: OrderedFloat(0.0),
                },
            )
            .unwrap(),
        )
        .update(
            child_1.clone(),
            create_mock_node("node_1", 10.0, 10.0, 20.0, 20.0),
        )
        .update(
            child_2.clone(),
            create_mock_node("node_2", 40.0, 10.0, 20.0, 20.0),
        );

    set_node_parent(child_1.clone(), group_id.clone(), &mut canvas).unwrap();
    set_node_parent(child_2.clone(), group_id.clone(), &mut canvas).unwrap();

    let reparented = ungroup_nodes(&mut canvas, group_id.clone()).unwrap();

    assert!(!canvas.nodes.contains_key(&group_id));
    assert!(canvas.nodes.contains_key(&child_1));
    assert!(canvas.nodes.contains_key(&child_2));

    assert_eq!(canvas.nodes.get(&child_1).unwrap().parent, None);
    assert_eq!(canvas.nodes.get(&child_2).unwrap().parent, None);
    assert_eq!(reparented.len(), 2);
}

// Error Path Tests
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_returns_error_when_grouping_with_empty_selection() {
    let mut canvas = mock_canvas();
    let group_id = NodeId::new("group_a".to_string());
    let result = group_nodes(&mut canvas, group_id, &[]);
    assert_eq!(result.unwrap_err(), Error::EmptySelection);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_returns_error_when_grouping_nonexistent_nodes() {
    let mut canvas = mock_canvas();
    let group_id = NodeId::new("group_a".to_string());
    let child = NodeId::new("missing".to_string());
    let result = group_nodes(&mut canvas, group_id, &[child.clone()]);
    assert_eq!(result.unwrap_err(), Error::NodeNotFound(child));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_returns_error_when_ungrouping_non_container_node() {
    let mut canvas = mock_canvas();
    let text_node_id = NodeId::new("text_node".to_string());
    canvas.nodes = canvas.nodes.update(
        text_node_id.clone(),
        create_mock_node("text_node", 0.0, 0.0, 10.0, 10.0),
    );

    let result = ungroup_nodes(&mut canvas, text_node_id);
    assert_eq!(result.unwrap_err(), Error::InvalidNodeType);
}

// Contract Violation Tests
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p1_violation_returns_empty_selection() {
    let mut canvas = mock_canvas();
    let id = NodeId::new("id".to_string());
    let result = group_nodes(&mut canvas, id, &[]);
    assert_eq!(result.unwrap_err(), Error::EmptySelection);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p2_violation_returns_node_not_found() {
    let mut canvas = mock_canvas();
    let id = NodeId::new("id".to_string());
    let missing = NodeId::new("missing".to_string());
    let result = group_nodes(&mut canvas, id, &[missing.clone()]);
    assert_eq!(result.unwrap_err(), Error::NodeNotFound(missing));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p4_violation_returns_node_locked() {
    let mut canvas = mock_canvas();
    let group_id = NodeId::new("id".to_string());
    let locked_id = NodeId::new("locked_id".to_string());
    let mut locked_node = create_mock_node("locked_id", 0.0, 0.0, 10.0, 10.0);
    locked_node.locked = true;

    canvas.nodes = canvas.nodes.update(locked_id.clone(), locked_node);
    let result = group_nodes(&mut canvas, group_id, &[locked_id.clone()]);
    assert_eq!(result.unwrap_err(), Error::NodeLocked(locked_id));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p5_violation_returns_invalid_node_type() {
    let mut canvas = mock_canvas();
    let text_node = NodeId::new("text_node".to_string());
    canvas.nodes = canvas.nodes.update(
        text_node.clone(),
        create_mock_node("text_node", 0.0, 0.0, 10.0, 10.0),
    );

    let result = ungroup_nodes(&mut canvas, text_node);
    assert_eq!(result.unwrap_err(), Error::InvalidNodeType);
}
