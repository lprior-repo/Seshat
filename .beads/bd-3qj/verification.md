bead_id: bd-3qj
bead_title: io-json-export: export canonical diagram json from projection
phase: p3
updated_at: 2026-03-01T21:52:00Z

# Verification: bd-3qj - io-json-export

## QA Evidence

### Export Tests
```
running 14 tests
test models::export::tests::given_empty_projection_when_exporting_then_returns_valid_json ... ok
test models::export::tests::given_projection_with_nodes_when_exporting_then_includes_nodes_in_json ... ok
test models::export::tests::given_projection_with_edges_when_exporting_then_includes_edges_in_json ... ok
test models::export::tests::given_valid_json_when_validating_schema_then_succeeds ... ok
test models::export::tests::given_invalid_json_when_validating_schema_then_fails ... ok
test result: ok. 14 passed; 0 failed
```

### Clippy
```
cargo clippy -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## Sign-off
- Actor: qa-enforcer
- Verified at: 2026-03-01T21:52:00Z
