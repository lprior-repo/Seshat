# Contract: bd-1zz - contract-envelope: define EventEnvelope and Author metadata

## Metadata
- bead_id: bd-1zz
- bead_title: contract-envelope: define EventEnvelope and Author metadata
- phase: p0
- updated_at: 2026-03-01T13:04:00Z

## Preconditions
- Rust Contract Signature: `fn decode_envelope(raw: &str) -> Result<EventEnvelope, ContractError>`
- Rust Error Contract: `enum ContractError { InvalidJson, MissingField, InvalidAuthor, UnknownOpType }`
- Legacy code path for this slice is identified and removable in one commit

## Postconditions
- Rust Postcondition Signature: `fn encode_envelope(op: &EventEnvelope) -> Result<String, ContractError>`
- Legacy path is deleted or unreachable by compile-time guarantees
- Replacement path passes focused tests with no fallback to removed code

## Invariants
- No migration path is introduced
- No dual-write compatibility path exists
- All fallible operations use typed Result errors

## Implementation Tasks
1. Create envelope types and serde impls
2. Add strict validation tests for required fields
