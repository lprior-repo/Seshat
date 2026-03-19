# Defects Document: seshat-g1ej

## DEFECT 1: Option B Implementation Missing Required Logging (CRITICAL)

**Severity:** P0 - Contract Violation  
**File:** `diagram_tool/src/geometry/routing.rs`  
**Lines:** 181-190

### Contract Requirement
Option B specifies: "`orthogonal_route` **logs the error** and still returns empty route (documented behavior)"

### Actual Implementation
```rust
pub fn orthogonal_route(from: Point, to: Point) -> OrthogonalRoute {
    match compute_orthogonal_route(from, to) {
        Ok(route) => route,
        Err(_e) => {
            // Error is explicitly handled here (Q2: not silently swallowed)
            OrthogonalRoute { points: vec![] }
        }
    }
}
```

### Problem
No logging occurs. The error `_e` is matched and dropped. The contract explicitly requires "logs the error" but there is no `log::`, `tracing::`, or similar logging statement anywhere in the file.

The comment at line 185 is **misleading** - it says "Error is explicitly handled" but "handling" without logging is functionally identical to the original "silently swallowed" behavior. The error is not visible to operators, cannot be observed in logs, and provides zero debugging value.

### Contract Violation
The implementation claims to use Option B but violates Option B's explicit requirement for **logging**.

### Fix Required
Add actual logging when the error case is hit. For example:
```rust
Err(e) => {
    tracing::debug!("orthogonal_route failed, returning empty: {:?}", e);
    OrthogonalRoute { points: vec![] }
}
```

Or change to Option A (return `Result`) if logging infrastructure is not available.

---

## DEFECT 2: PoolNotInitialized Runtime Test Coverage Gap

**Severity:** P2 - Verification Gap  
**File:** `diagram_tool/src/store_bridge.rs`  
**Lines:** 59-67, 144-269

### Red Queen Finding
`seshat-3z7f` (P0): StoreBridge has no runtime tests for PoolNotInitialized path

### Analysis
The tests in `store_bridge.rs` (lines 144-269) are gated with `#[cfg(kani)]` and `#[kani::proof)]`. These tests only compile and run under Kani verification, **NOT** during normal `cargo test`.

The `ok_or(BridgeError::PoolNotInitialized)?` at line 64 in `run_async` is **dead code from the perspective of runtime tests** - no test ever exercises the `None` pool path.

### Evidence
```bash
$ cargo test -p diagram_tool 2>&1 | grep -i store_bridge
# (no results - StoreBridge tests don't run)
```

### Consequence
The `PoolNotInitialized` error handling logic is functionally correct by code review, but has zero runtime test coverage. A future regression could break this path undetected.

### Fix Required
Add a `#[cfg(test)]` runtime test that:
1. Creates a `StoreBridge` where `spawn_async_pool` fails OR
2. Directly constructs a `StoreBridge` with `pool: None` and verifies proper error is returned

Example:
```rust
#[test]
fn test_run_async_returns_pool_not_initialized_when_pool_is_none() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    
    let bridge = StoreBridge {
        pool: None,
        runtime,
    };
    
    let result = bridge.fetch_events_since_sync(0);
    assert!(matches!(result, Err(BridgeError::PoolNotInitialized)));
}
```

---

## Summary

| Defect | Severity | Status |
|--------|----------|--------|
| Option B missing logging | P0 | MUST FIX |
| PoolNotInitialized no runtime test | P2 | SHOULD FIX |

**VERDICT: REJECTED** - Option B contract violation requires correction before approval.
