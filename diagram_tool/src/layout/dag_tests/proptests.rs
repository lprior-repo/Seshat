use crate::layout::dag::apply::apply_position;
use crate::layout::dag::crossing::{barycenter_sweep, barycentre};
use crate::layout::dag::positioning::{assign_coordinates, NODE_HEIGHT, NODE_WIDTH};
use crate::layout::dag::{dag_layout, DagLayoutSettings};
use diagram_models::document::{
    ArrowType, DiagramDocument, DocumentData, Edge, EdgeId, EditorState, LockState, NodeKind,
    NodeStyle, Revision,
};
use diagram_models::document::{Node, NodeId, OrderedFloat};
use im::HashMap;
use petgraph::graph::{DiGraph, NodeIndex};

use proptest::prelude::*;

fn make_node_for_prop(x: f64, y: f64) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: String::new(),
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        width: OrderedFloat(220.0),
        height: OrderedFloat(68.0),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent: None,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: im::HashMap::new(),
        z_index: 0,
        style: Some(NodeStyle::default()),
        collapsed: None,
    }
}

fn make_edge_for_prop(src: &NodeId, tgt: &NodeId) -> Edge {
    Edge {
        source: src.clone(),
        target: tgt.clone(),
        label: String::new(),
        style: diagram_models::document::EdgeStyle::Solid,
        arrow_type: ArrowType::Default,
        label_offset_t: OrderedFloat(0.5),
        color: None,
        thickness: OrderedFloat(1.5),
        directed: true,
        bend_points: im::vector![],
        tags: im::vector![],
        metadata: im::HashMap::new(),
        font_size: None,
        source_port: None,
        target_port: None,
    }
}

fn make_doc_for_prop(nodes: Vec<(NodeId, Node)>, edges: Vec<(EdgeId, Edge)>) -> DiagramDocument {
    DiagramDocument {
        version: 2,
        revision: Revision::INITIAL,
        document: DocumentData {
            nodes: nodes.into_iter().collect(),
            edges: edges.into_iter().collect(),
        },
        editor_state: EditorState::default(),
    }
}

