#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
use crate::ui::dispatch::errors::DispatchError;
use crate::ui::dispatch::send::node::*;
use diagram_models::document::NodeId;
use diagram_models::envelope::EventEnvelope;

#[test]
fn given_node_add_intent_without_channel_then_returns_wal_disconnected_error() {
    let env = EventEnvelope {
        op_id: "test".to_string(),
        operation: diagram_models::envelope::DomainOp::NodeAdd {
            id: NodeId::new("n1".to_string()),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            label: "test".to_string(),
        },
        author: diagram_models::envelope::Author {
            id: "u1".to_string(),
            name: "User".to_string(),
            email: None,
        },
        timestamp: 0,
    };

    let db_tx: Option<dioxus::prelude::Coroutine<EventEnvelope>> = None;
    let result = dispatch_node_add(&db_tx, env);

    assert!(matches!(result, Err(DispatchError::WalDisconnected)));
}

#[test]
fn given_empty_node_delete_batch_then_returns_no_selection_error() {
    let db_tx: Option<dioxus::prelude::Coroutine<EventEnvelope>> = None;
    let ids: Vec<String> = vec![];

    let result = dispatch_node_delete_batch(&db_tx, &ids);

    assert!(matches!(result, Err(DispatchError::NoSelection)));
}

#[test]
fn given_valid_node_delete_without_channel_then_returns_wal_disconnected_error() {
    let db_tx: Option<dioxus::prelude::Coroutine<EventEnvelope>> = None;

    let result = dispatch_node_delete(&db_tx, "node1");

    assert!(matches!(result, Err(DispatchError::WalDisconnected)));
}

#[test]
fn given_node_resize_intent_without_channel_then_returns_wal_disconnected_error() {
    let db_tx: Option<dioxus::prelude::Coroutine<EventEnvelope>> = None;
    let bounds = ResizeBounds::new(
        NodeId::new("n1".to_string()),
        0.0,
        0.0,
        100.0,
        100.0,
        10.0,
        10.0,
        120.0,
        120.0,
    );

    let result = dispatch_node_resize(&db_tx, bounds);

    assert!(matches!(result, Err(DispatchError::WalDisconnected)));
}
