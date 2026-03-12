# Architecture Refactor Report

## Beads Reviewed

### Latest: nu_runner Module Split

**Date**: 2026-03-12

**Original File**: `nu_runner/src/lib.rs` - 582 lines

**Issue**: File exceeded 300 line limit

**Solution**: Split into multiple modules following DDD principles

### Previous Reviews

- `seshat-zrx` - UI Dispatch: Prop Panel Node Shape (already implemented)
- `seshat-6j0` - Edge Style Dispatch (adds dispatch_update_edge_style)
- `seshat-6jd` - AI Conflict State Signal
- `seshat-7fz` - WAL Poller Dropped Event Detection
- `seshat-3t5` - Toast Component for AI Conflict State
- `seshat-fsa` - Toast Auto-Dismiss After 3 Seconds

## Issues Found

### 2. File Exceeding 300 Lines: `nu_runner/src/lib.rs`

**Original**: 582 lines  
**Issue**: Single file exceeded 300 line limit with mixed concerns

**Solution**: Split into focused modules with explicit responsibilities

**Files Changed/Created**:

| File | Lines | Purpose |
|------|-------|---------|
| `lib.rs` | 224 | Module declarations, re-exports, tests |
| `types.rs` | 103 | Core domain types (NuOutput, NuConfig, NuRunner) |
| `newtypes.rs` | 255 | Value objects (NuPath, TimeoutMs, ExitCode, EnvVars, Command) |
| `errors.rs` | 70 | Error taxonomy (NuError) |
| `state.rs` | 68 | State machine (RunnerState, RunnerStateError) |
| `validation.rs` | 184 | Pure validation functions |
| `runner.rs` | 200 | Implementation (Actions layer) |
| `main.rs` | 1 | Entry point |

### DDD Improvements Applied

1. **Newtypes (Eliminating Primitive Obsession)**:
   - `NuPath` - Wraps `String` for nushell executable path
   - `TimeoutMs` - Wraps `u64` for timeout, rejects zero
   - `ExitCode` - Wraps `i32` with semantic methods
   - `EnvVars` - Wraps `HashMap<String, String>` for UTF-8 guarantee
   - `Command` - Wraps validated command strings

2. **Explicit State Machine**:
   - Replaced `is_executing: bool` with `RunnerState` enum
   - State transitions: `start_executing()`, `finish_executing()`

3. **Error Taxonomy**:
   - `NuError` with 7 explicit variants

### 1. File Exceeding 300 Lines: `ai_event_detection.rs` (seshat-7fz)

**Original**: 421 lines  
**Issue**: Had 246 lines of inline tests (lines 175-421)

**Solution**: Extracted tests to separate file following codebase pattern (`tests.rs`)

**Files Changed**:
- `diagram_tool/src/ai_event_detection.rs` - Removed inline test module, added `mod tests;` declaration
- `diagram_tool/src/ai_event_detection/tests.rs` - NEW file containing all tests

## Final Line Counts

| File | Lines | Status |
|------|-------|--------|
| `nu_runner/src/lib.rs` | 224 | ✅ Under 300 |
| `nu_runner/src/types.rs` | 103 | ✅ Under 300 |
| `nu_runner/src/newtypes.rs` | 255 | ✅ Under 300 |
| `nu_runner/src/errors.rs` | 70 | ✅ Under 300 |
| `nu_runner/src/state.rs` | 68 | ✅ Under 300 |
| `nu_runner/src/validation.rs` | 184 | ✅ Under 300 |
| `nu_runner/src/runner.rs` | 200 | ✅ Under 300 |
| `ai_event_detection.rs` | 176 | ✅ Under 300 |
| `ai_event_detection/tests.rs` | 248 | ✅ Separate test file |
| `hooks/ai_conflict.rs` | 80 | ✅ Under 300 |

## DDD Compliance

The reviewed code follows Scott Wlaschin DDD principles:

1. **Parse, don't validate**: `AiConflictState` struct with `has_valid_reason()` method
2. **Make illegal states unrepresentable**: `DropDetectionResult` uses `Option<String>` for optional conflict message
3. **NewTypes**: Uses `NodeId`, `EdgeId`, `Author` from models instead of primitives
4. **Pure functions**: `detect_dropped_ai_events()`, `find_dropped_op_ids()`, `generate_conflict_message()` are all pure calculations

### nu_runner DDD Features

1. **Newtypes**: `NuPath`, `TimeoutMs`, `ExitCode`, `EnvVars`, `Command` eliminate primitive obsession
2. **Explicit State**: `RunnerState` enum replaces boolean flag
3. **Error Taxonomy**: 7 explicit `NuError` variants
4. **Parse at Boundaries**: Validation functions convert raw inputs to domain types

## Notes

- Pre-existing files (`dispatch.rs`, `properties.rs`, `app.rs`, `toast.rs`) exceed 300 lines but were not modified by the current beads - they existed before
- The compilation errors shown are pre-existing issues in the codebase unrelated to these beads
