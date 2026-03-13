# Defects Report: GEO-020 Hit Test Margin Zoom Behavior

## Issue Summary

The implementation of SCREEN_HIT_MARGIN zoom behavior in `diagram_tool/src/ui/canvas.rs` fails to satisfy the contract specification in `seshat-9t4/.beads/seshat-9t4/contract.md`.

## Phase 1: Contract & Bead Parity - FAIL

### Critical Contract Violations

1. **Missing Error Type (Contract Lines 31-34)**
   - Contract requires: `HitTestError` enum with variants `InvalidZoom`, `InvalidMargin`, `InvalidPoint`
   - Implementation: No such type exists; returns `Option<NodeId>` instead of `Result<T, HitTestError>`

2. **Missing Validation Functions (Contract Lines 52, 68-73)**
   - Contract requires: `fn screen_to_world_margin(screen_margin: f64, zoom: f64) -> Result<f64, HitTestError>`
   - Contract requires: `fn hit_test_with_margin(...) -> Result<bool, HitTestError>`
   - Implementation: These functions do NOT exist; logic is inlined in `find_node_at` with no error handling

3. **No Precondition Enforcement (Contract Lines 16-19)**
   - P1: `zoom` must be within [MIN_ZOOM, MAX_ZOOM] = [0.1, 4.0]
   - P2: `SCREEN_HIT_MARGIN` must be > 0
   - P3: Point coordinates must be finite
   - Implementation: NONE of these preconditions are validated

### Mathematical Correctness (Partial Pass)

The inline calculation at canvas.rs:212:
```rust
let hit_margin_world = SCREEN_HIT_MARGIN / zoom;
```
- At zoom 0.1: 50.0 ✓
- At zoom 1.0: 5.0 ✓
- At zoom 4.0: 1.25 ✓

Math is correct, but contract requires explicit validation.

## Phase 2: Farley Engineering Rigor - PARTIAL PASS

- Function length: 20 lines (under 25) ✓
- Parameter count: 3 (under 5) ✓
- **VIOLATION**: `safe_zoom()` exists (canvas.rs:319-321) but is NOT used in `find_node_at`

## Phase 3: NASA-Level Functional Rust - FAIL

- No sum types for HitTestError (contract requires it)
- No "Parse, Don't Validate" at boundary
- Raw `f64` for zoom instead of newtype

## Phase 4: Ruthless Simplicity - FAIL

- Contract explicitly requires `Result<T, HitTestError>` pattern
- Implementation uses `Option` instead - violates contract

## Phase 5: The Bitter Truth

The implementation represents the "happy path only" approach that ignores contract-specified error handling. This looks like a simplified draft that was never completed to meet the full specification.

## Required Fixes

1. Create `HitTestError` enum with variants: `InvalidZoom`, `InvalidMargin`, `InvalidPoint`, `PostconditionViolation`

2. Create `screen_to_world_margin(screen_margin: f64, zoom: f64) -> Result<f64, HitTestError>` with proper validation

3. Create `hit_test_with_margin(...) -> Result<bool, HitTestError>` using the above function

4. Add zoom range validation: `zoom.is_finite() && (MIN_ZOOM..=MAX_ZOOM).contains(&zoom)`

5. Add margin positivity validation: `screen_margin > 0.0`

6. Add point finiteness validation: `point.x.is_finite() && point.y.is_finite()`

7. Replace inline calculation in `find_node_at` with calls to the new validated functions
