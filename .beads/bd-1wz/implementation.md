# Implementation: bd-1wz - cli: fix dangling_reference and stale_revision codes

## Metadata
- bead_id: bd-1wz
- bead_title: cli: fix dangling_reference and stale_revision codes
- phase: p1
- updated_at: 2026-03-01T23:10:00Z

## Summary

Fixed the `error_code` function in `cli.rs` to recognize "stale_revision" pattern in error messages.
This ensures that stale revision errors produce consistent error codes throughout the CLI event stream
and correct exit codes.

## Changes Made

### 1. diagram_tool/src/cli.rs

Added "stale_revision" pattern to the `error_code` function (line 145):

```rust
} else if msg.contains("stale_revision") {
    String::from("stale_revision")
}
```

The pattern is placed after `dangling_reference` but before `schema` to maintain specificity ordering.

### 2. diagram_tool/src/cli_events_tests.rs

Added three new unit tests:

1. `given_stale_revision_error_when_error_code_called_then_returns_stale_revision`
   - Verifies that error messages containing "stale_revision" map to the correct code

2. `given_stale_revision_error_when_exit_code_called_then_returns_1`
   - Verifies that stale revision errors return exit code 1 (business logic error)

3. `given_dangling_reference_error_when_error_code_called_then_returns_dangling_reference`
   - Verifies that dangling reference errors (edge-dangling pattern) map correctly

## Verification Results

### Before Fix
```
{"event":"error",...,"code":"command_error","message":"stale_revision: test failed..."}
{"event":"finish",...,"code":"command_error"}
Exit code: 2
```

### After Fix
```
{"event":"error",...,"code":"stale_revision","message":"stale_revision: test failed..."}
{"event":"finish",...,"code":"stale_revision"}
Exit code: 1
```

## Test Results

All tests pass:
- 16 unit tests in cli_event_tests module
- 13 e2e tests in cli_e2e

## Files Modified
- diagram_tool/src/cli.rs (1 line added)
- diagram_tool/src/cli_events_tests.rs (30 lines added)
