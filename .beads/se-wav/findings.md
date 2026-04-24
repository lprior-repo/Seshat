# ARCH-DRIFT: canvas_domain/src/interaction_reducer/tests/mod.rs

## STATUS: DRIFTED — 3 files exceed 300-line limit

---

## 1. Line Count Audit

### Source Files (non-test) — ALL PASS
| File | Lines | Status |
|------|-------|--------|
| `types.rs` | 90 | PASS |
| `geometry.rs` | 52 | PASS |
| `resize.rs` | 79 | PASS |
| `release.rs` | 67 | PASS |
| `commit.rs` | 259 | PASS |
| `commit_tests.rs` | 263 | PASS |
| `tests/mod.rs` | 4 | PASS |

### Test Files — 3 VIOLATIONS
| File | Lines | Limit | Excess |
|------|-------|-------|--------|
| `tests/subgraph_tests.rs` | **1200** | 300 | **+900 (4x)** |
| `tests/basic_tests.rs` | **725** | 300 | **+425 (2.4x)** |
| `tests/proptests.rs` | **686** | 300 | **+386 (2.3x)** |
| `tests/inp_mobile_touch_tests.rs` | 219 | 300 | PASS |

---

## 2. Split Strategy for Violating Files

### 2a. `tests/subgraph_tests.rs` (1200 lines → 7 files)

Current sections by tag:
- SUB-001: z_index priority (lines 91–195) → `tests/subgraph_tests/z_index.rs`
- SUB-002: rubber-band across container boundary (lines 197–310) → `tests/subgraph_tests/rubberband.rs`
- SUB-003: collapse/expand container (lines 312–467) → `tests/subgraph_tests/collapse.rs`
- SUB-004: locked container with unlocked children (lines 469–600) → `tests/subgraph_tests/lock.rs`
- SUB-005: parent-child relationship preservation (lines 602–770) → `tests/subgraph_tests/parent_child.rs`
- SUB-006–010: drag/nesting/descendants (lines 772–975) → `tests/subgraph_tests/drag.rs`
- MUL-003: reparent across container boundary (lines 1049–1200) → `tests/subgraph_tests/reparent.rs`
- Shared helpers `make_subgraph_node` + `make_child_node` (lines 18–89) → `tests/subgraph_tests/mod.rs`
- `tests/subgraph_tests.rs` becomes `tests/subgraph_tests/mod.rs` (re-exports helpers + sub-modules)

### 2b. `tests/basic_tests.rs` (725 lines → 3 files)

- History atomicity regression tests (lines 155–321) → `tests/basic_tests/history.rs`
- MUL-001–005: resize with rotated/text/line/arrow/inversion (lines 323–725) → `tests/basic_tests/resize_mixed.rs`
- Core finalize + resize-target tests + helpers (lines 1–153) → `tests/basic_tests/mod.rs`
- **DDD finding**: `unlocked_node()` helper is copy-pasted 5 times inside individual test functions (lines 110, 337, 401, 463, 544). Should be extracted to module-level shared helper in `mod.rs`.

### 2c. `tests/proptests.rs` (686 lines → 4 files)

- `within()` property tests (lines 54–123, 321–415, 541–582, 627–639) → `tests/proptests/within.rs`
- `finalize_motion_release` property tests (lines 143–215, 297–319, 505–539, 671–686) → `tests/proptests/finalize.rs`
- Interaction mode property tests (lines 217–262, 441–503, 584–625) → `tests/proptests/mode.rs`
- Resize + subgraph property tests (lines 125–141, 275–295, 641–669) → `tests/proptests/resize.rs`
- Shared `node()` helper (lines 18–38) → `tests/proptests/mod.rs`

---

## 3. DDD / Scott Wlaschin Analysis

### 3a. Primitive Obsession — LOW RISK
Test helpers use `&str` for IDs and `f64` for coordinates. Acceptable for test data builders since the production types (`NodeId`, `OrderedFloat`) are properly wrapped. No action needed.

