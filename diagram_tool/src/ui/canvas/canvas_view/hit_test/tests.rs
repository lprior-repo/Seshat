#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
use im::HashMap;

use super::edge::*;
use diagram_models::document::{
    ArrowType, DiagramDocument, DocumentData, Edge, EdgeId, EdgeStyle, LockState, Node, NodeId,
    NodeKind, NodeStyle, OrderedFloat,
};

#[cfg(test)]
mod core_tests {
    use super::*;

    fn node_at(x: f64, y: f64) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: String::new(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(10.0),
            height: OrderedFloat(10.0),
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

    fn edge(source: NodeId, target: NodeId) -> Edge {
        Edge {
            source,
            target,
            label: String::new(),
            style: EdgeStyle::Solid,
            arrow_type: ArrowType::Straight,
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

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_low_zoom_when_clicking_near_edge_then_hit_test_uses_screen_consistent_radius() {
        let source_id = NodeId::new(String::from("source"));
        let target_id = NodeId::new(String::from("target"));
        let edge_id = EdgeId::new(String::from("e1"));

        let mut doc = DiagramDocument {
            document: DocumentData {
                nodes: HashMap::new()
                    .update(source_id.clone(), node_at(0.0, 0.0))
                    .update(target_id.clone(), node_at(100.0, 0.0)),
                edges: HashMap::new().update(edge_id.clone(), edge(source_id, target_id)),
            },
            ..DiagramDocument::default()
        };
        doc.editor_state.zoom = OrderedFloat(0.5);

        let hit = find_edge_at(&doc, 50.0, 17.0);
        assert_eq!(hit, Some(edge_id));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_high_zoom_when_clicking_same_world_distance_then_hit_test_is_tighter() {
        let source_id = NodeId::new(String::from("source"));
        let target_id = NodeId::new(String::from("target"));
        let edge_id = EdgeId::new(String::from("e1"));

        let mut doc = DiagramDocument {
            document: DocumentData {
                nodes: HashMap::new()
                    .update(source_id.clone(), node_at(0.0, 0.0))
                    .update(target_id.clone(), node_at(100.0, 0.0)),
                edges: HashMap::new().update(edge_id, edge(source_id, target_id)),
            },
            ..DiagramDocument::default()
        };
        doc.editor_state.zoom = OrderedFloat(2.0);

        let hit = find_edge_at(&doc, 50.0, 17.0);
        assert!(hit.is_none());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_overlapping_edges_when_hit_distance_ties_then_selection_is_stable_by_edge_id() {
        let source_id = NodeId::new(String::from("source"));
        let target_id = NodeId::new(String::from("target"));
        let edge_a = EdgeId::new(String::from("edge-a"));
        let edge_b = EdgeId::new(String::from("edge-b"));

        let doc = DiagramDocument {
            document: DocumentData {
                nodes: HashMap::new()
                    .update(source_id.clone(), node_at(0.0, 0.0))
                    .update(target_id.clone(), node_at(100.0, 0.0)),
                edges: HashMap::new()
                    .update(edge_b.clone(), edge(source_id.clone(), target_id.clone()))
                    .update(edge_a.clone(), edge(source_id, target_id)),
            },
            ..DiagramDocument::default()
        };

        let hit = find_edge_at(&doc, 50.0, 5.0);
        assert_eq!(hit, Some(edge_a));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_click_near_arrow_endpoint_when_within_endpoint_radius_then_edge_is_hit() {
        let source_id = NodeId::new(String::from("source"));
        let target_id = NodeId::new(String::from("target"));
        let edge_id = EdgeId::new(String::from("e1"));

        let doc = DiagramDocument {
            document: DocumentData {
                nodes: HashMap::new()
                    .update(source_id.clone(), node_at(0.0, 0.0))
                    .update(target_id.clone(), node_at(100.0, 0.0)),
                edges: HashMap::new().update(edge_id.clone(), edge(source_id, target_id)),
            },
            ..DiagramDocument::default()
        };

        let hit = find_edge_at(&doc, 109.0, 12.0);
        assert_eq!(hit, Some(edge_id));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_thin_vertical_edge_when_clicking_near_segment_then_hit_is_stable_across_zooms() {
        let source_id = NodeId::new(String::from("source"));
        let target_id = NodeId::new(String::from("target"));
        let edge_id = EdgeId::new(String::from("e1"));

        let mut doc = DiagramDocument {
            document: DocumentData {
                nodes: HashMap::new()
                    .update(source_id.clone(), node_at(40.0, 0.0))
                    .update(target_id.clone(), node_at(40.0, 120.0)),
                edges: HashMap::new().update(edge_id.clone(), edge(source_id, target_id)),
            },
            ..DiagramDocument::default()
        };

        for zoom in [0.5_f64, 1.0_f64, 2.0_f64, 3.0_f64] {
            doc.editor_state.zoom = OrderedFloat(zoom);
            let hit = find_edge_at(&doc, 47.0, 65.0);
            assert_eq!(hit, Some(edge_id.clone()));
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_endpoint_tie_when_clicking_shared_target_then_selection_is_stable_by_edge_id() {
        let source_a = NodeId::new(String::from("source-a"));
        let source_b = NodeId::new(String::from("source-b"));
        let target = NodeId::new(String::from("target"));
        let edge_a = EdgeId::new(String::from("edge-a"));
        let edge_b = EdgeId::new(String::from("edge-b"));

        let doc = DiagramDocument {
            document: DocumentData {
                nodes: HashMap::new()
                    .update(source_a.clone(), node_at(0.0, 0.0))
                    .update(source_b.clone(), node_at(0.0, 100.0))
                    .update(target.clone(), node_at(100.0, 0.0)),
                edges: HashMap::new()
                    .update(edge_b, edge(source_b, target.clone()))
                    .update(edge_a.clone(), edge(source_a, target)),
            },
            ..DiagramDocument::default()
        };

        let hit = find_edge_at(&doc, 105.0, 5.0);
        assert_eq!(hit, Some(edge_a));
    }
}

// =============================================================================
// SEL-002 Edge Selection Tests
// =============================================================================

/// Tests for SEL-002: Select single edge by clicking
/// Contract: .beads/task-sel-002/contract.md
#[cfg(test)]
mod sel_002_edge_selection_tests {
    use super::*;

    /// Simulate selecting a single edge by updating the document's `selected_items`
    fn select_single_edge(doc: &mut DiagramDocument, edge_id: EdgeId) {
        doc.editor_state.selected_items = std::iter::once(edge_id.to_string()).collect();
    }

    /// Helper to create a node at the given position
    fn node_at(x: f64, y: f64) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: String::new(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(10.0),
            height: OrderedFloat(10.0),
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

    /// Helper to create an edge between two nodes
    fn edge(source: NodeId, target: NodeId) -> Edge {
        Edge {
            source,
            target,
            label: String::new(),
            style: EdgeStyle::Solid,
            arrow_type: ArrowType::Straight,
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

    /// Create a test document with two nodes and one edge between them
    fn create_document_with_edge() -> DiagramDocument {
        let source_id = NodeId::new(String::from("node-a"));
        let target_id = NodeId::new(String::from("node-b"));
        let edge_id = EdgeId::new(String::from("edge-1"));

        DiagramDocument {
            document: DocumentData {
                nodes: HashMap::new()
                    .update(source_id.clone(), node_at(0.0, 0.0))
                    .update(target_id.clone(), node_at(100.0, 0.0)),
                edges: HashMap::new().update(edge_id, edge(source_id, target_id)),
            },
            ..DiagramDocument::default()
        }
    }

    // =========================================================================
    // Happy Path Tests
    // =========================================================================

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_sel_002_given_document_with_two_nodes_and_edge_when_clicking_edge_then_edge_is_selected(
    ) {
        let mut doc = create_document_with_edge();

        let hit = find_edge_at(&doc, 50.0, 0.0);

        if let Some(edge_id) = hit {
            assert_eq!(edge_id.as_str(), "edge-1");

            select_single_edge(&mut doc, edge_id.clone());
            assert_eq!(doc.editor_state.selected_items.len(), 1);
            assert!(doc
                .editor_state
                .selected_items
                .contains(&String::from("edge-1")));
        } else {
            panic!("Expected to find edge at click position");
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_sel_002_given_document_with_edge_when_clicking_at_edge_center_then_edge_selected() {
        let doc = create_document_with_edge();

        let hit = find_edge_at(&doc, 50.0, 0.0);
        if let Some(edge_id) = hit {
            assert_eq!(edge_id.as_str(), "edge-1");
        } else {
            panic!("Expected to find edge at center");
        }
    }

    // =========================================================================
    // Error Path Tests
    // =========================================================================

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_sel_002_given_empty_document_when_clicking_then_no_edge_selected() {
        let doc = DiagramDocument::default();
        let hit = find_edge_at(&doc, 50.0, 50.0);
        assert!(hit.is_none(), "Expected no edge for empty document");
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_sel_002_given_document_with_edge_when_clicking_far_from_edge_then_no_edge_selected() {
        let doc = create_document_with_edge();
        let hit = find_edge_at(&doc, 500.0, 500.0);
        assert!(
            hit.is_none(),
            "Expected no edge when clicking far from edge"
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_sel_002_given_document_when_clicking_with_nan_coordinates_then_no_edge_selected() {
        let doc = create_document_with_edge();
        let hit_nan = find_edge_at(&doc, f64::NAN, 0.0);
        let hit_inf = find_edge_at(&doc, f64::INFINITY, 0.0);
        assert!(hit_nan.is_none(), "Expected no edge for NaN coordinates");
        assert!(
            hit_inf.is_none(),
            "Expected no edge for Infinity coordinates"
        );
    }

    // =========================================================================
    // Edge Case Tests
    // =========================================================================

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_sel_002_given_horizontal_edge_when_clicking_at_endpoint_then_edge_selected() {
        let doc = create_document_with_edge();
        let hit = find_edge_at(&doc, 0.0, 0.0);
        if let Some(edge_id) = hit {
            assert_eq!(edge_id.as_str(), "edge-1");
        } else {
            panic!("Expected edge at endpoint");
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_sel_002_given_vertical_edge_when_clicking_along_edge_then_edge_selected() {
        let source_id = NodeId::new(String::from("source"));
        let target_id = NodeId::new(String::from("target"));
        let edge_id = EdgeId::new(String::from("vertical-edge"));

        let doc = DiagramDocument {
            document: DocumentData {
                nodes: HashMap::new()
                    .update(source_id.clone(), node_at(40.0, 0.0))
                    .update(target_id.clone(), node_at(40.0, 100.0)),
                edges: HashMap::new().update(edge_id.clone(), edge(source_id, target_id)),
            },
            ..DiagramDocument::default()
        };

        let hit = find_edge_at(&doc, 40.0, 50.0);
        if let Some(found_edge_id) = hit {
            assert_eq!(found_edge_id, edge_id);
        } else {
            panic!("Expected to find vertical edge");
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_sel_002_given_diagonal_edge_when_clicking_along_edge_then_edge_selected() {
        let source_id = NodeId::new(String::from("source"));
        let target_id = NodeId::new(String::from("target"));
        let edge_id = EdgeId::new(String::from("diagonal-edge"));

        let doc = DiagramDocument {
            document: DocumentData {
                nodes: HashMap::new()
                    .update(source_id.clone(), node_at(0.0, 0.0))
                    .update(target_id.clone(), node_at(100.0, 100.0)),
                edges: HashMap::new().update(edge_id.clone(), edge(source_id, target_id)),
            },
            ..DiagramDocument::default()
        };

        for (x, y) in [(25.0, 25.0), (50.0, 50.0), (75.0, 75.0)] {
            let hit = find_edge_at(&doc, x, y);
            if let Some(found_edge_id) = hit {
                assert_eq!(found_edge_id, edge_id, "Wrong edge at ({}, {})", x, y);
            } else {
                panic!("Expected edge at ({}, {})", x, y);
            }
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_sel_002_given_edge_with_bend_points_when_clicking_on_bend_then_edge_selected() {
        use diagram_models::document::Point;

        let source_id = NodeId::new(String::from("source"));
        let target_id = NodeId::new(String::from("target"));
        let edge_id = EdgeId::new(String::from("bent-edge"));

        let mut bend_points = im::Vector::new();
        bend_points.push_back(Point {
            x: OrderedFloat(50.0),
            y: OrderedFloat(50.0),
        });

        let edge_with_bend = Edge {
            source: source_id.clone(),
            target: target_id.clone(),
            label: String::new(),
            style: EdgeStyle::Solid,
            arrow_type: ArrowType::Straight,
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.5),
            directed: true,
            bend_points,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            font_size: None,
            source_port: None,
            target_port: None,
        };

        let doc = DiagramDocument {
            document: DocumentData {
                nodes: HashMap::new()
                    .update(source_id.clone(), node_at(0.0, 0.0))
                    .update(target_id.clone(), node_at(100.0, 100.0)),
                edges: HashMap::new().update(edge_id.clone(), edge_with_bend),
            },
            ..DiagramDocument::default()
        };

        let hit = find_edge_at(&doc, 50.0, 50.0);
        if let Some(found_edge_id) = hit {
            assert_eq!(found_edge_id, edge_id, "Expected bent edge to be selected");
        } else {
            panic!("Expected to find edge when clicking on bend point");
        }
    }

    // =========================================================================
    // Contract Verification Tests
    // =========================================================================

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_precondition_p1_document_contains_edge() {
        let doc = create_document_with_edge();
        assert!(
            doc.document
                .edges
                .contains_key(&EdgeId::new(String::from("edge-1"))),
            "P1: Document must contain edge"
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_precondition_p4_coordinates_finite() {
        let doc = create_document_with_edge();
        let valid_hit = find_edge_at(&doc, 50.0, 0.0);
        assert!(
            valid_hit.is_some(),
            "P4: Valid coordinates should find edge"
        );
        assert!(
            find_edge_at(&doc, f64::NAN, 0.0).is_none(),
            "P4: NaN should not find edge"
        );
        assert!(
            find_edge_at(&doc, 0.0, f64::INFINITY).is_none(),
            "P4: Infinity should not find edge"
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_postcondition_q1_selection_count_exactly_one() {
        let mut doc = create_document_with_edge();
        let hit = if let Some(h) = find_edge_at(&doc, 50.0, 0.0) {
            h
        } else {
            panic!("Expected to find edge at click position");
        };

        select_single_edge(&mut doc, hit.clone());
        assert_eq!(
            doc.editor_state.selected_items.len(),
            1,
            "Q1: Selection should contain exactly one item"
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_postcondition_q2_selection_contains_edge_id() {
        let mut doc = create_document_with_edge();
        let hit = if let Some(h) = find_edge_at(&doc, 50.0, 0.0) {
            h
        } else {
            panic!("Expected to find edge at click position");
        };

        select_single_edge(&mut doc, hit.clone());
        assert!(
            doc.editor_state.selected_items.contains(&hit.to_string()),
            "Q2: Selected items should contain the exact edge ID"
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_invariant_i1_selection_contains_valid_ids() {
        let mut doc = create_document_with_edge();
        let hit = if let Some(h) = find_edge_at(&doc, 50.0, 0.0) {
            h
        } else {
            panic!("Expected to find edge at click position");
        };

        select_single_edge(&mut doc, hit.clone());
        assert!(
            doc.document.edges.contains_key(&hit),
            "I1: Selected ID must exist in document"
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_invariant_i4_edge_selection_does_not_mutate_nodes() {
        let source_id = NodeId::new(String::from("node-a"));
        let target_id = NodeId::new(String::from("node-b"));
        let edge_id = EdgeId::new(String::from("edge-1"));

        let mut doc = DiagramDocument {
            document: DocumentData {
                nodes: HashMap::new()
                    .update(source_id.clone(), node_at(0.0, 0.0))
                    .update(target_id.clone(), node_at(100.0, 0.0)),
                edges: HashMap::new().update(edge_id.clone(), edge(source_id, target_id)),
            },
            ..DiagramDocument::default()
        };

        let original_node_a_x = doc
            .document
            .nodes
            .get(&NodeId::new(String::from("node-a")))
            .map(|n| n.x.0);
        let original_node_b_x = doc
            .document
            .nodes
            .get(&NodeId::new(String::from("node-b")))
            .map(|n| n.x.0);

        select_single_edge(&mut doc, edge_id);

        let node_a_x = doc
            .document
            .nodes
            .get(&NodeId::new(String::from("node-a")))
            .map(|n| n.x.0);
        let node_b_x = doc
            .document
            .nodes
            .get(&NodeId::new(String::from("node-b")))
            .map(|n| n.x.0);

        assert_eq!(
            original_node_a_x, node_a_x,
            "I4: Node positions must not change after edge selection"
        );
        assert_eq!(
            original_node_b_x, node_b_x,
            "I4: Node positions must not change after edge selection"
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_postcondition_q3_no_nodes_selected() {
        let mut doc = create_document_with_edge();
        let hit = if let Some(h) = find_edge_at(&doc, 50.0, 0.0) {
            h
        } else {
            panic!("Expected to find edge at click position");
        };

        select_single_edge(&mut doc, hit);
        let has_node = doc
            .editor_state
            .selected_items
            .iter()
            .any(|id| doc.document.nodes.contains_key(&NodeId::new(id.clone())));

        assert!(
            !has_node,
            "Q3: No nodes should be selected when selecting an edge"
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_postcondition_q5_selection_replaces_previous() {
        let mut doc = create_document_with_edge();
        let hit = if let Some(h) = find_edge_at(&doc, 50.0, 0.0) {
            h
        } else {
            panic!("Expected to find edge at click position");
        };
        select_single_edge(&mut doc, hit.clone());
        select_single_edge(&mut doc, hit);

        assert_eq!(
            doc.editor_state.selected_items.len(),
            1,
            "Q5: Single-select should replace previous selection"
        );
    }
}
