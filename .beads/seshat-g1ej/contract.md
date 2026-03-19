# Contract Specification: seshat-g1ej

## Context

- **Feature**: Bug fix - Remove unwrap/expect panic calls
- **Bug 1**: `orthogonal_route` wrapper in `routing.rs` line 171 silently swallows errors
- **Bug 2**: `StoreBridge` has `expect()` calls that panic instead of returning errors
- **Assumptions**:
  - `compute_orthogonal_route` already returns `Result<OrthogonalRoute, RoutingError>` correctly
  - The `orthogonal_route` wrapper is the only broken call site
  - `StoreBridge::spawn_async_pool` may already return `Result` - need to check

## Bug 1: orthogonal_route Silent Error Swallowing

### Current Behavior (Bug)
```rust
pub fn orthogonal_route(from: Point, to: Point) -> OrthogonalRoute {
    compute_orthogonal_route(from, to).unwrap_or_else(|_e| OrthogonalRoute { points: vec![] })
}
```
When `compute_orthogonal_route` returns `Err`, the error is silently dropped and an empty route is returned.

### Expected Behavior (Fix)
Option A: `orthogonal_route` returns `Result<OrthogonalRoute, RoutingError>` (breaking change to callers)
Option B: `orthogonal_route` logs the error and still returns empty route (documented behavior)
Option C: Remove `orthogonal_route` entirely if callers can be updated

### Preconditions for `compute_orthogonal_route` (inherited)
- **P1**: `from.x` and `from.y` must be finite
- **P2**: `to.x` and `to.y` must be finite
- **P3**: `from` and `to` must not be identical (within `TOLERANCE`)

### Postconditions for `orthogonal_route`
- **Q1**: If input is valid, returns non-empty `OrthogonalRoute`
- **Q2**: If input is invalid (P1-P3 violated), does NOT silently swallow error

### Violation Examples
- **VIOLATES Q2**: `orthogonal_route(Point::new(f64::NAN, 0.0), Point::new(1.0, 2.0))` -- currently returns empty `OrthogonalRoute { points: vec![] }` instead of propagating error

---

## Bug 2: StoreBridge expect() Calls

### Current Behavior (Bug)
`StoreBridge` methods use `expect()` on `self.pool.as_ref().unwrap()` which panics if pool was never initialized.

### Expected Behavior (Fix)
Return `Err(BridgeError::PoolNotInitialized)` instead of panicking.

### Preconditions for StoreBridge methods
- **P8**: The `StoreBridge` must have been successfully spawned (pool initialized)

### Error Taxonomy for StoreBridge
| Variant | Trigger | Semantic Meaning |
|---------|---------|------------------|
| `PoolNotInitialized` | P8 violated | Pool was never initialized |
| `RuntimeNotRunning` | I3 violated | Bridge used after shutdown |

### Invariants
- **I3**: `StoreBridge` pool field is never accessed after `shutdown()` completes

### Violation Examples
- **VIOLATES P8**: Calling `append_event_sync` on a `StoreBridge` where `spawn_async_pool` failed -- currently panics with `unwrap()` on `None` -- **MUST return `Err(BridgeError::PoolNotInitialized)`**

---

## Non-goals

- [ ] Modifying protected tests in `geometry/tests/`
- [ ] Graph routing error variants (SourceNotFound, TargetNotFound, etc.) - these are pre-existing, not part of this bug fix
- [ ] Comprehensive happy path testing of `compute_orthogonal_route` (already works)
- [ ] Full StoreBridge integration tests (not broken)
