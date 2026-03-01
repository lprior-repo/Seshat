# Implementation Summary: bd-2uy - ui-sync: update toolbar and ui state sync semantics

## Changes Made

### Phase 1: Tests First

Created new test module `cli_events_tests.rs` with 21 tests covering:
- CLI event JSONL serialization
- Error code mapping (parse_error, schema_violation, dag_violation, dangling_reference, semantic_error, stale_revision)
- Exit code mapping (2 for parse/command errors, 1 for others)
- Rejection path with Last Known Good (LKG) preservation
- Revision monotonicity and feedback

### Phase 2: Implementation

#### 1. Error Code Mapping (`cli.rs`)
- Updated `error_code()` function to properly map:
  - "dag"/"cycle" → `dag_violation`
  - "dangling"/"edge-dangling" → `dangling_reference`
  - "schema" → `schema_violation`
  - "semantic" → `semantic_error`
  - "parse"/"deserialize" → `parse_error`
  - Otherwise → `command_error`
- Made `CliEvent`, `error_code()`, `exit_code()` public for testing

#### 2. Validation Before Persistence (`cli_persistence.rs`)
- Added schema validation to `save_workspace_atomic()` to ensure only valid documents are persisted

#### 3. Enhanced Validate Command (`cli.rs`)
- Updated `Validate` command to run full validation pipeline (schema + document validation)
- Returns proper error codes for DAG violations and dangling references

#### 4. Added Patch Command (`cli.rs`)
- New `patch` subcommand supporting JSON Patch (RFC 6902) operations:
  - `test` - optimistic locking with revision check
  - `replace` - replace values
  - `add` - add values
  - `remove` - remove values (stub)
- **Requires test operation for /revision as first operation** (optimistic locking)
- Saves LKG on failure to `.lkg/` subdirectory
- Returns `stale_revision` error when revision test fails

#### 5. Revision Accessor (`models/document.rs`)
- Added `Revision::new()` constructor
- Added `Revision::value()` accessor method
- Made fields accessible for testing

## Verification

- **All 475 unit tests pass**
- **All 13 e2e tests pass**
- **Zero clippy errors**
- **Zero unwrap/expect/panic in production code**

## EARS Requirements Met

1. ✅ **JSONL events for every CLI command stage** - Commands emit start, stage, error, finish events
2. ✅ **Preserve last-known-good diagram state** - LKG fallback in load, LKG save on patch failure
3. ✅ **Validation pipeline before persistence** - Schema + document validation in save_workspace_atomic
4. ✅ **Machine-readable error codes** - Structured error codes (dag_violation, dangling_reference, stale_revision, etc.)
5. ✅ **No silent overwrites** - Revision test required for patch operations, LKG preserved on failure
6. ✅ **Revision monotonicity** - Revision only increments via server, preserves policy available

## Files Modified

- `diagram_tool/src/cli.rs` - Enhanced error mapping, validate command, patch command
- `diagram_tool/src/cli_persistence.rs` - Added validation before atomic save
- `diagram_tool/src/models/document.rs` - Added Revision accessors
- `diagram_tool/src/main.rs` - Added test module include
- `diagram_tool/src/cli_events_tests.rs` - New test module (21 tests)

## Files Created

- `.beads/bd-2uy/research_notes.md` - Research findings
- `.beads/bd-2uy/implementation.md` - This summary
