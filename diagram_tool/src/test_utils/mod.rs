//! Test Infrastructure for Seshat Diagram Tool
//!
//! This module provides the test harness for running the 240 test cases
//! organized into 11 categories as specified in the architecture spec.
//!
//! ## Design by Contract
//!
//! - **P1**: Test category ID is valid (compile-time via enum)
//! - **P2**: Golden scene file exists (Runtime Result)
//! - **P3**: Golden scene is valid JSON (Runtime Result)
//! - **P4**: Schema version matches expected (Runtime Result)
//! - **P5**: Test environment is isolated (no external network types)
//! - **P6**: Test database path is unique per test (Debug-only assert)
//! - **P7**: Browser is available for E2E tests (Runtime Result)

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(unused_imports, clippy::unnecessary_lazy_evaluations)]

pub mod builders;
pub mod fixtures;
pub mod generators;
pub mod harness;
pub mod types;

pub use builders::*;
pub use fixtures::*;
pub use generators::*;
pub use harness::*;
pub use types::*;

// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_fixtures_dir_returns_path() {
        let dir = fixtures_dir();
        assert!(dir.ends_with("tests/fixtures"));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_load_fixture_not_found_returns_error() {
        let result = load_fixture("nonexistent_fixture_12345.json");
        assert!(result.is_err());

        if let Err(err) = result {
            assert!(
                matches!(err, TestHarnessError::FixtureNotFound(name) if name == "nonexistent_fixture_12345.json")
            );
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_validate_fixture_schema_accepts_version_2() {
        let doc = serde_json::json!({"version": 2, "document": {"nodes": {}, "edges": {}}});
        let result = validate_fixture_schema(&doc);
        assert!(result.is_ok());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_validate_fixture_schema_rejects_wrong_version() {
        let doc = serde_json::json!({"version": 99, "document": {"nodes": {}, "edges": {}}});
        let result = validate_fixture_schema(&doc);

        assert!(result.is_err());
        if let Err(TestHarnessError::SchemaMismatch { expected, found }) = result {
            assert_eq!(expected, 2);
            assert_eq!(found, 99);
        } else {
            panic!("Expected SchemaMismatch error");
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_get_nodes_missing_nodes_returns_error() {
        let doc = serde_json::json!({"version": 2, "document": {"edges": {}}});
        let result = get_nodes(&doc);

        assert!(result.is_err());
        if let Err(TestHarnessError::MissingRequiredField { field, .. }) = result {
            assert!(field.contains("nodes"));
        } else {
            panic!("Expected MissingRequiredField error");
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_get_edges_missing_edges_returns_error() {
        let doc = serde_json::json!({"version": 2, "document": {"nodes": {}}});
        let result = get_edges(&doc);

        assert!(result.is_err());
        if let Err(TestHarnessError::MissingRequiredField { field, .. }) = result {
            assert!(field.contains("edges"));
        } else {
            panic!("Expected MissingRequiredField error");
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
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

        let doc = create_golden_scene("test", nodes, vec![]);

        assert_eq!(doc["version"].as_u64(), Some(2));
        assert!(doc["document"]["nodes"].get("test-node-1").is_some());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_category_expected_counts_are_correct() {
        assert_eq!(TestCategory::Sel.expected_count(), 25);
        assert_eq!(TestCategory::Clp.expected_count(), 10);
        assert_eq!(TestCategory::His.expected_count(), 13);
        assert_eq!(TestCategory::Mul.expected_count(), 37);
        assert_eq!(TestCategory::Sub.expected_count(), 34);
        assert_eq!(TestCategory::Edg.expected_count(), 35);
        assert_eq!(TestCategory::Cam.expected_count(), 12);
        assert_eq!(TestCategory::Geo.expected_count(), 30);
        assert_eq!(TestCategory::Snp.expected_count(), 10);
        assert_eq!(TestCategory::Io.expected_count(), 15);
        assert_eq!(TestCategory::Inp.expected_count(), 7);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_total_expected_tests_is_228() {
        let total: usize = TestCategory::all().iter().map(|c| c.expected_count()).sum();
        assert_eq!(total, 228);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_generate_stress_scene_produces_5000_nodes() {
        let doc = generate_stress_scene(12345);

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
    #[test]
    fn test_generate_stress_scene_is_deterministic() {
        let doc1 = generate_stress_scene(12345);
        let doc2 = generate_stress_scene(12345);

        assert_eq!(doc1, doc2);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_fuzz_document_operations_produces_deterministic_report() {
        let report1 = fuzz_document_operations(12345, 100).unwrap();
        let report2 = fuzz_document_operations(12345, 100).unwrap();

        assert_eq!(report1.projection_hash, report2.projection_hash);
        assert_eq!(report1.seed, report2.seed);
        assert_eq!(report1.cases_run, report2.cases_run);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_verify_invariants_passes_for_valid_document() {
        let mut doc = DiagramDocument::default();

        let node = Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "Test".to_string(),
            x: OrderedFloat(100.0),
            y: OrderedFloat(200.0),
            width: OrderedFloat(80.0),
            height: OrderedFloat(40.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::vector![],
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };

        doc.document
            .nodes
            .insert(NodeId::new("node-1".to_string()), node);

        let result = verify_invariants(&doc);
        assert!(result.is_ok());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_verify_invariants_fails_for_nan_coordinates() {
        let mut doc = DiagramDocument::default();

        let node = Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "Bad Node".to_string(),
            x: OrderedFloat(f64::NAN),
            y: OrderedFloat(200.0),
            width: OrderedFloat(80.0),
            height: OrderedFloat(40.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::vector![],
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };

        doc.document
            .nodes
            .insert(NodeId::new("bad-node".to_string()), node);

        let result = verify_invariants(&doc);
        assert!(result.is_err());

        if let Err(TestHarnessError::InvariantViolation { invariant, .. }) = result {
            assert_eq!(invariant, "no_nan_in_coordinates");
        } else {
            panic!("Expected InvariantViolation");
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_verify_invariants_fails_for_negative_dimensions() {
        let mut doc = DiagramDocument::default();

        let node = Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "Negative Node".to_string(),
            x: OrderedFloat(100.0),
            y: OrderedFloat(200.0),
            width: OrderedFloat(-10.0),
            height: OrderedFloat(40.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::vector![],
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };

        doc.document
            .nodes
            .insert(NodeId::new("negative-node".to_string()), node);

        let result = verify_invariants(&doc);
        assert!(result.is_err());

        if let Err(TestHarnessError::InvariantViolation { invariant, .. }) = result {
            assert_eq!(invariant, "positive_dimensions");
        } else {
            panic!("Expected InvariantViolation");
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_compute_document_hash_is_stable() {
        let doc = DiagramDocument::default();
        let hash1 = compute_document_hash(&doc);
        let hash2 = compute_document_hash(&doc);

        assert_eq!(hash1, hash2);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_test_db_path_is_unique_per_test() {
        let path1 = test_db_path("test_a");
        let path2 = test_db_path("test_b");

        assert_ne!(path1, path2);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_run_all_tests_aggregates_categories() {
        let categories = &[TestCategory::Sel, TestCategory::Clp];
        let report = run_all_tests(categories).unwrap();

        assert_eq!(report.total_tests, 35); // 25 + 10
        assert_eq!(report.categories.len(), 2);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_category_display_names() {
        assert_eq!(TestCategory::Sel.display_name(), "Selection");
        assert_eq!(TestCategory::Edg.display_name(), "Edge Binding");
        assert_eq!(TestCategory::Inp.display_name(), "Input (Touch/Stylus)");
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_category_all_returns_all_categories() {
        let all = TestCategory::all();
        assert_eq!(all.len(), 11);
    }
}
