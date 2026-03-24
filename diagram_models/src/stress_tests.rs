//! Adversarial stress tests for diagram models.
//!
//! Focuses on large data sets, extreme coordinates, and complex parent-child relationships.

use crate::clipboard_contract::{calculate_paste, copy, ClipboardData, Selection};
use crate::document::{NodeId, NodeKind, OrderedFloat};
use crate::subgraph::bounds::recompute_affected_container_bounds;
use crate::test_utils::{test_node_builder, DocBuilder};
use crate::transform::{
    calculate_alignment, calculate_distribution, AlignmentAxis, AlignmentMode, TransformError,
};
use im::HashMap;

#[test]
fn stress_test_calculate_paste_large_selection() {
    let mut builder = DocBuilder::new();
    let count = 1000;
    let mut selection_ids = Vec::new();

    // Create a chain of nodes: Node0 -> Node1 -> Node2 ...
    for i in 0..count {
        let id = format!("node_{}", i);
        let mut node_builder = test_node_builder(i as f64 * 10.0, i as f64 * 10.0, 50.0, 50.0);
        if i > 0 {
            node_builder = node_builder.with_parent(NodeId::new(format!("node_{}", i - 1)));
        }
        builder = builder.add_node(id.clone(), node_builder.build());
        selection_ids.push(NodeId::new(id));
    }

    let doc = builder.build();
    let selection = Selection {
        nodes: selection_ids,
    };
    let clipboard = copy(&selection, &doc).expect("Should copy selection");
    let result = calculate_paste(&clipboard, &doc).expect("Should paste selection");

    assert_eq!(result.new_nodes.len(), count);
    assert_eq!(result.new_selection.len(), count);

    // Verify parent remapping
    let id_map: HashMap<NodeId, NodeId> = result
        .new_nodes
        .iter()
        .enumerate()
        .map(|(i, (new_id, _))| (NodeId::new(format!("node_{}", i)), new_id.clone()))
        .collect();

    for (new_id, new_node) in &result.new_nodes {
        if let Some(parent_id) = &new_node.parent {
            // Find which old node this new node corresponds to
            let old_id = id_map
                .iter()
                .find(|(_, nid)| *nid == new_id)
                .map(|(oid, _)| oid)
                .unwrap();
            let old_node = doc.document.nodes.get(old_id).unwrap();

            if let Some(old_parent) = &old_node.parent {
                let expected_new_parent = id_map.get(old_parent).expect("Parent not remapped");
                assert_eq!(parent_id, expected_new_parent);
            }
        }
    }
}

#[test]
fn stress_test_transform_extremes() {
    let mut builder = DocBuilder::new();
    // Use extremely large but finite coordinates
    let large_val = 1.0e30;
    builder = builder
        .add_node_with("n1", -large_val, -large_val, 10.0, 10.0)
        .add_node_with("n2", large_val, large_val, 10.0, 10.0)
        .add_node_with("n3", 0.0, 0.0, 10.0, 10.0);

    let doc = builder.build();
    let selection = vec![
        NodeId::new("n1".to_string()),
        NodeId::new("n2".to_string()),
        NodeId::new("n3".to_string()),
    ];

    // Alignment should handle large values without panicking
    let result = calculate_alignment(
        &doc.document.nodes,
        &selection,
        AlignmentAxis::Horizontal,
        AlignmentMode::Center,
    );
    assert!(result.is_ok());

    // Distribution should handle large values
    let result = calculate_distribution(&doc.document.nodes, &selection, AlignmentAxis::Horizontal);
    assert!(result.is_ok());
}

#[test]
fn stress_test_transform_empty_singleton() {
    let mut builder = DocBuilder::new();
    builder = builder.add_node_with("n1", 0.0, 0.0, 10.0, 10.0);
    let doc = builder.build();

    // Empty selection
    let result = calculate_alignment(
        &doc.document.nodes,
        &[],
        AlignmentAxis::Horizontal,
        AlignmentMode::Start,
    );
    assert!(matches!(result, Err(TransformError::EmptySelection)));

    // Singleton selection (alignment requires 2+)
    let result = calculate_alignment(
        &doc.document.nodes,
        &[NodeId::new("n1".to_string())],
        AlignmentAxis::Horizontal,
        AlignmentMode::Start,
    );
    assert!(matches!(result, Err(TransformError::EmptySelection)));

    // Less than 3 for distribution
    let result = calculate_distribution(
        &doc.document.nodes,
        &[NodeId::new("n1".to_string())],
        AlignmentAxis::Horizontal,
    );
    assert!(matches!(result, Err(TransformError::EmptySelection)));
}

