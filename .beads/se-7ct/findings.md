# RED QUEEN VERDICT: selection_interaction_tests.rs

**Bead**: se-7ct
**Target**: `canvas_domain/src/selection_geometry/selection_interaction_tests.rs`
**Date**: 2026-04-24
**Agent**: nuka (seshat polecat)

## CROWN FORFEIT

The selection_geometry module has **zero effective test coverage**. The crown is forfeit.

---

## METHODOLOGY

1. Static analysis of all test files in `selection_geometry/`
2. `cargo test -p canvas_domain -- --list` to enumerate active tests
3. `cargo mutants -p canvas_domain --file canvas_domain/src/selection_geometry/core.rs` for mutation testing
4. `cargo clippy` with strict lint gates
5. Manual review of assertion quality

---

## FINDINGS

### CRITICAL-1: 94.8% Mutation Miss Rate (91/96 mutants MISSED)

`cargo-mutants` generated 96 mutants against `core.rs`. Only **2 were caught**. 91 mutants survived, meaning the implementation can be arbitrarily broken without any test failing.

Key uncaught mutations:
- `selection_bounds` can return ANY `Some((f64, f64, f64, f64))` tuple — all 80+ replacement mutants MISSED
- `selection_bounds` can return `None` — MISSED
- `delete ! in selected_node_ids` (line 15) — locked node filter can be inverted — MISSED
- `replace + with -` in bounds arithmetic (lines 49-50) — MISSED
- `replace - with +` in width/height calculation (line 55) — MISSED

**Root cause**: No `#[test]` functions exist. All tests are `#[cfg(kani)]` only.

### CRITICAL-2: All Tests Are Dead Code Under Normal Testing

Every test function across all 3 test files uses `#[cfg(kani)]`:

| File | Tests | All `#[cfg(kani)]` |
|------|-------|---------------------|
| `selection_interaction_tests.rs` | 3 | YES |
| `selection_bounds_tests.rs` | 3 | YES |
| `locked_nodes_tests.rs` | 3 | YES |
| **Total** | **9** | **ALL** |

`cargo test -p canvas_domain` passes 121 tests — **ZERO** are from `selection_geometry`.

`kani` is installed (0.67.0) but these proofs are never run in CI.

### CRITICAL-3: Tautological Assertion (Line 159)

```rust
assert!(
    !node.label.is_empty() || node.label.is_empty(),
    "Label is accessible for editing"
);
```

This assertion is **always true** — it's equivalent to `assert!(true)`. It tests nothing. The `test_utils::make_node` creates nodes with `label: String::from("n")` so `label.is_empty()` is always false, making the first branch always true regardless.

### MAJOR-1: Locked Node Exclusion Untested by Regular Tests

The GEO-024 feature (filtering locked nodes from selection) is tested ONLY by Kani proofs in `locked_nodes_tests.rs`. The mutation `delete ! in selected_node_ids` (inverting the lock filter) is MISSED — meaning if the filter were inverted to include only locked nodes, no regular test would catch it.

### MAJOR-2: Bounds Calculation Arithmetic Unprotected

All arithmetic operators in `selection_bounds` can be swapped (`+` → `-`, `+` → `*`, `-` → `/`) without test failure. The entire bounding box calculation is unverified:

- `max_x.max(n.x.0 + n.width.0)` → `+` can become `-` or `*` — MISSED
- `max_y.max(n.y.0 + n.height.0)` → `+` can become `-` or `*` — MISSED
- `(max_x - min_x, max_y - min_y)` → `-` can become `+` or `/` — MISSED

### MAJOR-3: Empty Selection Returns None — Unverified

`selection_bounds` returns `None` when no items are selected. The mutation replacing the entire function with `None` is MISSED. Even always returning None passes all tests.

### MINOR-1: SEL-003 Tests History, Not selection_geometry

The test `given_selection_history_when_undo_redo_then_selection_restored` exercises `History::undo()` and `History::redo()`. It doesn't call `selected_node_ids()` or `selection_bounds()`. It's testing the History crate, not the selection_geometry module.

### MINOR-2: Only 2 Mutants Caught

The 2 caught mutants were both on `selected_node_ids`:
- `replace selected_node_ids -> SmallVec::from_iter([[Default::default(); 4]])` — caught
- One other default-value replacement — caught

This means `selected_node_ids` has marginal coverage from some other test module (not from selection_geometry itself).

---

## DIMENSION SCORES

| Dimension | Tests Run | Survivors | Fitness | Status |
|-----------|-----------|-----------|---------|--------|
| mutation-coverage | 96 | 91 | 0.948 | HEMORRHAGING |
| test-gating | 9 | 9 | 1.000 | HEMORRHAGING |
| assertion-quality | 1 | 1 | 1.000 | HEMORRHAGING |
| lock-filter | 1 | 1 | 1.000 | HEMORRHAGING |
| bounds-arithmetic | 6 | 6 | 1.000 | HEMORRHAGING |

---

## RECOMMENDATIONS

1. **Convert ALL `#[cfg(kani)]` tests to dual-gate `#[cfg(any(test, kani))]`** — or add parallel `#[test]` versions
2. **Fix tautological assertion at line 159** — test something meaningful about label accessibility
3. **Add `#[test]` coverage for `selection_bounds`** — verify exact bounding box values for known node positions
4. **Add `#[test]` for locked node exclusion** — verify locked nodes are filtered from both `selected_node_ids` and `selection_bounds`
5. **Add `#[test]` for empty selection** — verify `selection_bounds` returns `None`
6. **Add edge case tests** — zero-width/height nodes, overlapping nodes, single-pixel nodes
7. **Target mutation kill rate ≥90%** — currently at 2.1% (2/96)

---

## FILES ANALYZED

| File | Lines | Role |
|------|-------|------|
| `selection_geometry/core.rs` | 56 | Implementation (2 pub fns) |
| `selection_geometry/selection_interaction_tests.rs` | 162 | 3 Kani proofs (SEL-002, SEL-003, SEL-005) |
| `selection_geometry/selection_bounds_tests.rs` | 106 | 3 Kani proofs (SEL-001, SEL-004, bounds) |
| `selection_geometry/locked_nodes_tests.rs` | 103 | 3 Kani proofs (GEO-024) |
| `selection_geometry/test_utils.rs` | 32 | Helper `make_node`, `make_node_with_lock` |
| `selection_geometry/mod.rs` | 24 | Module definition |

## BUILD STATUS

- `cargo test -p canvas_domain --no-run`: COMPILES
- `cargo test -p canvas_domain`: 121 passed (none from selection_geometry)
- `cargo clippy -p canvas_domain --tests`: No issues
- `cargo mutants`: 91/96 missed (94.8% miss rate)
