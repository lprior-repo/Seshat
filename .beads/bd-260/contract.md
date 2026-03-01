# Contract: bd-260 - cli: fix dag_violation error code

bead_id: bd-260
bead_title: cli: fix dag_violation error code
phase: p0
updated_at: 2026-03-01T22:55:00Z

## Summary

Ensure the CLI correctly emits `dag_violation` error code when DAG constraints are violated (cycles, self-loops).

## Preconditions

- `diagram_tool` binary exists with validate command
- JSONL event emission is implemented
- Error code mapping function exists in `cli.rs`

## Postconditions

- When a document with a DAG cycle is validated, the JSONL output contains an error event with `code: "dag_violation"`
- When a document with a self-loop edge is validated, the JSONL output contains an error event with `code: "dag_violation"`
- Exit code is 1 for DAG violations
- All related tests pass

## Invariants

- Error code mapping is deterministic
- `dag_violation` is returned for any error message containing "dag" or "cycle"
- Exit code is always 1 for semantic validation errors (not 2 for parse errors)

## Acceptance Tests

1. **given_dag_cycle_document_when_validate_runs_then_it_fails_with_dag_error**
   - Given: A document with nodes n1, n2 and edges e1: n1->n2, e2: n2->n1 (cycle)
   - When: validate command runs
   - Then: Exit code is non-zero and JSONL contains `{"event":"error","code":"dag_violation"}`

2. **given_self_loop_edge_when_validate_runs_then_it_fails_with_dag_error**
   - Given: A document with node n1 and edge e1: n1->n1 (self-loop)
   - When: validate command runs
   - Then: Exit code is non-zero and JSONL contains `{"event":"error","code":"dag_violation"}`

## Related Files

- `diagram_tool/src/cli.rs` - error_code() function
- `diagram_tool/src/models/validation.rs` - validate_document() function
- `diagram_tool/src/models/dag.rs` - validate_dag() function
- `diagram_tool/tests/cli_e2e.rs` - End-to-end tests
- `diagram_tool/src/cli_events_tests.rs` - Unit tests for error codes
