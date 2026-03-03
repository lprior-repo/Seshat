# Martin Fowler Test Plan: Test Infrastructure (bd-369)

## Test Categories and Counts

| Category | Prefix | Count | Description |
|----------|--------|-------|-------------|
| Selection | SEL | 25 | Hit-testing, selection state, handles |
| Clipboard | CLP | 10 | Copy/paste/cut/duplicate |
| History | HIS | 13 | Undo/redo, history stack |
| Multi-select | MUL | 37 | Drag/resize/rotate multiple |
| Subgraph | SUB | 34 | Groups, containers, reparenting |
| Edges | EDG | 35 | Bindings, routing, connections |
| Viewport | CAM | 12 | Pan, zoom, transforms |
| Geometry | GEO | 30 | Math, bounds, rotations |
| Snap/Align | SNP | 10 | Grid, alignment, distribution |
| Import/Export | IO | 15 | JSON, image export |
| Input | INP | 7 | Touch, stylus, gestures |
| **Total** | | **228** | (12 categories deferred for collaboration) |

---

## Happy Path Tests

### Fixture Loading
- `test_load_fixture_returns_valid_json_for_existing_fixture`
- `test_load_mixed_selection_has_five_nodes`
- `test_load_nested_subgraph_has_containment_structure`
- `test_validate_schema_accepts_version_2_documents`

### Test Runner
- `test_run_category_returns_report_with_pass_count`
- `test_run_all_tests_aggregates_category_reports`
- `test_test_names_are_expressive_and_unique`

### Golden Scene Creation
- `test_create_golden_scene_produces_valid_document`
- `test_save_golden_scene_writes_to_fixtures_dir`
- `test_golden_scene_is_deterministically_serializable`

### Property-Based Testing
- `test_fuzz_document_operations_produces_reproducible_results_from_same_seed`
- `test_verify_invariant_passes_for_valid_document`
- `test_fuzz_shrinks_failure_to_minimal_case`

### Visual Regression
- `test_capture_screenshot_produces_png`
- `test_compare_to_baseline_passes_for_identical_screenshot`
- `test_compare_to_baseline_fails_for_different_screenshot`

---

## Error Path Tests

### Fixture Errors
- `test_load_fixture_not_found_returns_fixture_not_found_error`
- `test_load_fixture_invalid_json_returns_invalid_json_error`
- `test_validate_schema_mismatch_returns_schema_mismatch_error`
- `test_get_nodes_missing_nodes_field_returns_missing_required_field_error`
- `test_get_edges_missing_edges_field_returns_missing_required_field_error`

### Test Runner Errors
- `test_run_category_not_implemented_returns_category_not_implemented_error`
- `test_test_timeout_returns_timeout_error`
- `test_ci_integration_failure_returns_ci_integration_error`

### Visual Regression Errors
- `test_compare_to_baseline_above_threshold_returns_visual_regression_error`
- `test_update_baseline_without_flag_returns_ci_integration_error`

### Property Test Errors
- `test_fuzz_determinism_violation_returns_property_failure_error`
- `test_fuzz_invariant_violation_returns_property_failure_error`

---

## Edge Case Tests

### Boundary Values
- `test_load_fixture_with_empty_nodes_map`
- `test_load_fixture_with_empty_edges_map`
- `test_load_fixture_with_max_coordinate_values`
- `test_load_fixture_with_negative_coordinates`
- `test_load_fixture_with_unicode_labels`
- `test_load_fixture_with_emoji_labels`
- `test_load_fixture_with_very_long_labels`

### Stress Tests
- `test_generate_5000_node_stress_fixture`
- `test_run_all_tests_completes_in_reasonable_time`
- `test_memory_usage_under_500mb_for_stress_fixture`

### Concurrency
- `test_parallel_test_categories_dont_interfere`
- `test_isolated_db_per_test`
- `test_no_shared_state_between_tests`

---

## Contract Verification Tests

