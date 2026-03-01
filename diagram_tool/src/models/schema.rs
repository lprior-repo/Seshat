#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::dag::validate_dag;
use crate::models::document::{DiagramDocument, NodeId, NodeKind};
use anyhow::{anyhow, bail, Result};
use im::HashSet;

/// Functional schema validation.
pub fn validate_schema(doc: &DiagramDocument) -> Result<()> {
    if doc.version != 2 {
        bail!("Document version must be 2, got {}", doc.version);
    }

    let nodes = &doc.document.nodes;
    let node_ids = nodes.keys().cloned().collect::<HashSet<NodeId>>();

    // 1. Validate Nodes
    nodes.iter().try_for_each(|(id, node)| {
        if node.width.0 < 0.0 {
            bail!("Node {id} has negative width: {}", node.width.0);
        }
        if node.height.0 < 0.0 {
            bail!("Node {id} has negative height: {}", node.height.0);
        }
        if let Some(parent_id) = &node.parent {
            if !node_ids.contains(parent_id) {
                bail!("Node {id} references non-existent parent {parent_id}");
            }
            if !nodes
                .get(parent_id)
                .is_some_and(|p| p.kind == NodeKind::Subgraph)
            {
                bail!("Node {id} parent {parent_id} is not a subgraph");
            }
        }
        Ok(())
    })?;

    // 1b. Check for circular parent chains using functional recursion
    for (id, _) in nodes.iter() {
        let has_cycle = check_parent_cycle(nodes, id, &HashSet::new());
        if has_cycle {
            bail!("Circular parent chain detected involving node {id}");
        }
    }

    // 2. Validate Edges and DAG
    validate_edges_and_dag(doc)?;

    Ok(())
}

#[allow(clippy::redundant_clone)]
fn check_parent_cycle(
    nodes: &im::HashMap<NodeId, crate::models::document::Node>,
    current: &NodeId,
    visited: &im::HashSet<NodeId>,
) -> bool {
    if visited.contains(current) {
        return true;
    }
    let mut next_visited = visited.clone();
    next_visited.insert(current.clone());

    nodes
        .get(current)
        .and_then(|n| n.parent.as_ref())
        .is_some_and(|parent| check_parent_cycle(nodes, parent, &next_visited))
}

/// Validate edges and DAG after parent chain validation
fn validate_edges_and_dag(doc: &DiagramDocument) -> Result<()> {
    let nodes = &doc.document.nodes;
    let node_ids = nodes.keys().cloned().collect::<HashSet<NodeId>>();

    // 2. Validate Edges
    doc.document.edges.iter().try_for_each(|(id, edge)| {
        if !node_ids.contains(&edge.source) {
            bail!("Edge {id:?} references non-existent source {}", edge.source);
        }
        if !node_ids.contains(&edge.target) {
            bail!("Edge {id:?} references non-existent target {}", edge.target);
        }
        if edge.label_offset_t.0 < 0.0 || edge.label_offset_t.0 > 1.0 {
            bail!(
                "Edge {id:?} has label_offset_t {} outside valid range [0, 1]",
                edge.label_offset_t.0
            );
        }
        if let Some(ref color) = edge.color {
            if !is_valid_hex_color(color) {
                bail!("Edge {id:?} has invalid color format: {color}");
            }
        }
        Ok(())
    })?;

    // 3. Validate DAG property
    validate_dag(nodes, &doc.document.edges).map_err(|e| anyhow!("DAG Validation Failed: {e}"))?;

    Ok(())
}

fn is_valid_hex_color(color: &str) -> bool {
    color.starts_with('#')
        && match color.len() {
            4 => {
                // #RGB
                color[1..].chars().all(|c| c.is_ascii_hexdigit())
            }
            7 => {
                // #RRGGBB
                color[1..].chars().all(|c| c.is_ascii_hexdigit())
            }
            5 => {
                // #RGBA
                color[1..].chars().all(|c| c.is_ascii_hexdigit())
            }
            9 => {
                // #RRGGBBAA
                color[1..].chars().all(|c| c.is_ascii_hexdigit())
            }
            _ => false,
        }
}

