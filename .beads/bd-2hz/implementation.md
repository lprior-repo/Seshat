# Implementation: bd-2hz - contract-optypes

## Summary

Implemented domain operation enum and payload shapes for the diagram tool's replay system.

## Files Changed

### diagram_tool/src/models/envelope.rs

Added the following types and functions:

1. **ContractError** - Extended with `InvalidPayload` variant
2. **OpKind** - Enum representing the category of domain operation:
   - `Node` - Operations on nodes
   - `Edge` - Operations on edges  
   - `Composite` - Multi-entity operations (group/ungroup)
   - `ZOrder` - Layer ordering operations

3. **DomainOp** - Enum representing all domain operations with payloads:
   - Node operations: `NodeAdd`, `NodeMove`, `NodeDelete`, `NodeRestore`
   - Edge operations: `EdgeConnect`, `EdgeDisconnect`
   - Z-order operations: `BringForward`, `SendBackward`, `BringToFront`, `SendToBack`
   - Composite operations: `Group`, `Ungroup`

4. **parse_domain_op(raw: &str) -> Result<DomainOp, ContractError>** - Parses JSON string into DomainOp

5. **domain_op_kind(op: &DomainOp) -> OpKind** - Returns the operation kind for a domain operation

6. **DomainOp::kind(&self) -> OpKind** - Method variant of domain_op_kind

## Contract Verification

- ✅ `fn parse_domain_op(raw: &str) -> Result<DomainOp, ContractError>` - Implemented
- ✅ `fn domain_op_kind(op: &DomainOp) -> OpKind` - Implemented
- ✅ `enum ContractError { UnknownOpType, InvalidPayload, MissingField }` - Extended with InvalidPayload

## Test Coverage

Added 24 new tests covering:
- Happy path: parsing all domain operation types
- Error path: invalid JSON, missing fields, unknown op types
- Exhaustive match test to prevent hidden op types
- Verification that method and free function return same results

## Design Decisions

- Used `#[serde(tag = "op")]` for DomainOp to enable JSON parsing with type discrimination
- All payload fields are required (no optional fields) to ensure parse-time validation
- Error handling uses Result throughout - no unwrap/expect/panic
- Pure functions in the core - no I/O or mutable state
