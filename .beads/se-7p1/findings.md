# BLACKHAT REVIEW: canvas_domain/src/drag_math/mod.rs

**Target**: `canvas_domain/src/drag_math/mod.rs` + `subgraphs.rs` + `tests/`
**Reviewer**: pipboy (adversarial black-hat)
**Date**: 2026-04-24
**Verdict**: REJECT

---

## PHASE 1: Contract & Bead Parity

The bead requests "adversarial security testing and attack surface analysis" of `drag_math/mod.rs`. The module is a 4-line re-export file that delegates to `subgraphs.rs`. The actual attack surface lives in `subgraphs.rs::calculate_resize_target_ids`.

**FINDING**: No contract-spec.md or martin-fowler-tests.md found to verify parity against. The Kani proofs in `tests/subgraph_tests.rs` partially serve as formal contract, but they are duplicated (identical proofs exist in both `tests/subgraph_tests.rs:1281-1416` and `tests/kani_proofs.rs`). This duplication is a maintenance hazard — one will drift.

---

## PHASE 2: Farley Engineering Rigor

### 2.1: `calculate_resize_target_ids` (subgraphs.rs:7-39) — 33 lines, PASS

Function is within the 25-line soft limit (33 is close). Two parameters, both references. Acceptable.

### 2.2: `drag_original_positions` (stubs.rs:85-124) — 40 lines, FAIL

**stubs.rs:95-115**: 20+ lines of `successors` + `fold` chain is a single nested closure that is hard to reason about. The fixed-point iteration via `std::iter::successors` for child expansion is clever but fragile:

- **Bug**: The `fold` on line 100 iterates ALL nodes in the document on EACH iteration, making this O(n*k) where k is the depth of nesting. For deeply nested hierarchies, this is pathological.
- **Bug**: The `unwrap_or_else(HashSet::new)` on line 115 is dead code — `successors` with a `Some` seed will always yield at least the initial set. The `unwrap_or_else` masks a logic error.

### 2.3: `mutate_doc_with_history` (stubs.rs:68-82) — takes mutable references to two Signals

Pure function signature violation — this function mutates two signal states (doc + history). Not pure, not easily testable.

---

## PHASE 3: NASA-Level Functional Rust (The Big 6)

### 3.1: CRITICAL — Unwrapped primitive tuples as geometry (subgraphs.rs:9)

```rust
node_geometry: &HashMap<NodeId, (f64, f64, f64, f64, bool)>
```

**SEVERE**: A 5-tuple `(f64, f64, f64, f64, bool)` as geometry representation is a type safety disaster:
- What is field 0? x? width? You can't tell from the type.
- The `bool` at position 4 — what does `true` mean? `is_subgraph`? `is_visible`? `is_locked`?
- Two different callsites construct this tuple differently — `subgraph_tests.rs:693-704` builds it from `node.kind == NodeKind::Subgraph`, but nothing prevents constructing it with wrong ordering.

**Attack vector**: A caller could swap x/y or w/h and get incorrect containment checks. The `within()` function from `canvas_math` receives these as positional arguments — no type-level protection.

**Remediation**: Introduce a named struct:
```rust
struct NodeGeometry { x: f64, y: f64, width: f64, height: f64, is_subgraph: bool }
```

### 3.2: Parse Don't Validate — Missing

`calculate_resize_target_ids` accepts `selected_ids: &[NodeId]` but performs NO validation that these IDs exist in `node_geometry`. Missing IDs are silently ignored (line 16: `if let Some(...)`). This is not "parse don't validate" — this is "don't even check."

### 3.3: Boolean parameter anti-pattern (stubs.rs:86)

`selected_items: &HashSet<String>` — strings as node IDs? `NodeId` exists. This forces string→NodeId conversion inside the function (line 91). The conversion should happen at the boundary, not inside.

---

## PHASE 4: Ruthless Simplicity & DDD

### 4.1: `let mut selected_set` (subgraphs.rs:11)

**MUTABLE STATE**: `selected_set` is mutated across two loops (lines 14-21 and 27-36). This is a functional transformation that should be expressed as a fold or pipe, not imperative mutation.

### 4.2: Panic vectors in tests

The test file (`subgraph_tests.rs`) has `#![allow(clippy::unwrap_used, clippy::panic, ...)]` at line 1. This blanket suppression hides ALL panic vectors in tests. While test code has more leeway, this is a red flag — it means no one reviewed whether these unwraps are safe.

Specific concerns:
- Line 351: `serde_json::to_string(&doc).expect("serialization should succeed")` — if serialization fails, the test panics without useful diagnostics.
- Line 383: `doc.document.nodes.get_mut(&node1_id)` — `get_mut` returns `Option`, the `if let Some` is fine, but the subsequent `.unwrap()` on line 1193 shows inconsistency.