#[cfg(test)]
mod tests {
    use super::validate_schema;
    use crate::models::document::{
        ArrowType, DiagramDocument, Edge, EdgeId, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    };
    use im::HashMap;

    fn node(kind: NodeKind, parent: Option<NodeId>) -> Node {
        Node {
            kind,
            icon: String::new(),
            label: String::new(),
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(100.0),
            height: OrderedFloat(60.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent,
            dag_rank: None,
            tags: vec![],
            metadata: HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    fn edge(source: &NodeId, target: &NodeId) -> Edge {
        Edge {
            source: source.clone(),
            target: target.clone(),
            label: String::new(),
            style: crate::models::document::EdgeStyle::Solid,
            arrow_type: ArrowType::Default,
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.5),
            directed: true,
            bend_points: vec![],
            tags: vec![],
            metadata: HashMap::new(),
            font_size: None,
        }
    }

    #[test]
    fn given_default_document_when_validated_then_schema_passes() {
        let doc = DiagramDocument::default();
        let result = validate_schema(&doc);
        assert!(result.is_ok());
    }

    #[test]
    fn given_non_v2_document_when_validated_then_schema_fails_without_runtime_gate() {
        let doc = DiagramDocument {
            version: 3,
            ..DiagramDocument::default()
        };

        let result = validate_schema(&doc);
        assert!(result.is_err());
    }

    #[test]
    fn given_node_parent_that_is_not_subgraph_when_validated_then_schema_fails() {
        let parent_id = NodeId::new(String::from("parent"));
        let child_id = NodeId::new(String::from("child"));

        let mut doc = DiagramDocument::default();
        doc.document.nodes = HashMap::new()
            .update(parent_id.clone(), node(NodeKind::Node, None))
            .update(child_id, node(NodeKind::Node, Some(parent_id)));

        assert!(validate_schema(&doc).is_err());
    }

    #[test]
    fn given_edge_with_missing_target_when_validated_then_schema_fails() {
        let a = NodeId::new(String::from("a"));
        let b = NodeId::new(String::from("b"));

        let mut doc = DiagramDocument::default();
        doc.document.nodes = HashMap::new().update(a.clone(), node(NodeKind::Node, None));
        doc.document.edges = HashMap::new().update(EdgeId::new(String::from("e1")), edge(&a, &b));

        assert!(validate_schema(&doc).is_err());
    }

    #[test]
    fn given_node_with_missing_parent_reference_when_validated_then_schema_fails() {
        let missing_parent = NodeId::new(String::from("missing-parent"));
        let child_id = NodeId::new(String::from("child"));

        let mut doc = DiagramDocument::default();
        doc.document.nodes =
            HashMap::new().update(child_id, node(NodeKind::Node, Some(missing_parent)));

        assert!(validate_schema(&doc).is_err());
    }

    #[test]
    fn given_node_with_existing_subgraph_parent_when_validated_then_schema_passes() {
        let parent_id = NodeId::new(String::from("parent"));
        let child_id = NodeId::new(String::from("child"));

        let mut doc = DiagramDocument::default();
        doc.document.nodes = HashMap::new()
            .update(parent_id.clone(), node(NodeKind::Subgraph, None))
            .update(child_id, node(NodeKind::Node, Some(parent_id)));

        assert!(validate_schema(&doc).is_ok());
    }

    // =============================================================================
    // SUB subgraph tests (bd-163) - Parent cycle prevention
    // =============================================================================

    #[test]
    fn given_circular_parent_chain_when_validated_then_schema_fails() {
        // Create a cycle: A -> B -> C -> A
        let a_id = NodeId::new(String::from("subgraph-a"));
        let b_id = NodeId::new(String::from("subgraph-b"));
        let c_id = NodeId::new(String::from("subgraph-c"));

        let mut doc = DiagramDocument::default();
        doc.document.nodes = HashMap::new()
            // A's parent is C
            .update(a_id.clone(), node(NodeKind::Subgraph, Some(c_id.clone())))
            // B's parent is A
            .update(b_id.clone(), node(NodeKind::Subgraph, Some(a_id.clone())))
            // C's parent is B
            .update(c_id, node(NodeKind::Subgraph, Some(b_id)));

        let result = validate_schema(&doc);
        assert!(result.is_err(), "circular parent chain should fail validation");

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.to_lowercase().contains("circular")
                || err_msg.to_lowercase().contains("cycle"),
            "error message should mention circular or cycle: {}",
            err_msg
        );
    }

    #[test]
    fn given_self_referential_parent_when_validated_then_schema_fails() {
        // A node that is its own parent
        let a_id = NodeId::new(String::from("subgraph-a"));

        let mut doc = DiagramDocument::default();
        doc.document.nodes = HashMap::new()
            .update(a_id.clone(), node(NodeKind::Subgraph, Some(a_id)));

        let result = validate_schema(&doc);
        assert!(result.is_err(), "self-referential parent should fail validation");
    }

    #[test]
    fn given_two_node_parent_cycle_when_validated_then_schema_fails() {
        // Create a 2-node cycle: A -> B -> A
        let a_id = NodeId::new(String::from("subgraph-a"));
        let b_id = NodeId::new(String::from("subgraph-b"));

        let mut doc = DiagramDocument::default();
        doc.document.nodes = HashMap::new()
            .update(a_id.clone(), node(NodeKind::Subgraph, Some(b_id.clone())))
            .update(b_id, node(NodeKind::Subgraph, Some(a_id)));

        let result = validate_schema(&doc);
        assert!(result.is_err(), "two-node parent cycle should fail validation");
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use crate::models::document::{
        ArrowType, DiagramDocument, DocumentData, Edge, EdgeId, EditorState, Node, NodeId,
        NodeKind, NodeStyle, OrderedFloat, Revision,
    };
    use crate::ui::grid::GridSize;
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
            any::<f64>().prop_map(OrderedFloat),
            Just(OrderedFloat(f64::NAN)),
            Just(OrderedFloat(f64::INFINITY)),
            Just(OrderedFloat(f64::NEG_INFINITY)),
            Just(OrderedFloat(0.0)),
            Just(OrderedFloat(f64::MIN)),
            Just(OrderedFloat(f64::MAX)),
        ]
    }

