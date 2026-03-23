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
    let labels_to_test = vec![
        ("null byte", "\0"),
        ("newline", "line1\nline2"),
        ("carriage return", "line1\rline2"),
        ("massive string", massive_string.as_str()),
        ("control chars", "\x01\x02\x03"),
        ("zero width space", "\u{200B}"),
        ("bidi override", "\u{202E}RLO"),
    ];

    let mut failed_cases = Vec::new();

    for (name, label) in labels_to_test {
        let result = apply_update_edge_label(state.clone(), "e1", label);
        match result {
            Ok(new_state) => {
                let e = new_state.edges.get(&edge_id).unwrap();
                println!(
                    "Accepted {name}: length={}, contains null={}",
                    e.label.len(),
                    e.label.contains('\0')
                );
                failed_cases.push(name);
            }
            Err(e) => {
                println!("Rejected {name}: {e:?}");
            }
        }
    }

    assert!(
        failed_cases.is_empty(),
        "Validation missing for cases: {:?}",
        failed_cases
    );
}
