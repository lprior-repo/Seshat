# Martin Fowler Test Plan: seshat-g1ej

## Bug Focus

This test plan focuses ONLY on:
1. Verifying `orthogonal_route` no longer silently swallows errors
2. Verifying `StoreBridge` returns proper errors instead of panicking

---

## Bug 1: orthogonal_route Error Propagation Tests

### Scenario 1: orthogonal_route with NaN endpoint returns error (not empty route)
Given: `from = Point::new(f64::NAN, 0.0)`, `to = Point::new(1.0, 2.0)`
When: `orthogonal_route(from, to)` is called
Then: Returns `Err(RoutingError::InvalidEndpoint)`
And: Does NOT silently return empty `OrthogonalRoute { points: vec![] }`
And: Does NOT panic

### Scenario 2: orthogonal_route with Infinity endpoint returns error (not empty route)
Given: `from = Point::new(1.0, 2.0)`, `to = Point::new(f64::INFINITY, 3.0)`
When: `orthogonal_route(from, to)` is called
Then: Returns `Err(RoutingError::InvalidEndpoint)` -- does NOT silently return empty route
And: Does NOT panic

### Scenario 3: orthogonal_route with degenerate points returns error (not empty route)
Given: `from = Point::new(1.0, 1.0)`, `to = Point::new(1.0 + 1e-11, 1.0 + 1e-11)`
When: `orthogonal_route(from, to)` is called
Then: Returns `Err(RoutingError::DegenerateRoute)` -- does NOT silently return empty route
And: Does NOT panic

### Scenario 4: orthogonal_route with valid input returns non-empty route
Given: `from = Point::new(0.0, 0.0)`, `to = Point::new(10.0, 0.0)`
When: `orthogonal_route(from, to)` is called
Then: Returns `Ok(OrthogonalRoute)` with non-empty `points` vector
And: `route.points.first()` equals `from`
And: `route.points.last()` equals `to`

---

## Bug 2: StoreBridge PoolNotInitialized Tests

### Scenario 5: append_event_sync on unspawned bridge returns PoolNotInitialized (not panic)
Given: A `StoreBridge` created via `spawn_async_pool` that returned `Err`, leaving pool as `None`
When: `append_event_sync(&envelope, None)` is called on this failed bridge
Then: Returns `Err(BridgeError::PoolNotInitialized)`
And: Does NOT panic with "unwrap on None"

### Scenario 6: append_batch_sync on unspawned bridge returns PoolNotInitialized (not panic)
Given: A `StoreBridge` where `spawn_async_pool` failed
When: `append_batch_sync(&[envelope1, envelope2], None)` is called
Then: Returns `Err(BridgeError::PoolNotInitialized)`
And: Does NOT panic

### Scenario 7: append_idempotent_sync on unspawned bridge returns PoolNotInitialized (not panic)
Given: A `StoreBridge` where `spawn_async_pool` failed
When: `append_idempotent_sync(envelope)` is called
Then: Returns `Err(BridgeError::PoolNotInitialized)`
And: Does NOT panic

### Scenario 8: fetch_events_since_sync on unspawned bridge returns PoolNotInitialized (not panic)
Given: A `StoreBridge` where `spawn_async_pool` failed
When: `fetch_events_since_sync(0)` is called
Then: Returns `Err(BridgeError::PoolNotInitialized)`
And: Does NOT panic

---

## Contract Verification: No Panic Assertions

### Scenario 9: verify_orthogonal_route_does_not_silently_swallow_errors
Given: Invalid input to `orthogonal_route` (NaN, Infinity, or degenerate points)
When: The function is called
Then: Returns `Err(...)` -- error propagated via Result return type
And: Does NOT silently return an empty route
And: Does NOT panic

### Scenario 10: verify_store_bridge_methods_do_not_panic_on_uninitialized_pool
Given: A `StoreBridge` with `pool = None`
When: Any of the sync methods are called
Then: All return `Err(BridgeError::PoolNotInitialized)` -- none panic

---

## Test Summary

| Test | Focus | Status |
|------|-------|--------|
| Scenario 1-3 | orthogonal_route error propagation | Core bug |
| Scenario 4 | orthogonal_route happy path | Sanity check |
| Scenario 5-8 | StoreBridge PoolNotInitialized | Core bug |
| Scenario 9-10 | Contract verification | Ensures no silent failures |

**Total: 10 tests** (realistic for a bug fix)
