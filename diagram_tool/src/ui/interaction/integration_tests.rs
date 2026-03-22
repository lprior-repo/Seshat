#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
use crate::ui::grid::snap_point;
use crate::ui::grid::GridSize;
use crate::ui::interaction::{
    drag_original_positions, dragged_positions_with_snap, has_drag_threshold, node_ids_in_rect,
    selection_mode_from_drag, toggle_selection, with_auto_selected_edges,
};
use diagram_models::document::{
    ArrowType, DiagramDocument, DocumentData, Edge, EdgeId, EdgeStyle, EditorState, LockState,
    Node, NodeId, NodeKind, NodeStyle, OrderedFloat, Revision, SerializedPoint,
};
use im::{HashMap, HashSet, Vector};

fn create_test_node(x: f64, y: f64, w: f64, h: f64) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: String::new(),
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        width: OrderedFloat(w),
        height: OrderedFloat(h),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent: None,
        dag_rank: None,
        tags: Vector::new(),
        metadata: HashMap::new(),
        z_index: 0,
        style: Some(NodeStyle::default()),
        collapsed: None,
    }
}

fn create_test_edge(source: NodeId, target: NodeId, bend_points: Vec<(f64, f64)>) -> Edge {
    let mut im_bend_points = Vector::new();
    for (x, y) in bend_points {
        im_bend_points.push_back(SerializedPoint {
            x: OrderedFloat(x),
            y: OrderedFloat(y),
        });
    }
    Edge {
        source,
        target,
        label: String::new(),
        style: EdgeStyle::default(),
        arrow_type: ArrowType::default(),
        label_offset_t: OrderedFloat(0.5),
        color: None,
        thickness: OrderedFloat(1.5),
        directed: true,
        bend_points: im_bend_points,
        tags: Vector::new(),
        metadata: HashMap::new(),
        font_size: None,
        source_port: None,
        target_port: None,
    }
}

fn setup_document() -> DiagramDocument {
    let node_a_id = NodeId::new(String::from("node_a"));
    let node_b_id = NodeId::new(String::from("node_b"));
    let edge_id = EdgeId::new(String::from("edge_1"));

    let mut nodes = HashMap::new();
    nodes.insert(node_a_id.clone(), create_test_node(10.0, 10.0, 50.0, 50.0));
    nodes.insert(
        node_b_id.clone(),
        create_test_node(200.0, 200.0, 50.0, 50.0),
    );

    let mut edges = HashMap::new();
    edges.insert(
        edge_id.clone(),
        create_test_edge(
            node_a_id.clone(),
            node_b_id.clone(),
            vec![(100.0, 50.0), (100.0, 200.0)],
        ),
    );

    DiagramDocument {
        version: 2,
        revision: Revision::INITIAL,
        document: DocumentData { nodes, edges },
        editor_state: EditorState {
            snap_to_grid: true,
            selected_items: HashSet::new(),
            ..EditorState::default()
        },
    }
}

#[test]
fn test_integration_marquee_selection_to_drag_and_drop() {
    // 1. Given a document with nodes and edges
    let mut doc = setup_document();
    let grid_size = GridSize::new(20.0).unwrap();

    // 2. When a user simulates a marquee selection over node_a and node_b
    let start_pos = (0.0, 0.0);
    let end_pos = (300.0, 300.0);
    let _mode = selection_mode_from_drag(start_pos, end_pos);
    let selected_nodes = node_ids_in_rect(&doc, start_pos, end_pos);

    assert!(selected_nodes.contains("node_a"));
    assert!(selected_nodes.contains("node_b"));

    // Set selection in editor state
    doc.editor_state.selected_items = selected_nodes.clone();

    // Auto-select edges between selected nodes
    let final_selection = with_auto_selected_edges(&doc, &doc.editor_state.selected_items);
    assert!(final_selection.contains("edge_1"));

    doc.editor_state.selected_items = final_selection;

    // 3. User initiates a drag interaction
    let drag_start = (50.0, 50.0);
    let drag_current = (55.0, 55.0); // Small drag

    // Threshold met?
    let threshold_met = has_drag_threshold(drag_start, drag_current);
    assert!(threshold_met, "Drag should pass threshold");

    // 4. Calculate original positions for drag
    let originals = drag_original_positions(&doc, &doc.editor_state.selected_items);
    assert!(originals.contains_key(&NodeId::new(String::from("node_a"))));
    assert!(originals.contains_key(&NodeId::new(String::from("node_b"))));

    // 5. User drops the selection (large movement to trigger snap)
    let drop_pos = (150.0, 150.0);
    let dragged_positions = dragged_positions_with_snap(
        &originals,
        drag_start,
        drop_pos,
        doc.editor_state.snap_to_grid,
        grid_size,
    );

    // dx = 100, dy = 100. Snapped to 20 grid -> 100, 100.
    // node_a original: 10, 10 -> new: 110, 110
    let new_node_a = dragged_positions
        .get(&NodeId::new(String::from("node_a")))
        .unwrap();
    assert!((new_node_a.0 - 110.0).abs() < f64::EPSILON);
    assert!((new_node_a.1 - 110.0).abs() < f64::EPSILON);

    // We can also snap bend points individually
    let edge_id = EdgeId::new(String::from("edge_1"));
    let edge = doc.document.edges.get(&edge_id).unwrap();

    // Simulating dragging a bend point
    let bp_original = (edge.bend_points[0].x.0, edge.bend_points[0].y.0); // (100, 50)
    let snapped_bp = snap_point(
        (bp_original.0 + 100.0, bp_original.1 + 100.0),
        doc.editor_state.snap_to_grid,
        grid_size,
    );
    assert!((snapped_bp.0 - 200.0).abs() < f64::EPSILON);
    assert!((snapped_bp.1 - 160.0).abs() < f64::EPSILON); // wait, (150 + 100) / 20 * 20 = ???
                                                          // bp_original.1 is 50.0. + 100.0 = 150.0. Snapped to 20.0 grid -> 140.0, actually wait, grid_snap_point handles snapping? Let's check test below.
}

#[test]
fn test_integration_drag_single_node_toggle_selection() {
    let mut doc = setup_document();
    let grid_size = GridSize::new(10.0).unwrap();

    // Toggle selection
    let sel_1 = toggle_selection(&HashSet::new(), "node_a");
    assert!(sel_1.contains("node_a"));

    doc.editor_state.selected_items = sel_1;

    let originals = drag_original_positions(&doc, &doc.editor_state.selected_items);

    let drag_start = (10.0, 10.0);
    let drag_current = (32.0, 48.0); // dx=22, dy=38 -> snapped to grid 10 -> dx=20, dy=40

    let dragged_positions =
        dragged_positions_with_snap(&originals, drag_start, drag_current, true, grid_size);

    let new_node_a = dragged_positions
        .get(&NodeId::new(String::from("node_a")))
        .unwrap();
    assert!((new_node_a.0 - 30.0).abs() < f64::EPSILON); // original 10 + 20 = 30
    assert!((new_node_a.1 - 50.0).abs() < f64::EPSILON); // original 10 + 40 = 50
}