### 3b. Parse, Don't Validate — COMPLIANT
All test helpers construct valid domain objects directly. No validation-then-use patterns detected.

### 3c. Shared Helper Duplication — MEDIUM RISK
- `node()` helper is duplicated verbatim between `basic_tests.rs:15` and `proptests.rs:18` (identical signatures, only `LockState` differs: Locked vs Unlocked).
- `unlocked_node()` is defined 5 times inside `basic_tests.rs` test function bodies (lines 110, 337, 401, 463, 544) — exact same function body each time.
- **Recommendation**: Extract to shared `tests/helpers.rs` module with `fn node()` and `fn unlocked_node()` variants.

### 3d. Clippy Suppression Concerns — HIGH RISK
- `subgraph_tests.rs:1-9`: Suppresses `unwrap_used`, `panic`, `unused_variables`, `unused_imports`
- `basic_tests.rs:1-8` and `proptests.rs:1-8`: Suppress `clippy::all`, `clippy::pedantic`, `clippy::nursery`, `unwrap_used`, `expect_used`, `panic`
- **Blanket suppression of `clippy::all` + `pedantic` + `nursery` in tests defeats the purpose of lint discipline.** Even in tests, `panic` and `unwrap_used` should be targeted per-use, not blanket-suppressed at module level.
- **Recommendation**: Remove blanket `clippy::all/pedantic/nursery` suppressions. Keep targeted `#[allow(clippy::unwrap_used)]` on individual test functions that genuinely need it.

---

## 4. Structural Cohesion Analysis

### Source module structure — GOOD
The `interaction_reducer` module follows clean separation:
- `types.rs`: Domain types (InteractionMode, ResizeHandle, error types)
- `geometry.rs`: Pure geometry calculations (resize_target_ids, re-exports from canvas_math)
- `resize.rs`: Resize interaction lifecycle (start_resize_interaction)
- `release.rs`: Gesture finalization (finalize_motion_release)
- `commit.rs`: Inline edit commit with history integration

All source files use proper guards:
```rust
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]
```

### Test module structure — NEEDS WORK
The `tests/` directory has 4 test files + 1 mod.rs. Three exceed the 300-line limit. The split strategies above would result in:
- `tests/basic_tests/` (3 files)
- `tests/subgraph_tests/` (8 files)
- `tests/proptests/` (5 files)
- `tests/inp_mobile_touch_tests.rs` (unchanged, 219 lines)
- `tests/mod.rs` (updated to reference new sub-directories)
- Optional: `tests/helpers.rs` (shared test utilities)

---

## 5. Action Items (for follow-up beads)

1. **CRITICAL**: Split `tests/subgraph_tests.rs` (1200 lines) into 7 domain-specific files + mod.rs
2. **CRITICAL**: Split `tests/basic_tests.rs` (725 lines) into 3 files + extract duplicated `unlocked_node()` helper
3. **CRITICAL**: Split `tests/proptests.rs` (686 lines) into 4 files
4. **HIGH**: Extract shared test helpers (`node()`, `unlocked_node()`) to `tests/helpers.rs`
5. **HIGH**: Remove blanket clippy suppressions in test modules; use targeted `#[allow]` annotations instead
6. **MEDIUM**: Deduplicate `node()` helper between `basic_tests.rs` and `proptests.rs` into shared module

---

## 6. Summary

| Check | Result |
|-------|--------|
| Source file sizes | ALL PASS (max 259 lines) |
| Test file sizes | **3 VIOLATIONS** (1200, 725, 686 lines) |
| DDD: Primitive Obsession | LOW RISK |
| DDD: Parse don't validate | COMPLIANT |
| DDD: Helper duplication | MEDIUM RISK |
| Clippy discipline | **HIGH RISK** (blanket suppressions) |
| Structural cohesion (source) | GOOD |
| Structural cohesion (tests) | **NEEDS WORK** |
| `#![forbid(unsafe_code)]` | COMPLIANT (all source files) |
| `#![deny(unwrap/expect/panic)]` | COMPLIANT (non-test cfg, all source files) |
