#![allow(dead_code, unused_imports)]
use crate::document::{
    ArrowType, Edge, EdgeId, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
};

use im::HashMap;
use proptest::prelude::*;

fn arb_node_id() -> impl Strategy<Value = NodeId> {
    "[a-z]{1,8}".prop_map(NodeId::new)
}

fn arb_edge_id() -> impl Strategy<Value = EdgeId> {
    "e_[a-z]{1,8}".prop_map(EdgeId::new)
}

fn arb_node_kind() -> impl Strategy<Value = NodeKind> {
    prop_oneof![
        Just(NodeKind::Node),
        Just(NodeKind::Subgraph),
        Just(NodeKind::Text)
    ]
}

fn arb_ordered_float_with_specials() -> impl Strategy<Value = OrderedFloat> {
    prop_oneof![
        any::<f64>().prop_map(OrderedFloat::new_unchecked),
        Just(OrderedFloat::new_unchecked(f64::NAN)),
        Just(OrderedFloat::new_unchecked(f64::INFINITY)),
        Just(OrderedFloat::new_unchecked(f64::NEG_INFINITY)),
        Just(OrderedFloat::new_unchecked(0.0)),
        Just(OrderedFloat::new_unchecked(f64::MIN)),
        Just(OrderedFloat::new_unchecked(f64::MAX)),
    ]
}

fn make_node(kind: NodeKind, parent: Option<NodeId>, x: f64, y: f64) -> Node {
    Node {
        kind,
        icon: String::new(),
        label: String::new(),
        x: OrderedFloat::new_unchecked(x),
        y: OrderedFloat::new_unchecked(y),
        width: OrderedFloat::new_unchecked(100.0),
        height: OrderedFloat::new_unchecked(60.0),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent,
        dag_rank: None,
        tags: im::vector![],
        metadata: HashMap::new(),
        z_index: 0,
        style: Some(NodeStyle::default()),
        collapsed: None,
    }
}

fn make_edge(source: NodeId, target: NodeId) -> Edge {
    Edge {
        source,
        target,
        label: String::new(),
        style: crate::document::EdgeStyle::Solid,
        arrow_type: ArrowType::Default,
        label_offset_t: OrderedFloat::new_unchecked(0.5),
        color: None,
        thickness: OrderedFloat::new_unchecked(1.5),
        directed: true,
        bend_points: im::vector![],
        tags: im::vector![],
        metadata: HashMap::new(),
        font_size: None,
        source_port: None,
        target_port: None,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[cfg(kani)]
#[kani::proof]
    fn prop_version_must_be_2(version in 0u32..100) {
        let doc = DiagramDocument {
            version,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new(),
                edges: HashMap::new(),
            },
            editor_state: EditorState::default(),
        };
        let result = validate_schema(&doc);
        if version == 2 {
            prop_assert!(result.is_ok());
        } else {
            prop_assert!(result.is_err());
        }
    }

    #[cfg(kani)]
#[kani::proof]
    fn prop_editor_state_extreme_floats(
        camera_x in arb_ordered_float_with_specials(),
        camera_y in arb_ordered_float_with_specials(),
        zoom in arb_ordered_float_with_specials(),
        grid_size_f64 in arb_ordered_float_with_specials(),
    ) {
        let grid_size = GridSize::new(grid_size_f64.0).unwrap_or_default();
        let doc = DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new(),
                edges: HashMap::new(),
            },
            editor_state: EditorState {
                camera_x,
                camera_y,
                zoom,
                grid_size,
                snap_to_grid: true,
                selected_items: im::HashSet::new(),
                edit_mode_target: None,
                editing_edge_id: None,
                theme: crate::document::EditorTheme::System,
                show_grid: true,
                minimap_visible: false,
            },
        };
        let result = validate_schema(&doc);
        // Smoke test - just verify no panic with extreme float values
        let _ = result;
    }

    #[cfg(kani)]
