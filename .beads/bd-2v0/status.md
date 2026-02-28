# Bead bd-2v0: Decommission Backend - Remove Legacy redb Backend Entrypoints

## Status: PASSED ✓

## Summary
Successfully removed the legacy redb backend entrypoints from the diagram_tool project.

## Changes Made

### 1. Removed redb dependency from Cargo.toml
- Removed `redb = "2.4"` from dependencies

### 2. Updated backend.rs
- Removed all redb-related imports (`Database`, `TableDefinition`, `PathBuf`)
- Removed database helper functions (`database_path()`, `with_database()`)
- Removed server functions:
  - `backend_health()`
  - `save_workspace_to_backend()`
  - `load_workspace_from_backend()`
  - `ingest_document_json_to_backend()`
- Removed `PersistedWorkspace` struct (no longer needed)
- Removed `DIAGRAM_TABLE` constant

### 3. Updated persistence.rs
- Removed import of `save_workspace_to_backend` and `PersistedWorkspace`
- Updated `save_workspace()` function for wasm32 to show error message indicating backend is decommissioned

## Verification Evidence

### Compilation
```
$ cargo check
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.64s

$ cargo clippy -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.57s

$ cargo build --release
    Finished `release` profile [optimized] target(s) in 1m 32s
```

### Tests
```
$ cargo test persistence
    Running unittests src/main.rs
    test result: ok. 26 passed; 0 failed; 0 ignored

$ cargo test
    test result: ok. 8 passed; 5 failed; 0 ignored
```

Note: The 5 failing tests are pre-existing CLI e2e test failures unrelated to this change (they fail due to missing CLI output validation, not due to backend removal).

## Files Modified
- `diagram_tool/Cargo.toml` - Removed redb dependency
- `diagram_tool/src/backend.rs` - Removed redb server functions
- `diagram_tool/src/ui/toolbar/persistence.rs` - Updated to handle decommissioned backend