#[test]
fn stress_test_subgraph_bounds_deep_nesting() {
    let mut nodes = HashMap::new();
    let depth = 100;

    // Create nested subgraphs: G0 contains G1 contains G2 ... contains Leaf
    for i in 0..depth {
        let id = NodeId::new(format!("G{}", i));
        let mut builder = test_node_builder(0.0, 0.0, 100.0, 100.0).with_kind(NodeKind::Subgraph);
        if i > 0 {
            builder = builder.with_parent(NodeId::new(format!("G{}", i - 1)));
        }
        nodes.insert(id, builder.build());
    }

    // Leaf node in the deepest subgraph
    let leaf_id = NodeId::new("Leaf".to_string());
    nodes.insert(
        leaf_id.clone(),
        test_node_builder(10.0, 10.0, 50.0, 50.0)
            .with_parent(NodeId::new(format!("G{}", depth - 1)))
            .build(),
    );

    // Moving the leaf node
    let moved = vec![leaf_id];

    // Note: recompute_affected_container_bounds currently only recomputes the immediate parent.
    // If we want it to propagate up, it might need multiple passes or recursion.
    // Let's verify current behavior first.
    let result = recompute_affected_container_bounds(nodes.clone(), &moved);

    let deepest_parent_id = NodeId::new(format!("G{}", depth - 1));
    let deepest_parent = result
        .get(&deepest_parent_id)
        .expect("Deepest parent missing");

    // Leaf is at (10, 10) size 50x50.
    // Subgraph bounds should be x=10-24=-14, y=10-24=-14, w=50+48=98, h=50+48=98
    assert_eq!(deepest_parent.x.0, -14.0);
    assert_eq!(deepest_parent.y.0, -14.0);
    assert_eq!(deepest_parent.width.0, 98.0);
    assert_eq!(deepest_parent.height.0, 98.0);

    // Verify that the parent of the deepest parent (G_{depth-2}) was NOT recomputed
    // because it wasn't in the immediate parent set of 'moved'.
    // If this is the intended behavior, the test passes.
    // If it's a bug, the test highlights it.
    if depth > 1 {
        let grandparent_id = NodeId::new(format!("G{}", depth - 2));
        let grandparent = result.get(&grandparent_id).expect("Grandparent missing");
        assert_eq!(grandparent.x.0, 0.0); // Remains unchanged
    }
}

#[test]
fn stress_test_stability_no_panics() {
    let mut builder = DocBuilder::new();
    // Mix of invalid/extreme values that should be handled gracefully
    builder = builder
        .add_node_with("n1", f64::NAN, 0.0, 10.0, 10.0) // This should actually be impossible due to OrderedFloat::new_unchecked in builder but let's see
        .add_node_with("n2", f64::INFINITY, f64::NEG_INFINITY, 1.0, 1.0)
        .add_node_with("n3", 0.0, 0.0, 0.0, 0.0);

    let doc = builder.build();
    let selection = vec![
        NodeId::new("n1".to_string()),
        NodeId::new("n2".to_string()),
        NodeId::new("n3".to_string()),
    ];

    // We expect these to maybe fail but definitely not panic.
    let _ = calculate_alignment(
        &doc.document.nodes,
        &selection,
        AlignmentAxis::Horizontal,
        AlignmentMode::Center,
    );
    let _ = calculate_distribution(&doc.document.nodes, &selection, AlignmentAxis::Horizontal);
    let _ = recompute_affected_container_bounds(doc.document.nodes.clone(), &selection);
}

#[test]
fn stress_test_hammer_transforms() {
    let mut builder = DocBuilder::new();
    let count = 100;
    for i in 0..count {
        builder = builder.add_node_with(format!("n{}", i), i as f64, i as f64, 10.0, 10.0);
    }
    let mut doc = builder.build();
    let selection: Vec<_> = (0..count).map(|i| NodeId::new(format!("n{}", i))).collect();

    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0..1000 {
        let axis = if rng.gen_bool(0.5) {
            AlignmentAxis::Horizontal
        } else {
            AlignmentAxis::Vertical
        };
        let mode = match rng.gen_range(0..3) {
            0 => AlignmentMode::Start,
            1 => AlignmentMode::Center,
            _ => AlignmentMode::End,
        };

        if rng.gen_bool(0.5) {
            let _ = calculate_alignment(&doc.document.nodes, &selection, axis, mode);
        } else {
            let _ = calculate_distribution(&doc.document.nodes, &selection, axis);
        }

        // Randomly move nodes to extreme positions
        if rng.gen_bool(0.1) {
            let id = NodeId::new(format!("n{}", rng.gen_range(0..count)));
            if let Some(node) = doc.document.nodes.get_mut(&id) {
                node.x = OrderedFloat::new_unchecked(rng.gen_range(-1e30..1e30));
                node.y = OrderedFloat::new_unchecked(rng.gen_range(-1e30..1e30));
            }
        }
    }
}

#[test]
fn stress_test_subgraph_bounds_mixed_kinds() {
    let mut nodes = HashMap::new();
    let g1 = NodeId::new("G1".to_string());
    let n1 = NodeId::new("N1".to_string());
    let t1 = NodeId::new("T1".to_string());

    // G1 is a subgraph
    nodes.insert(
        g1.clone(),
        test_node_builder(0.0, 0.0, 100.0, 100.0)
            .with_kind(NodeKind::Subgraph)
            .build(),
    );

    // N1 is a regular node, parent is G1
    nodes.insert(
        n1.clone(),
        test_node_builder(10.0, 10.0, 50.0, 50.0)
            .with_parent(g1.clone())
            .build(),
    );

    // T1 is a text node, parent is N1 (NOT a subgraph)
    nodes.insert(
        t1.clone(),
        test_node_builder(5.0, 5.0, 20.0, 20.0)
            .with_parent(n1.clone())
            .with_kind(NodeKind::Text)
            .build(),
    );

    let moved = vec![t1];
    let result = recompute_affected_container_bounds(nodes.clone(), &moved);

    // N1 is NOT a subgraph, so its bounds should NOT be recomputed
    let node_n1 = result.get(&n1).unwrap();
    assert_eq!(node_n1.x.0, 10.0);

    // G1 is a subgraph, but its child (N1) didn't move directly (only T1 did).
    // Wait, the function filters parents of moved nodes.
    // T1's parent is N1. N1 is NOT a subgraph. So G1 is NOT affected according to the current logic.
    let node_g1 = result.get(&g1).unwrap();
    assert_eq!(node_g1.x.0, 0.0);
}