### 4.3: `interaction_reducer.rs` blanket allows

```rust
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
```

The interaction_reducer module (which drag_math depends on) disables ALL panic protection. This is the module that handles user interactions. Any panic here crashes the UI.

---

## PHASE 5: The Bitter Truth

### 5.1: Duplicated Kani proofs

`tests/subgraph_tests.rs:1281-1416` is an exact duplicate of `tests/kani_proofs.rs`. This violates DRY and will inevitably drift.

### 5.2: Tests that test nothing (subgraph_tests.rs)

Multiple tests are `#[cfg(kani)]` — they ONLY run under the Kani verifier, never under `cargo test`. The only `#[test]` functions are `given_multi_selection_dragged_across_container_boundary_when_ends_inside_then_reparents` (line 1151) and `given_multi_selection_dragged_out_of_container_when_ends_outside_then_reparents_to_root` (line 1219).

**CRITICAL**: Both `#[test]` functions contain comments like:
```
// Note: The actual reparent logic should be triggered at drag-end
// This test documents the expected behavior
// Currently this is a PASS if the position check works
// The reparent implementation is what MUL-003 requires
```

These tests **DO NOT TEST THE ACTUAL BEHAVIOR**. They set up state, mutate positions, and then just assert positions are where they put them. They test tautologies. The real reparent logic is untested.

### 5.3: YAGNI — `DispatchError` enum with one variant

`stubs.rs:8-10`: `DispatchError` has a single variant `Failed`. This is premature abstraction — just use `()` or `Result<(), ()>`. Adding a second variant later is trivial; having one now is noise.

### 5.4: Weird indentation in test file

Lines 4-8 and throughout the test file have inconsistent indentation (extra leading spaces). This is cosmetic but reflects copy-paste without cleanup.

---

## ATTACK SURFACE ANALYSIS

### Surface 1: Geometry Tuple Injection
**Severity**: HIGH
**Vector**: `calculate_resize_target_ids` receives `HashMap<NodeId, (f64, f64, f64, f64, bool)>`. Any caller constructing this map can:
- Swap x/y ordering
- Provide negative widths/heights (the `within()` function likely doesn't guard against this)
- Set `is_subgraph=true` on non-subgraph nodes, causing unexpected node inclusion

### Surface 2: Silent Data Loss via Missing IDs
**Severity**: MEDIUM
**Vector**: If `selected_ids` contains IDs not present in `node_geometry`, they are silently dropped from the output. The function returns `selected_ids.to_vec()` as fallback when no subgraphs exist (line 24), but this means the function has two different semantics: "return selected" vs "return selected + contained".

### Surface 3: Non-deterministic Output Ordering
**Severity**: LOW
**Vector**: `selected_set.into_iter().collect()` (line 38) returns `Vec<NodeId>` in arbitrary HashSet order. Callers depending on ordering will get non-deterministic behavior. This could cause flickering in UI if the order is used for z-index stacking.

### Surface 4: O(n*k) Performance in `drag_original_positions`
**Severity**: MEDIUM
**Vector**: The `successors`/`fold` pattern in `stubs.rs:95-115` re-scans the entire document on each iteration. For a document with 10,000 nodes and 10 levels of nesting, this is 100,000 iterations. A maliciously crafted document with deep nesting could cause UI freezes.

### Surface 5: String-typed Node IDs in `drag_original_positions`
**Severity**: MEDIUM  
**Vector**: `selected_items: &HashSet<String>` forces string→NodeId conversion inside the function (stubs.rs:91). A typo in the string silently produces a NodeId that doesn't match any node, silently dropping it from drag calculations.

---

## SUMMARY

| Phase | Verdict | Critical Issues |
|-------|---------|-----------------|
| 1. Contract Parity | WARN | No contract-spec.md; duplicated Kani proofs |
| 2. Farley Rigor | FAIL | 40-line function; O(n*k) performance; mutable Signal mutation |
| 3. Functional Rust | FAIL | 5-tuple primitives; missing validation; String IDs |
| 4. Simplicity/DDD | FAIL | Mutable state; blanket panic allows; empty error enum |
| 5. Bitter Truth | FAIL | Tautological tests; 95% of tests are kani-only; DRY violation |

**OVERALL VERDICT: REJECT**

The module has 1 HIGH severity attack surface (geometry tuple injection), 3 MEDIUM severity issues, and the test suite provides near-zero confidence — 2 of 2 `#[test]` functions are tautologies that explicitly document they don't test the actual behavior.