#[kani::proof]
    fn prop_edge_to_nonexistent_node_fails(
        source in arb_node_id(),
        nonexistent in arb_node_id(),
        edge_id in arb_edge_id(),
    ) {
        prop_assume!(source != nonexistent);
        let doc = DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new().update(source.clone(), make_node(NodeKind::Node, None, 0.0, 0.0)),
                edges: HashMap::new().update(edge_id, make_edge(source, nonexistent)),
            },
            editor_state: EditorState::default(),
        };
        prop_assert!(validate_schema(&doc).is_err());
    }

    #[cfg(kani)]
#[kani::proof]
    fn prop_edge_both_nodes_nonexistent(
        source in arb_node_id(),
        target in arb_node_id(),
        edge_id in arb_edge_id(),
    ) {
        prop_assume!(source != target);
        let doc = DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new(),
                edges: HashMap::new().update(edge_id, make_edge(source, target)),
            },
            editor_state: EditorState::default(),
        };
        prop_assert!(validate_schema(&doc).is_err());
    }

    #[cfg(kani)]
#[kani::proof]
    fn prop_node_parent_must_exist_and_be_subgraph(
        child_id in arb_node_id(),
        parent_id in arb_node_id(),
        parent_kind in arb_node_kind(),
    ) {
        prop_assume!(child_id != parent_id);
        let is_subgraph = parent_kind == NodeKind::Subgraph;
        let doc = DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new()
                    .update(parent_id.clone(), make_node(parent_kind, None, 0.0, 0.0))
                    .update(child_id, make_node(NodeKind::Node, Some(parent_id.clone()), 0.0, 0.0)),
                edges: HashMap::new(),
            },
            editor_state: EditorState::default(),
        };
        let result = validate_schema(&doc);
        if is_subgraph {
            prop_assert!(result.is_ok());
        } else {
            prop_assert!(result.is_err());
        }
    }

    #[cfg(kani)]
#[kani::proof]
    fn prop_node_references_missing_parent(child_id in arb_node_id(), missing in arb_node_id()) {
        let doc = DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new().update(child_id, make_node(NodeKind::Node, Some(missing), 0.0, 0.0)),
                edges: HashMap::new(),
            },
            editor_state: EditorState::default(),
        };
        prop_assert!(validate_schema(&doc).is_err());
    }

    #[cfg(kani)]
#[kani::proof]
    fn prop_self_referential_edge(node_id in arb_node_id(), edge_id in arb_edge_id()) {
        let doc = DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new().update(node_id.clone(), make_node(NodeKind::Node, None, 0.0, 0.0)),
                edges: HashMap::new().update(edge_id, make_edge(node_id.clone(), node_id)),
            },
            editor_state: EditorState::default(),
        };
        let result = validate_schema(&doc);
        prop_assert!(result.is_err(), "self-loop should fail DAG validation");
    }

    #[cfg(kani)]
#[kani::proof]
    fn prop_empty_vs_populated_empty_nodes(
        num_nodes in 0usize..10,
    ) {
        let nodes: HashMap<NodeId, Node> = (0..num_nodes)
            .map(|i| {
                let id = NodeId::new(format!("n{}", i));
                (id.clone(), make_node(NodeKind::Node, None, i as f64 * 100.0, 0.0))
            })
            .collect();
        let doc = DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes,
                edges: HashMap::new(),
            },
            editor_state: EditorState::default(),
        };
        prop_assert!(validate_schema(&doc).is_ok());
    }

    #[cfg(kani)]
#[kani::proof]
    fn prop_deeply_nested_parent_chain(depth in 1usize..20) {
        let mut nodes = HashMap::new();
        for i in 0..depth {
            let id = NodeId::new(format!("n{}", i));
            let parent = if i == 0 {
                None
            } else {
                Some(NodeId::new(format!("n{}", i - 1)))
            };
            nodes.insert(id.clone(), make_node(NodeKind::Subgraph, parent, 0.0, 0.0));
        }
        let doc = DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes,
                edges: HashMap::new(),
            },
            editor_state: EditorState::default(),
        };
        prop_assert!(validate_schema(&doc).is_ok());
    }

    #[cfg(kani)]
