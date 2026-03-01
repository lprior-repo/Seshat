# Verification: bd-260 - cli: fix dag_violation error code

bead_id: bd-260
bead_title: cli: fix dag_violation error code
phase: p3
updated_at: 2026-03-01T23:01:00Z

## Test Results

### Unit Tests

```
running 21 tests
test cli_events_tests::cli_event_tests::given_dag_cycle_error_when_error_code_called_then_returns_dag_violation ... ok
... (all 21 tests pass)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 853 filtered out
```

### End-to-End Tests

```
running 13 tests
test given_dag_cycle_document_when_validate_runs_then_it_fails_with_dag_error ... ok
test given_self_loop_edge_when_validate_runs_then_it_fails_with_dag_error ... ok
test given_dangling_edge_document_when_validate_runs_then_it_fails_with_dangling_error ... ok
... (all 13 tests pass)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Full Test Suite

```
test result: ok. 872 passed; 0 failed; 5 ignored; 0 measured; 0 measured; finished in 1.53s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

Total: 885 tests passed, 0 failed

## Manual CLI Verification

### DAG Cycle Test

```bash
$ echo '{"version":2,"revision":1,"document":{"nodes":{"n1":{...},"n2":{...}},"edges":{"e1":{"source":"n1","target":"n2",...},"e2":{"source":"n2","target":"n1",...}}}}' > cycle.json
$ ./target/release/diagram_tool validate --input cycle.json
{"event":"start","command":"validate","ok":true,"code":"start","message":null}
{"event":"stage","name":"validating","details":{"path":"cycle.json"}}
{"event":"error","command":"validate","ok":false,"code":"dag_violation","message":"...Cycle detected..."}
{"event":"finish","command":"validate","ok":false,"code":"dag_violation","message":null}
EXIT: 1
```

### Self-Loop Test

```bash
$ echo '{"version":2,"revision":1,"document":{"nodes":{"n1":{...}},"edges":{"e1":{"source":"n1","target":"n1",...}}}}' > self-loop.json
$ ./target/release/diagram_tool validate --input self-loop.json
{"event":"start","command":"validate","ok":true,"code":"start","message":null}
{"event":"stage","name":"validating","details":{"path":"self-loop.json"}}
{"event":"error","command":"validate","ok":false,"code":"dag_violation","message":"...Cycle detected..."}
{"event":"finish","command":"validate","ok":false,"code":"dag_violation","message":null}
EXIT: 1
```

## Contract Acceptance Criteria

| Criteria | Status | Evidence |
|----------|--------|----------|
| DAG cycle emits `dag_violation` | PASS | E2E test + manual verification |
| Self-loop emits `dag_violation` | PASS | E2E test + manual verification |
| Exit code is 1 | PASS | Manual verification |
| All tests pass | PASS | 885 tests passed |

## QA Assessment

The `dag_violation` error code is correctly implemented:

1. **Error code mapping** - `error_code()` function correctly maps messages containing "dag" or "cycle" to `dag_violation`
2. **Validation pipeline** - `validate_document_data()` correctly detects cycles via `validate_dag()`
3. **DAG algorithm** - Kahn's algorithm correctly detects both multi-node cycles and self-loops
4. **Exit codes** - Semantic validation errors return exit code 1 (not parse error code 2)

## Conclusion

VERIFICATION PASSED. The feature was already correctly implemented in prior work (bd-2uy).