### Preconditions
- `test_precondition_p1_test_category_is_valid_enum`
- `test_precondition_p2_fixture_exists_before_load`
- `test_precondition_p3_fixture_is_valid_json`
- `test_precondition_p4_schema_version_matches`
- `test_precondition_p5_no_external_network_types`
- `test_precondition_p6_unique_db_path_per_test`
- `test_precondition_p7_browser_available`

### Postconditions
- `test_postcondition_q1_all_240_tests_have_stubs`
- `test_postcondition_q2_golden_scenes_load_and_validate`
- `test_postcondition_q3_runner_reports_pass_fail_per_category`
- `test_postcondition_q4_ci_runs_on_commit`
- `test_postcondition_q5_flaky_tests_quarantined`
- `test_postcondition_q6_baselines_require_explicit_approval`
- `test_postcondition_q7_proptest_shrinks_failures`

### Invariants
- `test_invariant_i1_test_environment_reproducible`
- `test_invariant_i2_golden_scenes_version_controlled`
- `test_invariant_i3_test_execution_deterministic_given_seed`
- `test_invariant_i4_no_test_depends_on_order`
- `test_invariant_i5_failures_produce_actionable_diagnostics`

---

## Contract Violation Tests

(One test per violation example in contract-spec.md)

### Precondition Violations

```rust
#[test]
fn test_p2_violation_load_nonexistent_fixture_returns_fixture_not_found() {
    // Given: A fixture name that doesn't exist
    let name = "nonexistent.json";

    // When: We try to load it
    let result = load_fixture(name);

    // Then: Returns Err(FixtureNotFound), NOT panic or unwrap failure
    assert!(matches!(result, Err(TestHarnessError::FixtureNotFound(n)) if n == name));
}
```

```rust
#[test]
fn test_p3_violation_load_corrupted_json_returns_invalid_json_error() {
    // Given: A fixture file with invalid JSON
    let name = "corrupted.json";
    fs::write(fixtures_dir().join(name), "{invalid json}").unwrap();

    // When: We try to load it
    let result = load_fixture(name);

    // Then: Returns Err(InvalidJson), NOT panic
    assert!(matches!(result, Err(TestHarnessError::InvalidJson { name: n, .. }) if n == name));

    // Cleanup
    fs::remove_file(fixtures_dir().join(name)).ok();
}
```

```rust
#[test]
fn test_p4_violation_schema_version_mismatch_returns_schema_mismatch_error() {
    // Given: A document with wrong schema version
    let doc = json!({"version": 99, "document": {"nodes": {}, "edges": {}}});

    // When: We validate schema
    let result = validate_fixture_schema(&doc);

    // Then: Returns Err(SchemaMismatch), NOT panic
    assert!(matches!(result, Err(TestHarnessError::SchemaMismatch { expected: 2, found: 99 })));
}
```

```rust
#[test]
fn test_p7_violation_browser_unavailable_returns_browser_unavailable_error() {
    // Given: Browser binary not in PATH
    std::env::set_var("PATH", "");

    // When: We try to ensure browser
    let result = ensure_browser();

    // Then: Returns Err(BrowserUnavailable), NOT panic
    assert!(matches!(result, Err(TestHarnessError::BrowserUnavailable(_))));
}
```

### Postcondition Violations

```rust
#[test]
fn test_q1_violation_missing_test_stubs_fails_ci() {
    // Given: A test category with 0 implemented tests
    let report = run_test_category(TestCategory::Sel);

    // When: We check test count
    let count = report.test_count;

    // Then: Should fail CI (not silently pass)
    assert!(count >= 25, "SEL category must have at least 25 tests, got {}", count);
}
```

```rust
#[test]
fn test_q2_violation_missing_required_field_returns_error() {
    // Given: A fixture missing required field
    let doc = json!({"version": 2, "document": {"edges": {}}}); // missing nodes

    // When: We try to get nodes
    let result = get_nodes(&doc);

    // Then: Returns Err(MissingRequiredField), NOT panic
    assert!(matches!(result, Err(TestHarnessError::MissingRequiredField { field: "nodes", .. })));
}
```