#[kani::proof]
    fn prop_circular_parent_chain_three_nodes(
        id_a in arb_node_id(),
        id_b in arb_node_id(),
        id_c in arb_node_id(),
    ) {
        prop_assume!(id_a != id_b && id_b != id_c && id_a != id_c);
        let doc = DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new()
                    .update(id_a.clone(), make_node(NodeKind::Subgraph, Some(id_c.clone()), 0.0, 0.0))
                    .update(id_b.clone(), make_node(NodeKind::Subgraph, Some(id_a.clone()), 0.0, 0.0))
                    .update(id_c.clone(), make_node(NodeKind::Subgraph, Some(id_b), 0.0, 0.0)),
                edges: HashMap::new(),
            },
            editor_state: EditorState::default(),
        };
        prop_assert!(validate_schema(&doc).is_err(), "circular parent chain should fail");
    }

    #[cfg(kani)]
#[kani::proof]
    fn prop_multiple_edges_same_nodes(
        source in arb_node_id(),
        target in arb_node_id(),
        edge_ids in prop::collection::vec(arb_edge_id(), 1..5),
    ) {
        prop_assume!(source != target);
        let edges: HashMap<EdgeId, Edge> = edge_ids
            .into_iter()
            .map(|eid| (eid.clone(), make_edge(source.clone(), target.clone())))
            .collect();
        let doc = DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new()
                    .update(source.clone(), make_node(NodeKind::Node, None, 0.0, 0.0))
                    .update(target.clone(), make_node(NodeKind::Node, None, 100.0, 0.0)),
                edges,
            },
            editor_state: EditorState::default(),
        };
        prop_assert!(validate_schema(&doc).is_ok());
    }

    #[cfg(kani)]
#[kani::proof]
    fn prop_node_extreme_coordinates(
        node_id in arb_node_id(),
        x in any::<f64>(),
        y in any::<f64>(),
        width in any::<f64>(),
        height in any::<f64>(),
    ) {
        let mut node = make_node(NodeKind::Node, None, 0.0, 0.0);
        node.x = OrderedFloat::new_unchecked(x);
        node.y = OrderedFloat::new_unchecked(y);
        node.width = OrderedFloat::new_unchecked(width);
        node.height = OrderedFloat::new_unchecked(height);
        let doc = DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new().update(node_id, node),
                edges: HashMap::new(),
            },
            editor_state: EditorState::default(),
        };
        let _ = validate_schema(&doc);
    }

    #[cfg(kani)]
#[kani::proof]
    fn prop_complex_dag(
        num_nodes in 2usize..10,
        edge_density in 0.0f64..1.0,
    ) {
        let mut nodes = HashMap::new();
        let node_ids: Vec<NodeId> = (0..num_nodes)
            .map(|i| NodeId::new(format!("n{}", i)))
            .collect();
        for id in &node_ids {
            nodes.insert(id.clone(), make_node(NodeKind::Node, None, 0.0, 0.0));
        }
        let mut edges = HashMap::new();
        let mut edge_count = 0usize;
        for (i, src) in node_ids.iter().enumerate() {
            for (j, tgt) in node_ids.iter().enumerate() {
                if i < j && (i as f64 + j as f64) * edge_density < num_nodes as f64 {
                    edges.insert(
                        EdgeId::new(format!("e{}", edge_count)),
                        make_edge(src.clone(), tgt.clone()),
                    );
                    edge_count += 1;
                }
            }
        }
        let doc = DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData { nodes, edges },
            editor_state: EditorState::default(),
        };
        let _ = validate_schema(&doc);
    }

    #[cfg(kani)]
#[kani::proof]
    fn prop_subgraph_with_children(
        subgraph_id in arb_node_id(),
        child_ids in prop::collection::vec(arb_node_id(), 1..5),
    ) {
        let mut nodes = HashMap::new();
        nodes.insert(subgraph_id.clone(), make_node(NodeKind::Subgraph, None, 0.0, 0.0));
        for child in &child_ids {
            prop_assume!(*child != subgraph_id);
            nodes.insert(child.clone(), make_node(NodeKind::Node, Some(subgraph_id.clone()), 0.0, 0.0));
        }
        let doc = DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes,
                edges: HashMap::new(),
            },
            editor_state: EditorState::default(),
        };
        prop_assert!(validate_schema(&doc).is_ok());
    }
}
