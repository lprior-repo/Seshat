# Implementation Summary: bd-260 - cli: fix dag_violation error code

bead_id: bd-260
bead_title: cli: fix dag_violation error code
phase: p2
updated_at: 2026-03-01T23:00:00Z

## Status: ALREADY IMPLEMENTED

The `dag_violation` error code was already correctly implemented in a prior bead (bd-2uy: ui-sync).

## Implementation Details

### Error Code Mapping (`diagram_tool/src/cli.rs`)

The `error_code()` function at line 138-158 already maps DAG-related errors to `dag_violation`:

```rust
pub fn error_code(err: &anyhow::Error) -> String {
    let msg = err.to_string().to_lowercase();
    // Check more specific patterns before general ones
    if msg.contains("dag") || msg.contains("cycle") {
        String::from("dag_violation")
    } else if msg.contains("dangling") || msg.contains("edge-dangling") {
        String::from("dangling_reference")
    } // ...
}
```

### Validation Pipeline (`diagram_tool/src/models/validation.rs`)

The `validate_document_data()` function detects DAG cycles:

- Line 85-90: Calls `validate_dag()` and creates a `ValidationIssue` with code `"dag-cycle"`
- The error message contains "DAG" and "cycle" which triggers the `dag_violation` code

### DAG Validation (`diagram_tool/src/models/dag.rs`)

The `validate_dag()` function uses Kahn's algorithm to detect cycles:
- Returns `CycleError::CycleDetected(EdgeId)` when a cycle is found
- Self-loops are detected as cycles (single-node cycles)

## Verification Results

### Unit Tests (cli_events_tests.rs)

- `given_dag_cycle_error_when_error_code_called_then_returns_dag_violation` - PASS

### End-to-End Tests (cli_e2e.rs)

- `given_dag_cycle_document_when_validate_runs_then_it_fails_with_dag_error` - PASS
- `given_self_loop_edge_when_validate_runs_then_it_fails_with_dag_error` - PASS

### Manual Verification

```bash
$ ./target/release/diagram_tool validate --input cycle.json
{"event":"error","command":"validate","ok":false,"code":"dag_violation",...}
EXIT: 1

$ ./target/release/diagram_tool validate --input self-loop.json
{"event":"error","command":"validate","ok":false,"code":"dag_violation",...}
EXIT: 1
```

## Files Examined (No Changes Required)

- `diagram_tool/src/cli.rs` - error_code() function already correct
- `diagram_tool/src/models/validation.rs` - DAG cycle detection already correct
- `diagram_tool/src/models/dag.rs` - Kahn's algorithm already correct
- `diagram_tool/tests/cli_e2e.rs` - Tests already passing
- `diagram_tool/src/cli_events_tests.rs` - Unit tests already passing

## Conclusion

This bead was a verification task. The `dag_violation` error code is correctly implemented and all tests pass. No code changes were required.
