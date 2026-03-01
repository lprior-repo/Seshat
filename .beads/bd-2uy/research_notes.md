# Research Notes: bd-2uy - ui-sync: update toolbar and ui state sync semantics

## Phase 0: Research Findings

### Files Analyzed

1. **cli.rs** - Command-line interface with JSONL event emission
   - Already emits `CliEvent` (start, error, finish) as JSONL
   - Has `error_code()` and `exit_code()` functions mapping errors
   - Uses `emit_event()` for stdout JSONL output

2. **cli_persistence.rs** - Atomic persistence with LKG fallback
   - `save_workspace_atomic()` - atomic write pattern with temp file + rename
   - `load_workspace_with_lkg()` - tries primary, falls back to `.lkg` file
   - `StageDetails` and `emit_stage_event()` for progress tracking
   - Error types: `CliPersistenceError`

3. **mutation/pipeline.rs** - Validation pipeline
   - `run_mutation()` - runs transform, validates schema, validates document
   - `RevisionPolicy::Increment` or `Preserve`
   - Returns `MutationError` on validation failure

4. **mutation/error.rs** - Error types
   - `MutationError::Schema(String)` 
   - `MutationError::Semantic(String)`

5. **models/document.rs** - Document model
   - `DiagramDocument` with `revision: Revision`
   - `Revision(u64)` with `increment()` method

6. **ui/toolbar/persistence.rs** - UI persistence
   - Uses `run_mutation_with_policy` with `RevisionPolicy::Preserve`
   - Has `ImportTransitionError` for parse/validation errors
   - Already uses the mutation pipeline

### Research Questions Answered

1. **Which existing modules should host the shared mutation pipeline?**
   - `mutation/pipeline.rs` - Already is the shared pipeline
   - Used by both CLI (`cli.rs`) and UI (`toolbar/persistence.rs`)

2. **Where should JSONL event structs live for reuse across commands?**
   - `cli.rs` has `CliEvent` - could be moved to a shared module
   - `cli_persistence.rs` has `StageEvent` - already shared
   - Could consolidate into a new `cli_events` module if needed

### Gap Analysis

Looking at EARS requirements:

1. **JSONL events for every CLI command stage** - PARTIAL
   - Currently: `start`, `finish`, `error` events
   - Missing: Stage-level events (validating, loaded, persisted) are in `cli_persistence.rs`
   - Need: Unified event stream with exit code mapping

2. **Preserve last-known-good diagram state** - EXISTS
   - `load_workspace_with_lkg()` already handles this
   - `.lkg` fallback on validation failure

3. **Validation pipeline before persistence/broadcast** - EXISTS
   - `run_mutation()` validates schema and document

4. **Rejection returns machine-readable error code** - PARTIAL
   - `error_code()` function exists in cli.rs
   - Could be more structured

5. **No silent overwrites** - EXISTS
   - Revision is monotonic and server-owned
   - But need to ensure UI exposes revision feedback

### Implementation Tasks Identified

1. **Ensure UI uses shared pipeline** - ALREADY DONE (uses `run_mutation_with_policy`)
2. **Apply only valid states** - ALREADY DONE (validation in pipeline)
3. **Expose revision/state feedback to humans** - NEEDS WORK
   - Add revision display in toolbar
   - Show validation status
4. **Add structured error-code mapping and JSONL serializer** - NEEDS WORK
   - Create error code enum for better mapping
   - Ensure all errors have codes

### Test Requirements

Need tests for:
1. Command JSONL format and exit code map
2. Rejection path preserving last-known-good state
3. Revision state feedback visibility

