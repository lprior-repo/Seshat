# Contract: bd-3ve - storage-bootstrap: create sqlite wal schema and pragma bootstrap

bead_id: bd-3ve
bead_title: storage-bootstrap: create sqlite wal schema and pragma bootstrap
phase: p0
updated_at: 2026-03-01T13:20:25Z

---

## EARS Requirements

### Ubiquitous
- THE SYSTEM SHALL persist all accepted diagram mutations as append-only domain events

### Event-Driven
- WHEN a mutation operation is submitted, THE SYSTEM SHALL validate and either atomically append or reject the operation

### Unwanted
- IF the operation cannot satisfy preconditions, THE SYSTEM SHALL NOT mutate durable state partially (partial writes break replay and trust)

---

## Preconditions

- auth_required: false
- required_inputs: []
- system_state:
  - SQLite connection is open with WAL enabled and synchronous FULL
  - Rust Contract Signature: fn bootstrap_store(db_path: &Path) -> Result<StoreBootstrap, StoreError>
  - Rust Error Contract: enum StoreError { Io, Sqlite, InvalidPragma, SchemaVersionMismatch }

---

## Postconditions

- state_changes:
  - Rust Postcondition Signature: fn current_store_config(conn: &Connection) -> Result<StoreConfig, StoreError>
  - Accepted operations increment revision monotonically by exactly one
  - Rejected operations return structured error codes without side effects
- return_guarantees: []

---

## Invariants

- Event log remains append-only and replay deterministic
- Idempotent operation IDs never produce duplicate durable mutations
- Human-authored operations keep priority over conflicting AI operations

---

## Implementation Tasks

### Phase 0: Research
- Read existing model and CLI wiring before writing tests

### Phase 1: Tests First
- Write failing tests for happy and error paths

### Phase 2: Implementation
- Create bootstrap module and deterministic schema migration v1
- Enforce PRAGMA journal_mode=WAL and synchronous=FULL on open

---

## AI Hints

- DO: Use functional patterns (map, and_then, ?), Return Result<T, Error>, READ files before modifying
- DO NOT: Use unwrap/expect, panic!/todo!/unimplemented!, modify clippy config
- Constitution: Zero unwrap law (NEVER use .unwrap or .expect), Test first (Tests MUST exist before implementation)

---

## Completion Checklist

- [ ] All acceptance tests written and passing
- [ ] All error path tests written and passing
- [ ] Implementation uses Result<T, Error> throughout
- [ ] Zero unwrap or expect calls
- [ ] cargo test passes
- [ ] cargo clippy passes
