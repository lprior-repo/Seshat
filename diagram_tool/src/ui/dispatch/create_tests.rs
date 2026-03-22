use crate::ui::dispatch::create::*;
use crate::ui::dispatch::errors::DispatchError;
use diagram_models::document::{EdgeId, NodeId};
use diagram_models::envelope::DomainOp;

#[test]
fn given_valid_node_add_intent_when_created_then_produces_correct_envelope() {
    let env = create_node_add_envelope(
        "node1".to_string(),
        100.0,
        200.0,
        50.0,
        50.0,
        "Test Node".to_string(),
    )
    .expect("Should succeed with valid coordinates");

    assert_eq!(env.author.id, "local-user");

    match env.operation {
        DomainOp::NodeAdd {
            id,
            x,
            y,
            width,
            height,
            label,
        } => {
            assert_eq!(id, NodeId::new("node1".to_string()));
            assert_eq!(x, 100.0);
            assert_eq!(y, 200.0);
            assert_eq!(width, 50.0);
            assert_eq!(height, 50.0);
            assert_eq!(label, "Test Node");
        }
        _ => panic!("Expected NodeAdd operation"),
    }
}

#[test]
fn given_invalid_coordinates_when_node_add_created_then_returns_error() {
    let result = create_node_add_envelope(
        "node1".to_string(),
        f64::NAN,
        200.0,
        50.0,
        50.0,
        "Test".to_string(),
    );

    assert!(matches!(result, Err(DispatchError::InvalidCoordinates)));
}

#[test]
fn given_node_delete_intent_when_created_then_produces_correct_envelope() {
    let env = create_node_delete_envelope("node1".to_string());

    match env.operation {
        DomainOp::NodeDelete { id } => {
            assert_eq!(id, NodeId::new("node1".to_string()));
        }
        _ => panic!("Expected NodeDelete operation"),
    }
}

#[test]
fn given_edge_connect_intent_when_created_then_produces_correct_envelope() {
    let env = create_edge_connect_envelope(
        "edge1".to_string(),
        "source1".to_string(),
        "target1".to_string(),
    );

    match env.operation {
        DomainOp::EdgeConnect { id, source, target } => {
            assert_eq!(id, EdgeId::new("edge1".to_string()));
            assert_eq!(source, NodeId::new("source1".to_string()));
            assert_eq!(target, NodeId::new("target1".to_string()));
        }
        _ => panic!("Expected EdgeConnect operation"),
    }
}

#[test]
fn given_edge_disconnect_intent_when_created_then_produces_correct_envelope() {
    let env = create_edge_disconnect_envelope("edge1".to_string());

    match env.operation {
        DomainOp::EdgeDisconnect { id } => {
            assert_eq!(id, EdgeId::new("edge1".to_string()));
        }
        _ => panic!("Expected EdgeDisconnect operation"),
    }
}

#[test]
fn given_node_resize_intent_when_created_then_produces_correct_envelope() {
    let env = create_node_resize_envelope(
        NodeId::new("node1".to_string()),
        10.0,
        20.0,
        30.0,
        40.0,
        15.0,
        25.0,
        35.0,
        45.0,
    )
    .expect("Should succeed");

    match env.operation {
        DomainOp::NodeResize {
            id,
            original_x,
            original_y,
            original_width,
            original_height,
            x,
            y,
            width,
            height,
        } => {
            assert_eq!(id, NodeId::new("node1".to_string()));
            assert_eq!(original_x, 10.0);
            assert_eq!(original_y, 20.0);
            assert_eq!(original_width, 30.0);
            assert_eq!(original_height, 40.0);
            assert_eq!(x, 15.0);
            assert_eq!(y, 25.0);
            assert_eq!(width, 35.0);
            assert_eq!(height, 45.0);
        }
        _ => panic!("Expected NodeResize operation"),
    }
}
