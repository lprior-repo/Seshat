use diagram_models::document::LockState;
use diagram_models::document::Node;
use diagram_models::document::NodeKind;
use diagram_models::document::OrderedFloat;
use diagram_models::document::{EdgeId, NodeId};
use diagram_models::projection::ops::{apply_edge_connect_checked, apply_update_edge_label};
use diagram_models::projection::types::DiagramProjection;

fn create_test_node(id: &str) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: id.to_string(),
        x: OrderedFloat(0.0),
        y: OrderedFloat(0.0),
        width: OrderedFloat(100.0),
        height: OrderedFloat(100.0),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent: None,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: im::HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    }
}

/// Adversarial test for edge label validation.
///
/// Per the contract (defects.md), safe whitespace control characters are allowed:
/// - `\n` (newline)
/// - `\r` (carriage return)
/// - `\t` (tab)
///
/// The following should be rejected:
/// - Null bytes
/// - Other control characters
/// - Strings exceeding MAX_LABEL_LENGTH (4096 chars)
/// - Zero-width spaces (visual spoofing)
/// - Bi-directional overrides (visual spoofing)
#[test]
fn adversarial_edge_labels() {
    let mut state = DiagramProjection::default();

    state
        .nodes
        .insert(NodeId::new("n1".to_string()), create_test_node("n1"));
    state
        .nodes
        .insert(NodeId::new("n2".to_string()), create_test_node("n2"));

    // Create an edge
    let state = apply_edge_connect_checked(state, "e1", "n1", "n2").unwrap();
    let edge_id = EdgeId::new("e1".to_string());

    let massive_string = "A".repeat(100_000);

    // Cases that should be ACCEPTED (safe whitespace)
    let accepted_labels = vec![
        ("newline", "line1\nline2"),
        ("carriage return", "line1\rline2"),
        ("tab", "col1\tcol2"),
    ];

    // Cases that should be REJECTED
    let rejected_labels = vec![
        ("null byte", "\0"),
        ("massive string", massive_string.as_str()),
        ("control chars", "\x01\x02\x03"),
        ("zero width space", "\u{200B}"),
        ("bidi override", "\u{202E}RLO"),
    ];

    let mut accepted_unexpectedly = Vec::new();
    let mut rejected_unexpectedly = Vec::new();

    // Verify accepted labels work
    for (name, label) in &accepted_labels {
        let result = apply_update_edge_label(state.clone(), "e1", label);
        match result {
            Ok(new_state) => {
                let e = new_state.edges.get(&edge_id).unwrap();
                println!("Correctly accepted {name}: length={}", e.label.len());
            }
            Err(e) => {
                println!("Unexpectedly rejected {name}: {e:?}");
                rejected_unexpectedly.push(*name);
            }
        }
    }

    // Verify rejected labels fail
    for (name, label) in &rejected_labels {
        let result = apply_update_edge_label(state.clone(), "e1", label);
        match result {
            Ok(new_state) => {
                let e = new_state.edges.get(&edge_id).unwrap();
                println!("Unexpectedly accepted {name}: length={}", e.label.len());
                accepted_unexpectedly.push(*name);
            }
            Err(e) => {
                println!("Correctly rejected {name}: {e:?}");
            }
        }
    }

    assert!(
        rejected_unexpectedly.is_empty(),
        "Labels should have been accepted but were rejected: {:?}",
        rejected_unexpectedly
    );

    assert!(
        accepted_unexpectedly.is_empty(),
        "Labels should have been rejected but were accepted: {:?}",
        accepted_unexpectedly
    );
}