    fn make_node(kind: NodeKind, parent: Option<NodeId>, x: f64, y: f64) -> Node {
        Node {
            kind,
            icon: String::new(),
            label: String::new(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(100.0),
            height: OrderedFloat(60.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent,
            dag_rank: None,
            tags: vec![],
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
            style: crate::models::document::EdgeStyle::Solid,
            arrow_type: ArrowType::Default,
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.5),
            directed: true,
            bend_points: vec![],
            tags: vec![],
            metadata: HashMap::new(),
            font_size: None,
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
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

        #[test]
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
                    editing_edge_id: None,
                    theme: crate::models::document::EditorTheme::System,
                    show_grid: true,
                    minimap_visible: false,
                },
            };
            let result = validate_schema(&doc);
            prop_assert!(result.is_ok() || result.is_err());
        }

        #[test]
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

        #[test]
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

        #[test]
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

        #[test]
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

        #[test]
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

        #[test]
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

        #[test]
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

        #[test]
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

        #[test]
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

        #[test]
        fn prop_node_extreme_coordinates(
            node_id in arb_node_id(),
            x in any::<f64>(),
            y in any::<f64>(),
            width in any::<f64>(),
            height in any::<f64>(),
        ) {
            let mut node = make_node(NodeKind::Node, None, 0.0, 0.0);
            node.x = OrderedFloat(x);
            node.y = OrderedFloat(y);
            node.width = OrderedFloat(width);
            node.height = OrderedFloat(height);
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

        #[test]
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

        #[test]
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
}
