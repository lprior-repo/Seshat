# QA-MANUAL: canvas_domain/src/selection_geometry/kani_proofs.rs

## File Under Test

`canvas_domain/src/selection_geometry/kani_proofs.rs` (106 lines)

Two Kani proofs:
1. `proof_selection_bounds_envelop_all_selected_unlocked_nodes` (line 48)
2. `proof_selected_node_ids_filters_locked_nodes` (line 88)

Supporting code:
- `core.rs` — `selected_node_ids()` and `selection_bounds()` implementations
- `test_utils.rs` — `make_node()` / `make_node_with_lock()` helpers

## Findings

### F1: CRITICAL — `selection_bounds` returns `Some((+inf, +inf, NaN, NaN))` when ALL selected nodes are locked (core.rs:55)

**Severity: HIGH — runtime produces mathematically invalid output**

When every node in `selected_items` is locked, the `filter_map` in `selection_bounds()` (line 30-37) filters out ALL nodes. The `fold` accumulator starts at `(inf, inf, -inf, -inf)` and never updates. Line 55 then computes:

```rust
Some((inf, inf, -inf - inf, -inf - inf))
= Some((inf, inf, NaN, NaN))
```

This returns `Some` with NaN values — a poison pill that will silently corrupt any downstream geometry calculation.

**The kani proof DOES catch this** — `proof_selection_bounds_envelop_all_selected_unlocked_nodes` line 84 asserts `locked_a && locked_b` in the else branch. However, the **core function itself has no guard**. The bug is in `core.rs`, not the proof.

**Fix**: After the fold, check if `min_x == f64::INFINITY` (the initial sentinel), and return `None` if so — meaning no unlocked nodes were found.

### F2: MEDIUM — `selection_bounds` returns `Some((inf, inf, NaN, NaN))` when selected_items contains IDs not in the document (core.rs:22-56)

If `selected_items` contains a string ID that doesn't correspond to any node in `doc.document.nodes`, the `filter_map` filters it out. Same accumulator-poisoning scenario as F1.

This is a realistic scenario — stale selection state after node deletion.

**Fix**: Same as F1 — check sentinel after fold.

### F3: MEDIUM — `selection_bounds` propagates NaN from node coordinates into output (core.rs:46-50)

The fold uses `min()` and `max()` which propagate NaN:
- `f64::INFINITY.min(f64::NAN)` = `NaN`
- `f64::NEG_INFINITY.max(f64::NAN)` = `NaN`

If any node has `x.0` or `y.0` set to NaN, the entire selection bounds becomes NaN.

**Note**: The kani proof `make_any_node` (line 12-13) assumes `is_finite()`, which excludes NaN. This means the kani proof does NOT cover the NaN propagation path.

**Risk**: `OrderedFloat` wraps raw f64 — it does NOT guarantee finiteness. NaN can enter through deserialization of malformed JSON.

### F4: LOW — `selection_bounds` returns negative width/height for overflow (core.rs:55)

If `n.x.0 + n.width.0` overflows f64 (exceeds ~1.8e308), the `max_x` fold accumulator becomes `+inf`. Then `max_x - min_x` = `+inf`, which is at least not NaN but is still a degenerate value.

The kani proof constrains coordinates to `[-1e10, 1e10]` and widths/heights to `[0, 1e10)`, which prevents overflow. However, the production code has no such bounds.

### F5: INFO — `selected_node_ids` silently drops phantom selections (core.rs:5-19)

When an ID in `selected_items` doesn't exist in `doc.document.nodes`, it's silently filtered out. This is correct behavior but could mask bugs in selection state management. Not a bug in this module, but worth noting.

### F6: INFO — Kani proofs are well-structured

The two kani proofs correctly verify:
- Bounds envelop all unlocked selected nodes (proof at line 48)
- Locked nodes are excluded from `selected_node_ids` (proof at line 88)
- All-locked case returns None from `selection_bounds` (line 83-85)

The `make_any_node` helper properly constrains:
- Finiteness via `is_finite()`
- Non-negative width/height
- Bounded coordinates `[-1e10, 1e10]`

### F7: INFO — Test files in sibling modules use `#[cfg(kani)]` gate but are in non-kani files

`selection_bounds_tests.rs`, `selection_interaction_tests.rs`, `locked_nodes_tests.rs` all mark their tests with `#[cfg(kani)]` and `#[kani::proof]`, but the `mod.rs` only gates `kani_proofs.rs` with `#[cfg(kani)]`. The other test modules are gated with `#[cfg(test)]` but contain only `#[cfg(kani)]` functions. This means:
- Under `cargo test` — modules compile but contain no test functions (harmless dead code)
- Under `cargo kani` — all proofs across all files are discovered

This is correct but slightly confusing organization.

## Summary of Actionable Items

| ID | Severity | File | Line | Issue |
|----|----------|------|------|-------|
| F1 | HIGH | core.rs | 55 | Returns Some with NaN when all selected nodes are locked |
| F2 | MEDIUM | core.rs | 55 | Returns Some with NaN for phantom/orphan selection IDs |
| F3 | MEDIUM | core.rs | 46-50 | NaN in node coords propagates to output |
| F4 | LOW | core.rs | 55 | Overflow can produce infinite width/height |

## Recommended Fix for F1+F2 (core.rs)

Add a guard after the fold:

```rust
if min_x == f64::INFINITY {
    return None;
}
```

This handles both "all locked" and "no matching nodes" cases correctly.
