# Contract: bd-3lk - event-envelope: define canonical operation envelope and serde schema

bead_id: bd-3lk
bead_title: event-envelope: define canonical operation envelope and serde schema
phase: p0
updated_at: 2026-03-01T13:24:09Z

---

## Preconditions

- auth_required: false
- required_inputs: []
- system_state:
  - SQLite connection is open with WAL enabled and synchronous FULL
  - Rust Contract Signature: fn parse_event_envelope(input: &str) -> Result<EventEnvelope, ContractError>
  - Rust Error Contract: enum ContractError { InvalidJson, UnknownOpType, MissingField, InvalidAuthor }

---

## Postconditions

- state_changes:
  - Rust Postcondition Signature: fn encode_event_envelope(op: &EventEnvelope) -> Result<String, ContractError>
  - Accepted operations increment revision monotonically by exactly one
  - Rejected operations return structured error codes without side effects

---

## Invariants

- Event log remains append-only and replay deterministic
- Idempotent operation IDs never produce duplicate durable mutations
- Human-authored operations keep priority over conflicting AI operations

---

## Implementation Tasks

### Phase 1: Tests First
- Write failing tests for happy and error paths

### Phase 2: Implementation
- Add EventEnvelope and Operation enums with serde and schema tests
- Reject unknown op_type or malformed payload with typed errors

---

## AI Hints

- DO: Use functional patterns (map, and_then, ?), Return Result<T, Error>
- DO NOT: Use unwrap/expect, panic!/todo!/unimplemented!
- Constitution: Zero unwrap law, Test first

---

## Completion Checklist

- [ ] All tests written and passing
- [ ] Implementation uses Result<T, Error> throughout
- [ ] Zero unwrap or expect calls
- [ ] cargo test passes
