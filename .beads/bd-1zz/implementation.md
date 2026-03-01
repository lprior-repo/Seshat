# Implementation: bd-1zz - contract-envelope

## Summary
Implemented the EventEnvelope and Author metadata types with strict validation for the contract-envelope bead.

## Files Changed

### Created: `diagram_tool/src/models/envelope.rs`
New module defining:
- `ContractError` enum with variants: `InvalidJson`, `MissingField`, `InvalidAuthor`, `UnknownOpType`
- `Author` struct with `id`, `name`, and optional `email` fields
- `OpType` enum with `Create`, `Update`, `Delete`, `Migrate` variants  
- `EventEnvelope` struct containing `id`, `operation`, `author`, `timestamp`, and optional `payload`
- `decode_envelope(raw: &str) -> Result<EventEnvelope, ContractError>` - parses JSON with strict validation
- `encode_envelope(op: &EventEnvelope) -> Result<String, ContractError>` - serializes to JSON

### Modified: `diagram_tool/src/models/mod.rs`
Added `pub mod envelope;` to export the new module.

## Contract Compliance

### Preconditions (Satisfied)
- ✅ `fn decode_envelope(raw: &str) -> Result<EventEnvelope, ContractError>` signature implemented
- ✅ `enum ContractError { InvalidJson, MissingField, InvalidAuthor, UnknownOpType }` implemented

### Postconditions (Satisfied)
- ✅ `fn encode_envelope(op: &EventEnvelope) -> Result<String, ContractError>` implemented
- ✅ All operations use typed Result errors

### Invariants (Satisfied)
- ✅ No migration path introduced
- ✅ No dual-write compatibility path
- ✅ All fallible operations use typed Result errors

## Implementation Details

### Design Decisions
1. **Parse-don't-validate**: The decode function parses JSON directly into the validated types, rejecting invalid data at the boundary
2. **Strict author validation**: Author must have both `id` and `name` fields; missing either returns `InvalidAuthor` error
3. **Payload null handling**: JSON `null` is treated as Rust `None` for optional payload
4. **Operation type validation**: Unknown operation types return `UnknownOpType` error

### Testing
- 11 comprehensive tests covering:
  - Valid JSON parsing
  - Invalid JSON error handling
  - Missing required fields (id, author, timestamp, op)
  - Invalid author (missing name)
  - Unknown operation types
  - All operation types (create, update, delete, migrate)
  - Author with and without email
  - Roundtrip encoding/decoding with and without payload

## Quality Gates
- ✅ All 11 tests pass
- ✅ No clippy warnings in new code
- ✅ Follows functional Rust patterns (no unwrap/expect/panic in source)
