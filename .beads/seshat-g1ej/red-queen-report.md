# Red Queen Adversarial Testing Report: seshat-g1ej

## Session Info

- **Bead**: seshat-g1ej - Fix unwrap() in geometry routing and store bridge
- **Date**: 2026-03-18
- **Adversarial Testing**: Complete
- **Beads Created**: 4

---

## Executive Summary

Adversarial testing found **4 issues** during attack phase:

| ID | Severity | Issue | Bead |
|----|----------|-------|------|
| RQ-001 | P0 | orthogonal_route silently swallows errors | seshat-0v2h |
| RQ-002 | P0 | StoreBridge has no runtime tests - PoolNotInitialized fix unverified | seshat-3z7f |
| RQ-003 | P1 | orthogonal_route documentation contradicts implementation | seshat-62eo |
| RQ-004 | P2 | compute_orthogonal_route_avoiding produces NaN routes with NaN obstacles | seshat-cdhy |

---

## Issue RQ-001: orthogonal_route Silent Error Swallowing (P0)

**Bead**: `seshat-0v2h`  
**Severity**: CRITICAL (P0)  
**Status**: Contract violation

### Description

The `orthogonal_route` wrapper function (routing.rs:181-189) returns an empty `OrthogonalRoute { points: vec![] }` on ANY error from `compute_orthogonal_route`. This violates contract Q2 which states:

> Q2: If input is invalid (P1-P3 violated), does NOT silently swallow error

### Contract Violation Details

The contract specifies Option A, B, or C for fixing the error swallowing. The implementation chose Option B (return empty route but "not silently swallowed"). However, returning empty route IS silently swallowing the error:

1. Callers CANNOT distinguish between:
   - A valid empty route (conceptually possible in domain)
   - An error case that returned empty route

2. The error information is LOST - callers have no way to know an error occurred

### Attack Results

```rust
// Test 1: NaN input
orthogonal_route(Point::new(f64::NAN, 0.0), Point::new(1.0, 2.0))
// Returns: OrthogonalRoute { points: vec![] }
// compute_orthogonal_route says: Err(InvalidEndpoint)
// ISSUE: Error silently swallowed!

// Test 2: Infinity input
orthogonal_route(Point::new(f64::INFINITY, 0.0), Point::new(1.0, 2.0))
// Returns: OrthogonalRoute { points: vec![] }
// compute_orthogonal_route says: Err(InvalidEndpoint)
// ISSUE: Error silently swallowed!

// Test 3: Degenerate points
orthogonal_route(Point::new(5.0, 5.0), Point::new(5.0, 5.0))
// Returns: OrthogonalRoute { points: vec![] }
// compute_orthogonal_route says: Err(DegenerateRoute)
// ISSUE: Error silently swallowed!
```

### Impact

- **Data Integrity**: Wrong output looks correct to callers
- **Debugging**: Errors are invisible to calling code
- **Contract**: Q2 explicitly violated

---

## Issue RQ-002: StoreBridge Has No Runtime Tests (P0)

**Bead**: `seshat-3z7f`  
**Severity**: CRITICAL (P0)  
**Status**: Testing gap

### Description

ALL StoreBridge tests in `store_bridge.rs` are gated behind `#[cfg(kani)]`:

- Line 149: `#[cfg(kani)]` test_spawn_and_shutdown
- Line 170: `#[cfg(kani)]` test_append_event_sync
- Line 181: `#[cfg(kani)]` test_fetch_events_since_sync
- Line 223: `#[cfg(kani)]` test_append_batch_sync
- Line 246: `#[cfg(kani)]` test_append_idempotent_sync

### Impact

```bash
$ cargo test --package diagram_tool store_bridge
# NO TESTS RUN - all gated behind #[cfg(kani)]
```

1. **PoolNotInitialized fix is unverified at runtime**: The fix (changing `pool` to `Option<SqlitePool>` and returning proper errors) CANNOT be verified under `cargo test`

2. **No regression protection**: Future changes to StoreBridge could reintroduce panics without detection

3. **Kani-only verification**: Formal verification (kani) runs separately, not in standard CI

### Verification

```rust
// run_async method properly returns PoolNotInitialized error
fn run_async<F, Fut, R>(&self, f: F) -> Result<R, BridgeError>
where
    F: FnOnce(sqlx::SqlitePool) -> Fut,
    Fut: std::future::Future<Output = Result<R, AsyncStoreError>>,
{
    let pool = self.pool.as_ref().ok_or(BridgeError::PoolNotInitialized)?;
    // ...
}
```

The implementation looks correct, but there are NO RUNTIME TESTS to verify this works.

---

## Issue RQ-003: Documentation Contradicts Implementation (P1)

**Bead**: `seshat-62eo`  
**Severity**: MAJOR (P1)  
**Status**: Documentation bug

### Description

The doc comment for `orthogonal_route` at routing.rs:171 states:

> Errors result in an empty route, but the error is NOT silently swallowed.