```rust
#[test]
fn test_q6_violation_baseline_update_without_flag_returns_error() {
    // Given: No --update-baselines flag
    std::env::remove_var("UPDATE_BASELINES");

    // When: We try to update baseline
    let result = update_baseline("test", &screenshot);

    // Then: Returns Err(CiIntegration), NOT silently update
    assert!(matches!(result, Err(TestHarnessError::CiIntegration(msg)) if msg.contains("flag")));
}
```

### Invariant Violations

```rust
#[test]
fn test_i3_violation_determinism_failure_returns_property_failure_error() {
    // Given: Same seed should produce same result
    let seed = 12345u64;

    // When: We run fuzz twice with same seed
    let result1 = fuzz_document_operations(seed, 100).unwrap();
    let result2 = fuzz_document_operations(seed, 100).unwrap();

    // Then: Results must be identical
    assert_eq!(result1.projection_hash, result2.projection_hash,
        "Determinism violation: same seed produced different results");
}
```

```rust
#[test]
fn test_i4_violation_test_isolation_failure_detected() {
    // Given: Two tests that might share state
    let db_path1 = test_db_path("test_a");
    let db_path2 = test_db_path("test_b");

    // When: Tests run in sequence
    run_test("test_a", &db_path1);
    run_test("test_b", &db_path2);

    // Then: Each should have isolated DB
    assert_ne!(db_path1, db_path2, "Test DB paths must be unique");
    assert!(!state_leaked_between_tests(&db_path1, &db_path2),
        "Test isolation failure: state leaked between tests");
}
```

---

## Given-When-Then Scenarios

### Scenario 1: Load Valid Golden Scene
```
Given: A valid golden scene fixture exists at fixtures/mixed_selection.json
When: load_fixture("mixed_selection.json") is called
Then:
  - Returns Ok(Value) with parsed JSON
  - Value contains "version" field with value 2
  - Value contains "document.nodes" with >= 5 entries
  - Value contains "document.edges" with >= 1 entry
```

### Scenario 2: Run Full Test Suite
```
Given: All test categories have implementations
When: run_all_tests(&[all categories]) is called
Then:
  - Returns Ok(TestSuiteReport)
  - Report contains per-category pass/fail counts
  - Report contains total test count >= 228
  - All categories report at least expected test count
```

### Scenario 3: Fuzz Test Finds Failure
```
Given: Property-based test with invariant "no NaN in coordinates"
When: fuzz_document_operations(seed, 1000) finds a violating case
Then:
  - Returns Err(PropertyFailure)
  - Error contains shrunk minimal case
  - Error contains shrinks count > 0
  - Minimal case is reproducible with same seed
```

### Scenario 4: Visual Regression Detected
```
Given: A baseline screenshot exists for "selection-handles"
When: compare_to_baseline(&new_screenshot, "selection-handles", 0.1) is called
  And new_screenshot differs by 15% from baseline
Then:
  - Returns Err(VisualRegression { baseline: "selection-handles", delta: 15.0 })
  - NOT a panic or silent pass
```

### Scenario 5: CI Integration
```
Given: Code is committed to main branch
When: CI pipeline runs
Then:
  - moon run :test is executed
  - All rust unit tests pass
  - Playwright e2e tests run
  - Any failure blocks merge
```

### Scenario 6: Flaky Test Quarantine
```
Given: A test passes 8/10 runs (flaky)
When: Flaky test detector runs
Then:
  - Test is moved to quarantine file
  - CI passes without running quarantined test
  - Quarantine report lists flaky tests
  - NOT silently passing unreliable tests
```

### Scenario 7: Baseline Update Workflow
```
Given: Visual regression test fails intentionally (UI changed)
When: Developer runs with --update-baselines flag
Then:
  - New baseline is saved
  - Git shows diff in fixtures/
  - PR requires explicit review of baseline changes
```

---

## Exit Criteria Checklist

- [x] Every precondition has a type encoding specified
- [x] Every precondition has a concrete violation example
- [x] Every postcondition has a concrete violation example
- [x] Every violation example has a matching named test
- [x] Every `&mut` parameter has mutation postconditions (none in this module)
- [x] Every failure mode has a corresponding error variant
- [x] Test names describe behavior unambiguously
