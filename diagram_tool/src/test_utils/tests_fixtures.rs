#[cfg(kani)]
#[kani::proof]
fn test_fixtures_dir_returns_path() {
    let dir = crate::test_utils::fixtures_dir();
    assert!(dir.ends_with("tests/fixtures"));
}

#[cfg(kani)]
#[kani::proof]
fn test_load_fixture_not_found_returns_error() {
    let result = crate::test_utils::load_fixture("nonexistent_fixture_12345.json");
    assert!(result.is_err());

    if let Err(err) = result {
        assert!(
            matches!(err, crate::test_utils::TestHarnessError::FixtureNotFound(name) if name == "nonexistent_fixture_12345.json")
        );
    }
}

#[cfg(kani)]
#[kani::proof]
fn test_validate_fixture_schema_accepts_version_2() {
    let doc = serde_json::json!({"version": 2, "document": {"nodes": {}, "edges": {}}});
    let result = crate::test_utils::validate_fixture_schema(&doc);
    assert!(result.is_ok());
}

#[cfg(kani)]
#[kani::proof]
fn test_validate_fixture_schema_rejects_wrong_version() {
    let doc = serde_json::json!({"version": 99, "document": {"nodes": {}, "edges": {}}});
    let result = crate::test_utils::validate_fixture_schema(&doc);

    assert!(result.is_err());
    if let Err(crate::test_utils::TestHarnessError::SchemaMismatch { expected, found }) = result {
        assert_eq!(expected, 2);
        assert_eq!(found, 99);
    } else {
        panic!("Expected SchemaMismatch error");
    }
}

#[cfg(kani)]
#[kani::proof]
fn test_get_nodes_missing_nodes_returns_error() {
    let doc = serde_json::json!({"version": 2, "document": {"edges": {}}});
    let result = crate::test_utils::get_nodes(&doc);

    assert!(result.is_err());
    if let Err(crate::test_utils::TestHarnessError::MissingRequiredField { field, .. }) = result {
        assert!(field.contains("nodes"));
    } else {
        panic!("Expected MissingRequiredField error");
    }
}

#[cfg(kani)]
#[kani::proof]
fn test_get_edges_missing_edges_returns_error() {
    let doc = serde_json::json!({"version": 2, "document": {"nodes": {}}});
    let result = crate::test_utils::get_edges(&doc);

    assert!(result.is_err());
    if let Err(crate::test_utils::TestHarnessError::MissingRequiredField { field, .. }) = result {
        assert!(field.contains("edges"));
    } else {
        panic!("Expected MissingRequiredField error");
    }
}
