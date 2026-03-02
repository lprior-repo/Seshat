---
bead_id: bd-2vq
bead_title: edge-case-bdd-tests-snapshot-recovery
phase: p2
updated_at: 2026-03-02T05:57:00Z
---

# Verification: Snapshot Recovery BDD Tests

## Test Count: 7 BDD tests

## Coverage

| Scenario | Test |
|----------|------|
| Staleness detection | given_stale_snapshot_when_load_projection_then_returns_stale_error |
| Corrupted payload | given_corrupted_payload_when_load_projection_then_returns_serialization_error |
| Truncated JSON | given_truncated_json_payload_when_load_projection_then_returns_serialization_error |
| Semantic invalidity | given_semantically_invalid_payload_when_load_projection_then_returns_error |
| Incompatible format | given_incompatible_snapshot_format_when_load_projection_then_handles_gracefully |
| Missing metadata | given_snapshot_with_missing_metadata_fields_when_load_then_returns_serialization_error |
| Missing nodes field | given_snapshot_missing_nodes_field_when_load_then_returns_serialization_error |

## Execution

```
cargo test -p diagram_tool given_
```
Result: 683 tests pass including all snapshot recovery tests.

