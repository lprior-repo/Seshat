use super::document::{LockState, Node, NodeId, NodeKind, OrderedFloat};
use super::subgraph::reparenting::{set_node_parent_ext, unparent_node_ext};
use im::HashMap;

fn create_test_node(id: &str, x: f64, y: f64, parent: Option<NodeId>) -> Node {
    Node {
        kind: if id.starts_with("sg") {
            NodeKind::Subgraph
        } else {
            NodeKind::Text
        },
        icon: String::new(),
        label: id.to_string(),
        x: OrderedFloat::new_unchecked(x),
        y: OrderedFloat::new_unchecked(y),
        width: OrderedFloat::new_unchecked(100.0),
        height: OrderedFloat::new_unchecked(60.0),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    }
}

#[test]
fn test_sub023_child_coords_relative_to_parent() {
    let mut nodes = HashMap::new();
    let sg1_id = NodeId::new("sg1".to_string());
    let n1_id = NodeId::new("n1".to_string());

    let sg1 = create_test_node("sg1", 100.0, 100.0, None);
    let n1 = create_test_node("n1", 50.0, 50.0, Some(sg1_id.clone()));

    nodes.insert(sg1_id.clone(), sg1);
    nodes.insert(n1_id.clone(), n1.clone());

    // Stored coordinates are relative
    assert_eq!(n1.x.0, 50.0);
    assert_eq!(n1.y.0, 50.0);

    // World coordinates are absolute
    let (wx, wy) = n1.get_world_coords_im(&nodes).unwrap();
    assert_eq!(wx, 150.0);
    assert_eq!(wy, 150.0);
}

#[test]
fn test_sub024_moving_parent_updates_child_world_coords() {
    let mut nodes = HashMap::new();
    let sg1_id = NodeId::new("sg1".to_string());
    let n1_id = NodeId::new("n1".to_string());

    let mut sg1 = create_test_node("sg1", 100.0, 100.0, None);
    let n1 = create_test_node("n1", 50.0, 50.0, Some(sg1_id.clone()));

    nodes.insert(sg1_id.clone(), sg1.clone());
    nodes.insert(n1_id.clone(), n1.clone());

    // Move parent
    sg1.x = OrderedFloat::new_unchecked(200.0);
    nodes.insert(sg1_id.clone(), sg1);

    // Child stored coords same
    let n1_stored = nodes.get(&n1_id).unwrap();
    assert_eq!(n1_stored.x.0, 50.0);

    // Child world coords changed
    let (wx, wy) = n1_stored.get_world_coords_im(&nodes).unwrap();
    assert_eq!(wx, 250.0);
    assert_eq!(wy, 150.0);
}

#[test]
fn test_sub025_nesting_multiple_levels() {
    let mut nodes = HashMap::new();
    let sg1_id = NodeId::new("sg1".to_string());
    let sg2_id = NodeId::new("sg2".to_string());
    let n1_id = NodeId::new("n1".to_string());

    let sg1 = create_test_node("sg1", 100.0, 100.0, None);
    let sg2 = create_test_node("sg2", 50.0, 50.0, Some(sg1_id.clone()));
    let n1 = create_test_node("n1", 10.0, 10.0, Some(sg2_id.clone()));

    nodes.insert(sg1_id, sg1);
    nodes.insert(sg2_id, sg2);
    nodes.insert(n1_id.clone(), n1.clone());

    let (wx, wy) = n1.get_world_coords_im(&nodes).unwrap();
    assert_eq!(wx, 160.0); // 100 + 50 + 10
    assert_eq!(wy, 160.0);
}

#[test]
fn test_sub026_reparenting_preserves_world_position() {
    let mut canvas = crate::models::subgraph::types::CanvasState {
        nodes: HashMap::new(),
        edges: HashMap::new(),
    };

    let sg1_id = NodeId::new("sg1".to_string());
    let n1_id = NodeId::new("n1".to_string());

    let sg1 = create_test_node("sg1", 100.0, 100.0, None);
    let n1 = create_test_node("n1", 150.0, 150.0, None); // world coord

    canvas.nodes.insert(sg1_id.clone(), sg1);
    canvas.nodes.insert(n1_id.clone(), n1);

    // Reparent n1 to sg1, keeping world pos
    set_node_parent_ext(n1_id.clone(), sg1_id.clone(), &mut canvas, true).unwrap();

    let n1_updated = canvas.nodes.get(&n1_id).unwrap();
    assert_eq!(n1_updated.parent, Some(sg1_id));
    assert_eq!(n1_updated.x.0, 50.0); // 150 - 100
    assert_eq!(n1_updated.y.0, 50.0);

    let (wx, wy) = n1_updated.get_world_coords_im(&canvas.nodes).unwrap();
    assert_eq!(wx, 150.0);
    assert_eq!(wy, 150.0);
}

#[test]
fn test_sub027_root_node_is_world_space() {
    let nodes = HashMap::new();
    let n1 = create_test_node("n1", 123.0, 456.0, None);

    let (wx, wy) = n1.get_world_coords_im(&nodes).unwrap();
    assert_eq!(wx, 123.0);
    assert_eq!(wy, 456.0);
}

#[test]
fn test_unparenting_preserves_world_position() {
    let mut canvas = crate::models::subgraph::types::CanvasState {
        nodes: HashMap::new(),
        edges: HashMap::new(),
    };

    let sg1_id = NodeId::new("sg1".to_string());
    let n1_id = NodeId::new("n1".to_string());

    let sg1 = create_test_node("sg1", 100.0, 100.0, None);
    let n1 = create_test_node("n1", 50.0, 50.0, Some(sg1_id.clone()));

    canvas.nodes.insert(sg1_id.clone(), sg1);
    canvas.nodes.insert(n1_id.clone(), n1);

    // Unparent n1, keeping world pos
    unparent_node_ext(n1_id.clone(), &mut canvas, true).unwrap();

    let n1_updated = canvas.nodes.get(&n1_id).unwrap();
    assert_eq!(n1_updated.parent, None);
    assert_eq!(n1_updated.x.0, 150.0); // 100 + 50
    assert_eq!(n1_updated.y.0, 150.0);
}