prop_compose! {
    fn arb_dag_layout_settings()(
        layer_spacing in 1.0..1000.0f64,
        node_spacing in 1.0..500.0f64,
    ) -> DagLayoutSettings {
        DagLayoutSettings { layer_spacing, node_spacing }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    #[allow(clippy::unwrap_used)]
    fn prop_all_coordinates_finite(
        node_count in 1usize..20,
        settings in arb_dag_layout_settings(),
    ) {
        let nodes: Vec<(NodeId, Node)> = (0..node_count)
            .map(|i| (NodeId::new(format!("n{i}")), make_node_for_prop(0.0, 0.0)))
            .collect();

        let doc = make_doc_for_prop(nodes, vec![]);
        let result = dag_layout(&doc, &settings);

        for node in result.document.nodes.values() {
            prop_assert!(node.x.0.is_finite(), "x coordinate must be finite");
            prop_assert!(node.y.0.is_finite(), "y coordinate must be finite");
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    #[allow(clippy::unwrap_used)]
    fn prop_layer_ordering_respected(
        edge_count in 0usize..10,
        settings in arb_dag_layout_settings(),
    ) {
        let a = NodeId::new("a".to_string());
        let b = NodeId::new("b".to_string());
        let c = NodeId::new("c".to_string());

        let mut edges: Vec<(EdgeId, Edge)> = vec![];
        if edge_count % 3 == 0 {
            edges.push((EdgeId::new("e1".to_string()), make_edge_for_prop(&a, &b)));
        }
        if edge_count % 3 == 1 {
            edges.push((EdgeId::new("e2".to_string()), make_edge_for_prop(&b, &c)));
        }
        if edge_count % 3 == 2 {
            edges.push((EdgeId::new("e3".to_string()), make_edge_for_prop(&a, &c)));
        }

        let doc = make_doc_for_prop(
            vec![
                (a.clone(), make_node_for_prop(0.0, 0.0)),
                (b.clone(), make_node_for_prop(0.0, 0.0)),
                (c.clone(), make_node_for_prop(0.0, 0.0)),
            ],
            edges,
        );

        let result = dag_layout(&doc, &settings);

        let get_x = |id| result.document.nodes.get(&id).map_or(0.0, |n| n.x.0);

        if result.document.edges.contains_key(&EdgeId::new("e1".to_string())) {
            let (ax, bx) = (get_x(&a), get_x(&b));
            prop_assert!(ax <= bx, "a.x={ax} must be <= b.x={bx} when a→b");
        }
        if result.document.edges.contains_key(&EdgeId::new("e2".to_string())) {
            let (bx, cx) = (get_x(&b), get_x(&c));
            prop_assert!(bx <= cx, "b.x={bx} must be <= c.x={cx} when b→c");
        }
        if result.document.edges.contains_key(&EdgeId::new("e3".to_string())) {
            let (ax, cx) = (get_x(&a), get_x(&c));
            prop_assert!(ax <= cx, "a.x={ax} must be <= c.x={cx} when a→c");
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    #[allow(clippy::unwrap_used)]
    fn prop_no_node_overlap(
        node_count in 2usize..15,
        settings in arb_dag_layout_settings(),
    ) {
        let nodes: Vec<(NodeId, Node)> = (0..node_count)
            .map(|i| (NodeId::new(format!("n{i}")), make_node_for_prop(0.0, 0.0)))
            .collect();

        let doc = make_doc_for_prop(nodes, vec![]);
        let result = dag_layout(&doc, &settings);

        let positions: Vec<(&NodeId, f64, f64)> = result
            .document
            .nodes
            .iter()
            .filter(|(_, n)| !n.lock_state.is_locked() && n.parent.is_none())
            .map(|(id, n)| (id, n.x.0, n.y.0))
            .collect();

        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                let (_, x1, y1) = positions[i];
                let (_, x2, y2) = positions[j];
                let dx = (x1 - x2).abs();
                let dy = (y1 - y2).abs();
                let overlapping = dx < NODE_WIDTH && dy < NODE_HEIGHT;
                prop_assert!(!overlapping, "nodes at ({x1},{y1}) and ({x2},{y2}) overlap");
            }
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    #[allow(clippy::unwrap_used)]
    fn prop_edge_endpoints_valid(
        node_count in 2usize..10,
        edge_count in 0usize..9,
        settings in arb_dag_layout_settings(),
    ) {
        let node_ids: Vec<NodeId> = (0..node_count)
            .map(|i| NodeId::new(format!("n{i}")))
            .collect();

        let nodes: Vec<(NodeId, Node)> = node_ids
            .iter()
            .map(|id| (id.clone(), make_node_for_prop(0.0, 0.0)))
            .collect();

        let edges: Vec<(EdgeId, Edge)> = (0..edge_count.min(node_count - 1))
            .map(|i| {
                let src = node_ids[i].clone();
                let tgt = node_ids[i + 1].clone();
                (EdgeId::new(format!("e{i}")), make_edge_for_prop(&src, &tgt))
            })
            .collect();

        let doc = make_doc_for_prop(nodes, edges.clone());
        let result = dag_layout(&doc, &settings);

        for (edge_id, edge) in &result.document.edges {
            prop_assert!(
                result.document.nodes.contains_key(&edge.source),
                "edge {edge_id:?} source must exist"
            );
            prop_assert!(
                result.document.nodes.contains_key(&edge.target),
                "edge {edge_id:?} target must exist"
            );
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    #[allow(clippy::unwrap_used)]
    fn prop_extreme_coordinates_no_panic(
        x in f64::MIN..f64::MAX,
        y in f64::MIN..f64::MAX,
        settings in arb_dag_layout_settings(),
    ) {
        let x = if x.is_nan() { 0.0 } else { x };
        let y = if y.is_nan() { 0.0 } else { y };

        let doc = make_doc_for_prop(
            vec![(NodeId::new("n".to_string()), make_node_for_prop(x, y))],
            vec![],
        );

        let result = dag_layout(&doc, &settings);

        for node in result.document.nodes.values() {
            if !node.lock_state.is_locked() {
                prop_assert!(node.x.0.is_finite(), "result x must be finite");
                prop_assert!(node.y.0.is_finite(), "result y must be finite");
            }
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    #[allow(clippy::unwrap_used)]
    fn prop_node_count_preserved(
        node_count in 0usize..50,
        settings in arb_dag_layout_settings(),
    ) {
        let nodes: Vec<(NodeId, Node)> = (0..node_count)
            .map(|i| (NodeId::new(format!("n{i}")), make_node_for_prop(0.0, 0.0)))
            .collect();

        let doc = make_doc_for_prop(nodes, vec![]);
        let result = dag_layout(&doc, &settings);

        prop_assert_eq!(result.document.nodes.len(), node_count);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    #[allow(clippy::unwrap_used)]
    fn prop_edge_count_preserved(
        node_count in 2usize..20,
        edge_count in 0usize..19,
        settings in arb_dag_layout_settings(),
    ) {
        let node_ids: Vec<NodeId> = (0..node_count)
            .map(|i| NodeId::new(format!("n{i}")))
            .collect();

        let nodes: Vec<(NodeId, Node)> = node_ids
            .iter()
            .map(|id| (id.clone(), make_node_for_prop(0.0, 0.0)))
            .collect();

        let edges: Vec<(EdgeId, Edge)> = (0..edge_count.min(node_count - 1))
            .map(|i| {
                let src = node_ids[i % node_ids.len()].clone();
                let tgt = node_ids[(i + 1) % node_ids.len()].clone();
                (EdgeId::new(format!("e{i}")), make_edge_for_prop(&src, &tgt))
            })
            .collect();

        let doc = make_doc_for_prop(nodes, edges.clone());
        let result = dag_layout(&doc, &settings);

        prop_assert_eq!(result.document.edges.len(), edges.len());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    #[allow(clippy::unwrap_used)]
    fn prop_empty_document_no_panic(settings in arb_dag_layout_settings()) {
        let doc = make_doc_for_prop(vec![], vec![]);
        let result = dag_layout(&doc, &settings);
        prop_assert!(result.document.nodes.is_empty());
        prop_assert!(result.document.edges.is_empty());
    }
}
