#[cfg(test)]
mod tests {
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

    fn make_node(x: f64, y: f64) -> Node {
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
            metadata: HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    fn make_locked_node(x: f64, y: f64) -> Node {
        Node {
            lock_state: LockState::Locked,
            ..make_node(x, y)
        }
    }

    fn make_edge(src: &NodeId, tgt: &NodeId) -> Edge {
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
            bend_points: im::Vector::new(),
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            font_size: None,
            source_port: None,
            target_port: None,
        }
    }

    fn empty_editor() -> EditorState {
        EditorState {
            snap_to_grid: false,
            ..EditorState::default()
        }
    }

    fn make_doc(nodes: Vec<(NodeId, Node)>, edges: Vec<(EdgeId, Edge)>) -> DiagramDocument {
        DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: nodes.into_iter().collect(),
                edges: edges.into_iter().collect(),
            },
            editor_state: empty_editor(),
        }
    }

    // ── Test 1: A→B→C sequential: A.x < B.x < C.x ──────────────────────────
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn sequential_dag_x_ordering() {
        let a = NodeId::new("A".to_string());
        let b = NodeId::new("B".to_string());
        let c = NodeId::new("C".to_string());

        let doc = make_doc(
            vec![
                (a.clone(), make_node(0.0, 0.0)),
                (b.clone(), make_node(0.0, 0.0)),
                (c.clone(), make_node(0.0, 0.0)),
            ],
            vec![
                (EdgeId::new("e1".to_string()), make_edge(&a, &b)),
                (EdgeId::new("e2".to_string()), make_edge(&b, &c)),
            ],
        );

        let result = dag_layout(&doc, &DagLayoutSettings::default());
        let get_x = |id| result.document.nodes.get(&id).map_or(0.0, |n| n.x.0);

        let (ax, bx, cx) = (get_x(&a), get_x(&b), get_x(&c));
        assert!(ax < bx, "A.x={ax} must be < B.x={bx}");
        assert!(bx < cx, "B.x={bx} must be < C.x={cx}");
    }

    // ── Test 2: No edges → no panic, all nodes present ──────────────────────
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn no_edges_no_panic() {
        let a = NodeId::new("A".to_string());
        let b = NodeId::new("B".to_string());

        let doc = make_doc(
            vec![(a, make_node(0.0, 0.0)), (b, make_node(0.0, 0.0))],
            vec![],
        );

        let result = dag_layout(&doc, &DagLayoutSettings::default());
        assert_eq!(result.document.nodes.len(), 2);
    }

    // ── Test 3: Cycle A→B→A falls back without panic ────────────────────────
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn cycle_fallback_no_panic() {
        let a = NodeId::new("A".to_string());
        let b = NodeId::new("B".to_string());

        let doc = make_doc(
            vec![
                (a.clone(), make_node(0.0, 0.0)),
                (b.clone(), make_node(0.0, 0.0)),
            ],
            vec![
                (EdgeId::new("e1".to_string()), make_edge(&a, &b)),
                (EdgeId::new("e2".to_string()), make_edge(&b, &a)),
            ],
        );

        let result = dag_layout(&doc, &DagLayoutSettings::default());
        assert_eq!(result.document.nodes.len(), 2);
    }

    // ── Test 4: Locked nodes are not moved ──────────────────────────────────
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn locked_nodes_unchanged() {
        let locked = NodeId::new("locked".to_string());
        let free = NodeId::new("free".to_string());

        let doc = make_doc(
            vec![
                (locked.clone(), make_locked_node(999.0, 888.0)),
                (free, make_node(0.0, 0.0)),
            ],
            vec![],
        );

        let result = dag_layout(&doc, &DagLayoutSettings::default());
        assert!(result.document.nodes.contains_key(&locked));
        let Some(ln) = result.document.nodes.get(&locked) else {
            return;
        };
        assert!(
            (ln.x.0 - 999.0).abs() < f64::EPSILON,
            "locked x must not change"
        );
        assert!(
            (ln.y.0 - 888.0).abs() < f64::EPSILON,
            "locked y must not change"
        );
    }

    // ── Test 5: Deterministic — two calls on same input produce same result ─
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn deterministic_output() {
        let a = NodeId::new("A".to_string());
        let b = NodeId::new("B".to_string());
        let c = NodeId::new("C".to_string());

        let doc = make_doc(
            vec![
                (a.clone(), make_node(0.0, 0.0)),
                (b.clone(), make_node(0.0, 0.0)),
                (c.clone(), make_node(0.0, 0.0)),
            ],
            vec![
                (EdgeId::new("e1".to_string()), make_edge(&a, &b)),
                (EdgeId::new("e2".to_string()), make_edge(&b, &c)),
            ],
        );

        let r1 = dag_layout(&doc, &DagLayoutSettings::default());
        let r2 = dag_layout(&doc, &DagLayoutSettings::default());

        let get_xy = |r: &DiagramDocument, id| r.document.nodes.get(&id).map(|n| (n.x.0, n.y.0));

        assert_eq!(get_xy(&r1, &a), get_xy(&r2, &a), "A must be deterministic");
        assert_eq!(get_xy(&r1, &b), get_xy(&r2, &b), "B must be deterministic");
        assert_eq!(get_xy(&r1, &c), get_xy(&r2, &c), "C must be deterministic");
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn nested_children_follow_ancestor_delta() {
        let root = NodeId::new("root".to_string());
        let child = NodeId::new("child".to_string());
        let grandchild = NodeId::new("grandchild".to_string());

        let mut child_node = make_node(10.0, 10.0);
        child_node.parent = Some(root.clone());

        let mut grandchild_node = make_node(20.0, 20.0);
        grandchild_node.parent = Some(child.clone());

        let doc = make_doc(
            vec![
                (root.clone(), make_node(0.0, 0.0)),
                (child.clone(), child_node),
                (grandchild.clone(), grandchild_node),
            ],
            vec![],
        );

        let result = dag_layout(&doc, &DagLayoutSettings::default());

        let root_before = doc
            .document
            .nodes
            .get(&root)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let root_after = result
            .document
            .nodes
            .get(&root)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let delta = (root_after.0 - root_before.0, root_after.1 - root_before.1);

        let grandchild_before = doc
            .document
            .nodes
            .get(&grandchild)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let grandchild_after = result
            .document
            .nodes
            .get(&grandchild)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));

        assert!((grandchild_after.0 - (grandchild_before.0 + delta.0)).abs() < f64::EPSILON);
        assert!((grandchild_after.1 - (grandchild_before.1 + delta.1)).abs() < f64::EPSILON);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_two_unconnected_nodes_when_dag_layout_then_nodes_are_centered_in_single_layer() {
        let a = NodeId::new("A".to_string());
        let b = NodeId::new("B".to_string());
        let doc = make_doc(
            vec![
                (a.clone(), make_node(0.0, 0.0)),
                (b.clone(), make_node(0.0, 0.0)),
            ],
            vec![],
        );

        let result = dag_layout(&doc, &DagLayoutSettings::default());
        let a_pos = result
            .document
            .nodes
            .get(&a)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let b_pos = result
            .document
            .nodes
            .get(&b)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));

        let y_values = [a_pos.1, b_pos.1];

        assert_eq!(a_pos.0, 120.0);
        assert_eq!(b_pos.0, 120.0);
        assert!(y_values.contains(&80.0));
        assert!(y_values.contains(&208.0));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_single_edge_when_dag_layout_then_layer_spacing_and_padding_are_applied() {
        let a = NodeId::new("A".to_string());
        let b = NodeId::new("B".to_string());
        let doc = make_doc(
            vec![
                (a.clone(), make_node(0.0, 0.0)),
                (b.clone(), make_node(0.0, 0.0)),
            ],
            vec![(EdgeId::new("e1".to_string()), make_edge(&a, &b))],
        );

        let result = dag_layout(&doc, &DagLayoutSettings::default());
        let a_pos = result
            .document
            .nodes
            .get(&a)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let b_pos = result
            .document
            .nodes
            .get(&b)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));

        assert_eq!(a_pos, (120.0, 80.0));
        assert_eq!(b_pos, (480.0, 80.0));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_node_without_neighbors_when_barycentre_computed_then_result_is_max() {
        let mut graph = DiGraph::<NodeId, ()>::new();
        let isolated = graph.add_node(NodeId::new(String::from("isolated")));
        let ref_pos = std::collections::HashMap::new();

        let value = barycentre(isolated, &ref_pos, &graph, petgraph::Direction::Incoming);
        assert_eq!(value, f64::MAX);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_neighbor_positions_when_barycentre_computed_then_mean_is_used() {
        let mut graph = DiGraph::<NodeId, ()>::new();
        let a = graph.add_node(NodeId::new(String::from("a")));
        let b = graph.add_node(NodeId::new(String::from("b")));
        let c = graph.add_node(NodeId::new(String::from("c")));
        graph.add_edge(a, c, ());
        graph.add_edge(b, c, ());

        let ref_pos = std::collections::HashMap::from([(a, 2.0), (b, 6.0)]);
        let value = barycentre(c, &ref_pos, &graph, petgraph::Direction::Incoming);
        assert_eq!(value, 4.0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_mixed_layer_sizes_when_assigning_coordinates_then_shorter_layers_are_centered() {
        let mut graph = DiGraph::<NodeId, ()>::new();
        let n0 = graph.add_node(NodeId::new(String::from("n0")));
        let n1 = graph.add_node(NodeId::new(String::from("n1")));
        let n2 = graph.add_node(NodeId::new(String::from("n2")));
        let layers = vec![vec![n0, n1], vec![n2]];

        let coords = assign_coordinates(&layers, &DagLayoutSettings::default());
        let y0 = coords.get(&n0).map_or(0.0, |(_, y)| *y);
        let y1 = coords.get(&n1).map_or(0.0, |(_, y)| *y);
        let y2 = coords.get(&n2).map_or(0.0, |(_, y)| *y);

        assert_eq!(y0, 80.0);
        assert_eq!(y1, 208.0);
        assert_eq!(y2, 144.0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_ancestor_deltas_when_applying_position_then_deltas_are_accumulated() {
        let root_id = NodeId::new(String::from("root"));
        let child_id = NodeId::new(String::from("child"));
        let grandchild_id = NodeId::new(String::from("grandchild"));

        let mut child = make_node(5.0, 10.0);
        child.parent = Some(root_id.clone());
        let mut grandchild = make_node(20.0, 30.0);
        grandchild.parent = Some(child_id.clone());

        let all_nodes = HashMap::new()
            .update(root_id.clone(), make_node(0.0, 0.0))
            .update(child_id.clone(), child)
            .update(grandchild_id.clone(), grandchild.clone());

        let deltas = HashMap::new()
            .update(root_id, (10.0, 20.0))
            .update(child_id, (5.0, 7.0));

        let moved = apply_position(
            &grandchild_id,
            &grandchild,
            &HashMap::new(),
            &deltas,
            &all_nodes,
        );
        assert_eq!(moved.x.0, 35.0);
        assert_eq!(moved.y.0, 57.0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_nonzero_root_origin_when_dag_layout_runs_then_child_follows_exact_root_delta() {
        let root = NodeId::new(String::from("root"));
        let child = NodeId::new(String::from("child"));

        let mut child_node = make_node(70.0, 90.0);
        child_node.parent = Some(root.clone());

        let doc = make_doc(
            vec![
                (root.clone(), make_node(50.0, 70.0)),
                (child.clone(), child_node),
            ],
            vec![],
        );

        let result = dag_layout(&doc, &DagLayoutSettings::default());
        let root_before = doc
            .document
            .nodes
            .get(&root)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let root_after = result
            .document
            .nodes
            .get(&root)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let child_before = doc
            .document
            .nodes
            .get(&child)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let child_after = result
            .document
            .nodes
            .get(&child)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let delta = (root_after.0 - root_before.0, root_after.1 - root_before.1);

        assert_eq!(child_after.0, child_before.0 + delta.0);
        assert_eq!(child_after.1, child_before.1 + delta.1);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_crossed_two_layer_order_when_swept_then_barycenter_reorders_layer() {
        let mut graph = DiGraph::<NodeId, ()>::new();
        let a = graph.add_node(NodeId::new(String::from("a")));
        let b = graph.add_node(NodeId::new(String::from("b")));
        let c = graph.add_node(NodeId::new(String::from("c")));
        let d = graph.add_node(NodeId::new(String::from("d")));

        graph.add_edge(a, d, ());
        graph.add_edge(b, c, ());

        let layers = vec![vec![a, b], vec![c, d]];
        let swept = barycenter_sweep(layers, &graph);

        assert_eq!(swept.len(), 2);
        assert_eq!(swept[0], vec![a, b]);
        assert_eq!(swept[1], vec![d, c]);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_multi_layer_graph_when_swept_then_matches_reference_sweep_order() {
        fn barycentre_ref(
            node: NodeIndex,
            ref_pos: &std::collections::HashMap<NodeIndex, f64>,
            graph: &DiGraph<NodeId, ()>,
            dir: petgraph::Direction,
        ) -> f64 {
            let neighbors = graph
                .neighbors_directed(node, dir)
                .filter_map(|n| ref_pos.get(&n).copied())
                .collect::<Vec<_>>();

            if neighbors.is_empty() {
                f64::MAX
            } else {
                neighbors.iter().sum::<f64>() / neighbors.len() as f64
            }
        }

        fn reference_sweep(
            layers: Vec<Vec<NodeIndex>>,
            graph: &DiGraph<NodeId, ()>,
        ) -> Vec<Vec<NodeIndex>> {
            (0..4_u8).fold(layers, |acc, sweep| {
                let n = acc.len();
                if sweep % 2 == 0 {
                    (1..n).fold(acc, |mut ls, l| {
                        let ref_pos = ls[l - 1]
                            .iter()
                            .enumerate()
                            .map(|(i, &node)| (node, i as f64))
                            .collect::<std::collections::HashMap<_, _>>();
                        ls[l].sort_by(|&a, &b| {
                            barycentre_ref(a, &ref_pos, graph, petgraph::Direction::Incoming)
                                .partial_cmp(&barycentre_ref(
                                    b,
                                    &ref_pos,
                                    graph,
                                    petgraph::Direction::Incoming,
                                ))
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });
                        ls
                    })
                } else {
                    (0..n.saturating_sub(1)).rev().fold(acc, |mut ls, l| {
                        let ref_pos = ls[l + 1]
                            .iter()
                            .enumerate()
                            .map(|(i, &node)| (node, i as f64))
                            .collect::<std::collections::HashMap<_, _>>();
                        ls[l].sort_by(|&a, &b| {
                            barycentre_ref(a, &ref_pos, graph, petgraph::Direction::Outgoing)
                                .partial_cmp(&barycentre_ref(
                                    b,
                                    &ref_pos,
                                    graph,
                                    petgraph::Direction::Outgoing,
                                ))
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });
                        ls
                    })
                }
            })
        }

        let mut graph = DiGraph::<NodeId, ()>::new();
        let node_a = graph.add_node(NodeId::new(String::from("a")));
        let node_b = graph.add_node(NodeId::new(String::from("b")));
        let node_c = graph.add_node(NodeId::new(String::from("c")));
        let node_d = graph.add_node(NodeId::new(String::from("d")));
        let node_e = graph.add_node(NodeId::new(String::from("e")));
        let node_f = graph.add_node(NodeId::new(String::from("f")));

        graph.add_edge(node_a, node_c, ());
        graph.add_edge(node_a, node_d, ());
        graph.add_edge(node_b, node_d, ());
        graph.add_edge(node_b, node_e, ());
        graph.add_edge(node_c, node_f, ());
        graph.add_edge(node_e, node_f, ());

        let layers = vec![
            vec![node_a, node_b],
            vec![node_e, node_d, node_c],
            vec![node_f],
        ];
        let expected = reference_sweep(layers.clone(), &graph);
        let actual = barycenter_sweep(layers, &graph);

        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod proptests {
    use crate::layout::dag::apply::apply_position;
    use crate::layout::dag::crossing::{barycenter_sweep, barycentre};
    use crate::layout::dag::positioning::{assign_coordinates, NODE_HEIGHT, NODE_WIDTH};
    use crate::layout::dag::{dag_layout, DagLayoutSettings};
    use diagram_models::document::{
        ArrowType, DiagramDocument, DocumentData, Edge, EdgeId, EditorState, LockState, NodeKind,
        NodeStyle, Revision,
    };
    use diagram_models::document::{Node, NodeId, OrderedFloat};
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

    fn make_doc_for_prop(
        nodes: Vec<(NodeId, Node)>,
        edges: Vec<(EdgeId, Edge)>,
    ) -> DiagramDocument {
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
}
