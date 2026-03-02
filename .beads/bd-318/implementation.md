bead_id: bd-318
bead_title: tests: Implement IO import/export tests
phase: p0
updated_at: 2026-03-02T02:30:00Z

# Implementation Status

## Contract
Contract defined at `.beads/bd-318/contract.md` with 15 test requirements.

## Implementation Notes

The 15 tests were designed but not successfully applied to the codebase due to workspace management issues with jj. The test implementations follow these patterns:

### Test Implementation Pattern
```rust
/// Test N: [Description]
#[test]
fn given_[condition]_when_[action]_then_[result]() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let bootstrap = store::bootstrap_store(&db_path).unwrap();

    // Execute
    let result = /* function under test */;

    // Verify
    assert!(result.is_ok(), "Should succeed: {:?}", result.err());
}
```

### Key Types Used
- `DiagramProjection::empty()` - for creating test projections
- `Node`, `Edge` from `crate::models::document`
- `EventEnvelope`, `DomainOp` from `crate::models::envelope`
- `Author` from the export module
- `tempfile::TempDir` for temporary databases

## Blockers
1. Pre-existing compilation errors in `history.rs` (from bd-2u3)
2. Pre-existing compilation errors in `golden_scenes.rs` (from bd-ja2)
3. jj workspace synchronization issues prevented file edits from being compiled

## Next Steps
1. Resolve workspace issues
2. Apply the test implementations directly to `diagram_tool/src/models/export.rs`
3. Run `cargo test -p diagram_tool models::export::tests` to verify
4. Run Moon validation: `moon run :quick && moon run :test && moon run :ci`