This is FALSE. Returning empty route IS silently swallowing the error.

### Evidence

The comment also says:

> the explicit match handles it visibly in code

**False distinction**: "visible" vs "silent" error swallowing is meaningless if the error information is discarded. Both are silent swallowing from the caller's perspective.

### Location

`diagram_tool/src/geometry/routing.rs:168-175`:

```rust
/// Wrapper for `compute_orthogonal_route` that provides backward compatibility.
///
/// This wrapper exists for legacy callers that expect `OrthogonalRoute` directly.
/// Errors result in an empty route, but the error is NOT silently swallowed -
/// the explicit match handles it visibly in code.
```

---

## Issue RQ-004: NaN Obstacle Coordinates Produce NaN Routes (P2)

**Bead**: `seshat-cdhy`  
**Severity**: MINOR (P2)  
**Status**: Latent bug

### Description

If `compute_orthogonal_route_avoiding` is called with an obstacle containing NaN coordinates:

1. `is_inside()` returns `false` (NaN comparisons are always false)
2. `build_detour_route()` produces route with NaN points
3. `route_intersects()` returns `false` (NaN comparisons)
4. Final route contains NaN points

### Attack Example

```rust
let obstacle = AABB::new(f64::NAN, 0.0, 100.0, 100.0);
let route = compute_orthogonal_route_avoiding(
    Point::new(0.0, 0.0),
    Point::new(200.0, 200.0),
    &obstacle
);
// Returns Ok with NaN points in route - caller cannot tell
```

### Impact

- Lower severity because `is_inside` validation catches NaN endpoints
- Obstacle NaN is a separate validation concern
- Could cause issues in rendering or downstream calculations

---

## Attack Categories Executed

### Category 1: Happy Path Verification
- [x] `compute_orthogonal_route` works correctly with valid inputs
- [x] `compute_orthogonal_route_avoiding` works correctly with valid obstacle
- [x] StoreBridge spawn/shutdown work in kani proofs (not runtime)

### Category 2: Input Boundary Attacks
- [x] NaN coordinates - orthogonal_route returns empty, error lost
- [x] Infinity coordinates - orthogonal_route returns empty, error lost
- [x] Degenerate points - orthogonal_route returns empty, error lost
- [x] Very large coordinates - handled gracefully
- [x] Sub-pixel differences - treated as degenerate

### Category 3: State Attacks
- [x] StoreBridge after shutdown - no runtime test to verify
- [x] Pool never initialized - no runtime test to verify

### Category 4: Output Contract Attacks
- [x] Empty route returned on error - indistinguishable from valid empty route
- [x] Error information completely lost

### Category 5: Cross-Command Consistency
- [x] `compute_orthogonal_route` returns Result (correct)
- [x] `orthogonal_route` returns empty route on error (incorrect - violates Q2)

---

## Bead Summary

| Bead ID | Title | Priority | Status |
|---------|-------|----------|--------|
| seshat-0v2h | orthogonal_route: Contract violation - silent error swallowing | P0 | Open |
| seshat-3z7f | StoreBridge: No runtime tests - PoolNotInitialized fix unverified | P0 | Open |
| seshat-62eo | orthogonal_route: Documentation claim contradicts implementation | P1 | Open |
| seshat-cdhy | compute_orthogonal_route_avoiding: Missing validation for NaN obstacle | P2 | Open |

---

## Red Queen Gate Status

**NOT CLEARED** - Issues remain open:

1. [ ] seshat-0v2h (P0) must be fixed before release
2. [ ] seshat-3z7f (P0) must have runtime tests added
3. [ ] seshat-62eo (P1) documentation must be corrected
4. [ ] seshat-cdhy (P2) should be addressed or documented

---

## Recommendations

### Immediate (P0)

1. **For orthogonal_route**: Change return type to `Result<OrthogonalRoute, RoutingError>` (Option A from contract). Update all callers to handle the Result. This is the only way to fully satisfy Q2.

2. **For StoreBridge**: Add runtime tests (not `#[cfg(kani)]`) that verify:
   - PoolNotInitialized error is returned when pool is None
   - Operations work correctly when pool is Some
   - Shutdown properly closes pool

### Short-term (P1)

3. **Documentation**: Fix the `orthogonal_route` doc comment to accurately describe behavior. If the wrapper is deprecated, add `#[deprecated]` attribute.

### Long-term (P2)

4. **Obstacle validation**: Consider adding validation for obstacle coordinates in `compute_orthogonal_route_avoiding`.

---

## Conclusion

Adversarial testing identified 2 critical (P0) issues that violate the contract and 2 additional issues. The orthogonal_route wrapper violates contract Q2 by silently swallowing errors. The StoreBridge fix cannot be verified at runtime due to missing tests.

**Release Recommendation**: DO NOT MERGE until P0 issues are addressed.

---

*Report generated by Red Queen adversarial testing*
*Beads tracked in: br database*