# Contract: bd-2hz - contract-optypes: define domain operation enum and payload shapes

## Metadata
- bead_id: bd-2hz
- bead_title: contract-optypes: define domain operation enum and payload shapes
- phase: p0
- updated_at: 2026-03-01T13:20:00Z

## Preconditions
- Rust Contract Signature: `fn parse_domain_op(raw: &str) -> Result<DomainOp, ContractError>`
- Rust Error Contract: `enum ContractError { UnknownOpType, InvalidPayload, MissingField }`
- Legacy code path for this slice is identified and removable in one commit

## Postconditions
- Rust Contract Signature: `fn domain_op_kind(op: &DomainOp) -> OpKind`
- Legacy path is deleted or unreachable by compile-time guarantees
- Replacement path passes focused tests with no fallback to removed code

## Invariants
- No migration path is introduced
- No dual-write compatibility path exists
- All fallible operations use typed Result errors

## Implementation Tasks
1. Add op enum and payload structs
2. Add exhaustive match test to prevent hidden op types
