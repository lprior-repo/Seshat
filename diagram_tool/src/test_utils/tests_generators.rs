use crate::test_utils::NodeSpec;
use diagram_models::document::{LockState, NodeKind};

#[cfg(kani)]
#[kani::proof]
fn test_create_golden_scene_produces_valid_document() {
    let nodes = vec![NodeSpec {
        id: "test-node-1".to_string(),
        kind: NodeKind::Node,
        label: "Test Node".to_string(),
        x: 100.0,
        y: 200.0,
        width: 80.0,
        height: 40.0,
        icon: String::new(),
        parent: None,
        lock_state: LockState::Unlocked,
        z_index: 0,
        metadata: serde_json::Map::new(),
    }];

    let doc = crate::test_utils::create_golden_scene("test", nodes, vec![]);

    assert_eq!(doc["version"].as_u64(), Some(2));
    assert!(doc["document"]["nodes"].get("test-node-1").is_some());
}

#[cfg(kani)]
#[kani::proof]
fn test_generate_stress_scene_produces_5000_nodes() {
    let doc = crate::test_utils::generate_stress_scene(12345);

    if let Some(nodes) = doc["document"]["nodes"].as_object() {
        assert_eq!(nodes.len(), 5000);
    } else {
        panic!("Expected nodes object");
    }

    if let Some(edges) = doc["document"]["edges"].as_object() {
        assert_eq!(edges.len(), 5000);
    } else {
        panic!("Expected edges object");
    }
}

#[cfg(kani)]
#[kani::proof]
fn test_generate_stress_scene_is_deterministic() {
    let doc1 = crate::test_utils::generate_stress_scene(12345);
    let doc2 = crate::test_utils::generate_stress_scene(12345);

    assert_eq!(doc1, doc2);
}

#[cfg(kani)]
#[kani::proof]
fn test_fuzz_document_operations_produces_deterministic_report() {
    let report1 = crate::test_utils::fuzz_document_operations(12345, 100).unwrap();
    let report2 = crate::test_utils::fuzz_document_operations(12345, 100).unwrap();

    assert_eq!(report1.projection_hash, report2.projection_hash);
    assert_eq!(report1.seed, report2.seed);
    assert_eq!(report1.cases_run, report2.cases_run);
}
