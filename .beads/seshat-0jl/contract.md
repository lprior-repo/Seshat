# Contract Specification: Graph Cycle Policy Engine

## Context
- **Feature**: Extract Graph Cycle Policy Engine from projection.rs
- **Domain terms**:
  - `CyclePolicy` - Enum controlling whether cycles are allowed/denied in graph
  - `enforce_cycle_policy` - Validates projection against cycle policy
  - `apply_policy_op` - Applies domain operation with policy enforcement
- **Assumptions**:
  - The graph is directed (edges have source/target)
  - Cycle detection is done via DAG validation
  - Policy enforcement happens after state mutation
- **Open questions**:
  - Should CyclePolicy be moved to its own module or stay in projection?
  - What is the dependency on the dag module?

## Preconditions
- [ ] `enforce_cycle_policy` requires a valid `DiagramProjection` with initialized `cycle_policy` field
- [ ] `apply_policy_op` requires valid state and non-null operation
- [ ] Edge operations require source and target nodes to exist

## Postconditions
- [ ] `enforce_cycle_policy` returns `Ok(())` when policy is `Allow` regardless of graph state
- [ ] `enforce_cycle_policy` returns `Err(CycleViolation)` when policy is `Deny` and cycle detected
- [ ] `apply_policy_op` returns new state when operation succeeds
- [ ] `apply_policy_op` returns error when operation would violate policy

## Invariants
- [ ] `CyclePolicy` is always either `Allow` or `Deny` (no invalid state)
- [ ] `enforce_cycle_policy` does not modify the projection state
- [ ] When `Deny` policy is set, all edge additions are validated against DAG constraint

## Error Taxonomy
- `ReplayError::CycleViolation` - Graph contains a cycle when policy is `Deny`
- `ReplayError::PolicyMissing` - Cycle policy field not initialized
- `ReplayError::PolicyViolation` - Operation violates policy constraints
- `ReplayError::InvalidEvent` - Malformed event input
- `ReplayError::InvariantViolation` - Internal state invariant violated

## Contract Signatures
```rust
pub fn enforce_cycle_policy(state: &DiagramProjection) -> Result<(), ReplayError>
pub fn apply_policy_op(state: DiagramProjection, op: &DomainOp) -> Result<DiagramProjection, ReplayError>
pub enum CyclePolicy { Allow, Deny }
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| state is valid | Runtime-checked constructor | `DiagramProjection::new() -> Result` |
| cycle_policy initialized | Default value | `CyclePolicy::default()` |
| edge nodes exist | Runtime validation | `has_node()` check |
| DAG constraint | Runtime validation | `validate_dag()` |

## Violation Examples (REQUIRED)
- VIOLATES <P1>: `enforce_cycle_policy(&projection_with_cycle)` where policy is `Deny` -- should produce `Err(ReplayError::CycleViolation(...))`
- VIOLATES <P2>: `apply_policy_op(state, &edge_op)` where edge creates cycle and policy` -- should produce is `Deny `Err(ReplayError::CycleViolation(...))`
- VIOLATES <Q1>: State after `apply_policy_op` with violating operation -- should NOT contain the edge that creates cycle

## Ownership Contracts (Rust-specific)
- `enforce_cycle_policy(&state)`: Shared borrow - no mutation
- `apply_policy_op(state, op)`: Takes ownership of `state`, returns new owned `DiagramProjection`
- `CyclePolicy`: Simple enum, Copy type, no heap allocation

## Non-goals
- [ ] Implementing the actual cycle detection algorithm (delegated to dag module)
- [ ] Supporting multiple cycle policies (only Allow/Deny)
- [ ] Serialization format changes
