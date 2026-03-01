# Contract: bd-1wz - cli: fix dangling_reference and stale_revision codes

## Metadata
- bead_id: bd-1wz
- bead_title: cli: fix dangling_reference and stale_revision codes
- phase: p0
- updated_at: 2026-03-01T22:55:00Z

## Problem Statement

The `error_code` function in `cli.rs` does not recognize "stale_revision" in error messages,
causing the following issues when a stale revision error occurs during patch operations:

1. A duplicate error event is emitted with incorrect code "command_error" instead of "stale_revision"
2. The finish event has incorrect code "command_error" instead of "stale_revision"
3. The exit code is 2 (for command_error) when it should be 1 (for business logic errors)

The dangling_reference error code works correctly because the validation error messages
contain "dangling" which is recognized by the current pattern matching.

## Preconditions
- `diagram_tool/src/cli.rs` contains `error_code` function with pattern matching
- Patch command emits "stale_revision" code before returning anyhow error
- E2E tests exist for both error codes in `diagram_tool/tests/cli_e2e.rs`

## Postconditions
- `error_code` function recognizes "stale_revision" pattern in error messages
- Only ONE error event is emitted with correct code for stale_revision
- Finish event has correct code "stale_revision" for stale revision errors
- Exit code is 1 (not 2) for stale_revision errors
- All existing tests continue to pass

## Invariants
- No changes to error event structure or JSONL format
- No changes to existing error patterns (dangling, dag, schema, semantic, parse)
- Pattern order must remain: specific patterns before general ones
- Exit code policy: business logic errors return 1, command/parse errors return 2

## Acceptance Criteria
1. Given stale revision error, when `error_code` called, returns "stale_revision"
2. Given stale revision error, when CLI runs, emits exactly one error event with "stale_revision"
3. Given stale revision error, when CLI runs, finish event has "stale_revision" code
4. Given stale revision error, exit code is 1 (not 2)
5. All existing e2e tests pass

## Implementation Tasks
1. Add "stale_revision" pattern to `error_code` function before general patterns
2. Add unit test for stale_revision error code mapping
3. Verify all e2e tests pass
