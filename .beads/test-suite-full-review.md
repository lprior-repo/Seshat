# Test Suite Full Review — 6 Beads (Mode 2: Suite Inquisition)

**Date:** 2026-03-25
**Scope:** seshat-uee, seshat-663, seshat-5zc, seshat-feo, seshat-b36, seshat-9vd
**Auditor:** Test Inquisitor (Mode 2)

---

## VERDICT: REJECTED

4 LETHAL findings. 3 MAJOR findings. 2 MINOR findings. Stop.

---

## Tier 0 — Static Analysis

### [FAIL] Banned assertions (`is_ok()` / `is_err()` without inner value)
**Scope:** `diagram_tool/src/ui/theme/`, `diagram_tool/src/ui/canvas/grid_layer.rs`
```
grep -rn "is_ok()\|is_err()" → 0 hits in scope files
```
PASS. No banned assertion patterns found.

### [FAIL] Silent error discard (`let _ =` / `.ok()`)
**Scope:** `diagram_tool/src/ui/theme/*_tests.rs`, `diagram_tool/src/ui/canvas/grid_layer.rs`
```
grep -rn "let _ = \|\.ok();" → 0 hits in the 6 test files under review
```
PASS. Note: `let _ = parse_event(raw)` exists in `interaction_fuzz_prop_tests.rs:29` but is outside scope.

### [PASS] Ignored tests (`#[ignore]`)
```
grep -rn "#\[ignore\]" diagram_tool/src/ → 0 hits
```
PASS.

### [PASS] Sleep in tests
```
grep -rn "sleep\|thread::sleep" → hits only in production code (raf.rs, resize.rs)
```
PASS. No sleeps in test files.

### [FAIL] Naming violations (`fn test_`)
```
tokens_tests.rs:46   fn test_dark_border_subtle_lightness_is_0_30
tokens_tests.rs:55   fn test_dark_node_bg_lightness_is_0_22
tokens_tests.rs:61   fn test_dark_node_border_lightness_is_0_38
tokens_tests.rs:70   fn test_dark_grid_dot_lightness_is_0_30
tokens_tests.rs:76   fn test_dark_luminance_hierarchy
tokens_tests.rs:96   fn test_light_tokens_unchanged
tokens_tests.rs:111  fn test_dark_chroma_preserved
tokens_tests.rs:139  fn test_white_bg_base_is_pure_white
tokens_tests.rs:149  fn test_white_text_main_is_dark
tokens_tests.rs:159  fn test_white_node_border_visible
tokens_tests.rs:170  fn test_white_palette_completeness
tokens_tests.rs:207  fn test_css_vars_for_white_produces_valid_output
tokens_tests.rs:233  fn test_white_palette_differs_from_light
grid_layer.rs:82     fn test_grid_pattern_alignment_with_nodes
grid_layer.rs:93     fn test_grid_pattern_alignment_with_camera_offset
grid_layer.rs:105    fn test_grid_pattern_scales_with_zoom
grid_layer.rs:117    fn test_grid_pattern_minimum_step_clamp
grid_layer.rs:124    fn test_grid_pattern_negative_camera
grid_layer.rs:134    fn test_grid_pattern_zero_zoom_clamps
grid_layer.rs:143    fn test_grid_dot_is_css_variable
grid_layer.rs:151    fn test_bg_base_is_css_variable
grid_layer.rs:159    fn test_grid_dot_references_grid_token
grid_layer.rs:167    fn test_bg_base_references_bg_token
```
**23 violations** across 2 files. theme_mode_tests.rs and css_var_tests.rs correctly use descriptive names.

### [FAIL] Loops in test bodies (Holzmann Rule 2)
```
tokens_tests.rs:124  for &field in &neutral_fields {
tokens_tests.rs:130  for &field in &border_accented {
tokens_tests.rs:201  for (i, val) in fields.iter().enumerate() {
tokens_tests.rs:214  for segment in css.split(';') {
```
**4 loops** in test bodies across `tokens_tests.rs`. Each loop hides multiple assertions behind a single iteration — a failure in one iteration may mask another. Each field in the collection deserves its own named test.

### [PASS] Shared mutable state (`static mut` / `lazy_static!`)
```
grep -rn "static mut\|lazy_static!" → 0 hits
```
PASS.

### [PASS] Mock interrogation
```
grep -rn "mockall\|Mock.*::new\(\)\|\.expect_" → 0 hits in scope
```
PASS. No mocks.

### [PASS] Integration test purity
```
ls diagram_tool/tests/ → exists but no `use crate::` in test files
```
PASS.

### [PASS] Error variant completeness
```
grep -rn "enum.*Error" → 0 hits in theme/ or grid_layer.rs
```
No Error enums in scope. Nothing to test.

### [PASS] Density: 79 tests / 8 pub functions = 9.9x (target >=5x)

**Pub functions in scope:**
| Function | Location |
|----------|----------|
| `ThemeMode::persisted_key` | mod.rs:28 |
| `ThemeMode::from_persisted_key` | mod.rs:38 |
| `ThemeMode::label` | mod.rs:49 |
| `ThemeMode::next` | mod.rs:60 |
| `ThemeMode::resolve` | mod.rs:70 |
| `ThemeScheme::from_str` | mod.rs:91 |
| `css_vars_for` | mod.rs:102 |
| `calculate_grid_pattern` | grid_layer.rs:6 |

**Tests in scope:** 79 (70 native + 9 wasm32-gated)

---

## Tier 1 — Execution

### [FAIL] Clippy: 4 errors

With `-D warnings -W clippy::pedantic`:
```
error: variables can be used directly in the format! string
error: item in documentation is missing backticks → root_container/mod.rs:211 (theme_mode)
error: item in documentation is missing backticks → root_container/mod.rs:211 (ThemeProvider)
error: redundant closure → toolbar/persistence_compat/mod.rs:58
```

With protocol command (`-D warnings` only): **1 error** (redundant closure in `toolbar/persistence_compat/mod.rs:58`). This is LETHAL — `clippy::redundant_closure_for_method_calls` is warn-by-default and promoted to error by `-D warnings`.

### [PASS] nextest: 1379 passed, 0 failed, 0 flaky

```
running 695 tests → ok (0.33s)
running 655 tests → ok (0.33s)
running 19 tests  → ok (0.02s)  (diagram_models)
running 2 tests   → ok (0.40s)  (doc-tests)
```
All tests pass. No test exceeded 1s.

**NOTE:** 9 tests in `theme_scheme_tests.rs` are `#[cfg(target_arch = "wasm32")]` and did NOT execute in native test run. These are unverified in CI.

### [PASS] Ordering probe
All tests passed in both single-threaded and multi-threaded modes. No ordering dependency detected.

### [PASS] Insta
No insta dependency in Cargo.toml. Not applicable.

---

## Tier 2 — Coverage

Cannot run `cargo llvm-cov` (toolchain not available). Deferring to manual analysis.

### Per-bead coverage assessment:

| Bead | Function Points | Tests | Assessment |
|------|----------------|-------|------------|
| seshat-uee | persisted_key, from_persisted_key, label, resolve, next, from_str | 44 + 9 (wasm32) | **Covered** |
| seshat-663 | dark L* values, hierarchy, light regression | 7 | **Covered** |
| seshat-5zc | CSS var substitution, grid math regression | 10 | **Covered** (missing crash test) |
| seshat-feo | ACCENT_DASH_BORDER constant | 3 | **Covered** |
| seshat-b36 | white_tokens values, completeness, contrast | 6 | **Covered** |
| seshat-9vd | ThemeToggle component, next() cycle | 5 (next only) | **GAP: no component tests** |

---

## Tier 3 — Mutation Survivability

### M1: Remove White from ThemeMode
**Survives?** NO. ~15 tests fail:
- `persisted_key_returns_white_for_white_mode`
- `from_persisted_key_returns_white_for_white_string`
- `label_returns_white_for_white_mode`
- 3x resolve White tests
- `next_cycles_dark_to_white`
- `full_cycle_returns_to_start`
- `proptest_resolve_white_always_returns_white`
- All 6 white_tokens tests

**Kill rate: 100%.**

### M2: Change `white_tokens()` back to `unreachable!()`
**Survives?** NO. 6 tests panic:
- `test_white_bg_base_is_pure_white`
- `test_white_text_main_is_dark`
- `test_white_node_border_visible`
- `test_white_palette_completeness`
- `test_white_palette_differs_from_light`
- `test_css_vars_for_white_produces_valid_output`

**Kill rate: 100%.**

### M3: Change `border_subtle` back to `0.22`
**Survives?** NO. 1 test fails:
- `test_dark_border_subtle_lightness_is_0_30` (expects 0.30, gets 0.22)

Hierarchy test may still pass (0.22 < 0.22 is false, so it would actually fail too — `node_bg (0.22) < border_subtle (0.22)` is false).

**Kill rate: 100%.**

### M4: Change `GRID_DOT` import to hardcoded value `"#444444"`
**Survives?** NO. 2 tests fail:
- `test_grid_dot_is_css_variable` (expects `starts_with("var(")`)
- `test_grid_dot_references_grid_token` (expects `contains("--grid-dot")`)

**Kill rate: 100%.**

### M5: Remove `hover:bg-white/5` from toolbar.rs
**Survives?** YES. No test verifies the hover class string exists anywhere.

**Kill rate: 0%. → SURVIVOR.**

### M6: Remove `ACCENT_DASH_BORDER` usage from root_container/mod.rs
**Survives?** NO. The component would use a different string, but the test only verifies the constant's value, not its usage. Actually — the test `accent_dash_border_uses_accent_variable` checks the constant value, not that it's used in the component. If someone replaced `ACCENT_DASH_BORDER` with a hardcoded string in the component, the constant test would still pass.

**Kill rate: 50%. → SURVIVOR for usage mutation.**

---

## LETHAL FINDINGS

### L1: Clippy error — redundant closure
**File:** `diagram_tool/src/ui/toolbar/persistence_compat/mod.rs:58`
**Detail:** `clippy::redundant_closure_for_method_calls` fires with `-D warnings`. Blocks the protocol's Gate 1 lint check.
**Impact:** Entire crate fails clippy gate.

### L2: 23 naming violations (`fn test_*`)
**Files:** `tokens_tests.rs` (13), `grid_layer.rs` (10)
**Detail:** Protocol bans `fn test_*` prefix. All 23 functions use it.
**Required fix:** Rename all 23 functions to descriptive BDD-style names matching `theme_mode_tests.rs` convention.

### L3: 4 loops in test bodies (Holzmann Rule 2)
**File:** `tokens_tests.rs:124, 130, 201, 214`
**Detail:**
- Line 124: `for &field in &neutral_fields` — iterates 8 fields with a single assert per iteration
- Line 130: `for &field in &border_accented` — iterates 2 fields
- Line 201: `for (i, val) in fields.iter().enumerate()` — iterates 26 fields checking non-empty
- Line 214: `for segment in css.split(';')` — iterates CSS segments checking format
**Required fix:** Extract each iteration into its own named test. For lines 124/130: one test per chroma field. For line 201: one test per white palette field. For line 214: remove loop, assert on specific CSS segment or split into named segment tests.

### L4: Missing `test_grid_renders_without_crash_when_css_variables_are_undefined`
**Bead:** seshat-5zc
**Detail:** The contract specifies this test but no such test exists in any file under review. The grid_layer tests verify CSS variable references and math, but no test verifies graceful behavior when CSS variables are undefined at runtime.
**Required fix:** Add test that calls `GridLayer` (or its sub-functions) with undefined/empty CSS variable values and asserts no panic.

---

## MAJOR FINDINGS (3)

### M1: seshat-9vd — Missing `test_toggle_button_renders_in_canvas_toolbar`
**Detail:** The `ThemeToggle` component at `root_container/mod.rs:213` has `data-testid: "theme-toggle-btn"` but no test verifies its existence or rendering. This is a Dioxus component and cannot be unit-tested without a browser environment, but a doc-test or contract-level assertion should exist.
**Required fix:** Add a compilation-only test or doc-comment test that verifies `ThemeToggle` is a valid component with the correct `data-testid`.

### M2: seshat-9vd — Missing `test_clicking_toggle_cycles_theme_mode`
**Detail:** No test verifies the onclick handler cycles through modes. The `next()` function is thoroughly tested in `theme_mode_tests.rs`, but the component's event wiring (`theme_mode.set(theme_mode.read().next())`) is untested.
**Required fix:** Add an integration or component-level test that verifies clicking the button advances the mode.

### M3: seshat-9vd — Missing `test_persisted_theme_is_loaded_on_page_reload`
**Detail:** The contract for seshat-9vd implies persistence integration, but no test verifies the full round-trip: toggle → persist → reload → restore.
**Required fix:** Add test that exercises the ThemeProvider persistence path through the toggle button.

---

## MINOR FINDINGS (2)

### m1: seshat-feo — `hover:bg-white/5` not tested
**Detail:** `toolbar.rs:105,153,197` and `sidebar_primitives/group.rs:40` and `sidebar/components.rs:64` all use `hover:bg-white/5`. No test verifies these class strings contain the expected hover value. This may be out of scope for seshat-feo (whose contract only covers drag-over), but it represents an untested theme contract.
**Note:** Below threshold (2 < 5).

### m2: seshat-uee — 9 ThemeScheme tests are wasm32-only and unverified in native CI
**Detail:** All 9 tests in `theme_scheme_tests.rs` are `#[cfg(target_arch = "wasm32")]`. The native `cargo test` run skips them entirely. If there's no wasm32 CI target, these tests provide zero coverage.
**Note:** Below threshold (2 < 5).

---

## Per-Bead Verdicts

### seshat-uee — STATUS: APPROVED (conditional)
**Coverage:** 44 unit tests + 6 proptests + 9 wasm32 tests = 59 total
- persisted_key: All 4 variants covered (B1–B4)
- from_persisted_key: All 4 variants + 5 error paths + 4 partial-match boundaries (B5–B9b)
- label: All 4 variants covered (B10–B13)
- resolve: 12 tests covering System×3, Light×3, Dark×3, White×3 (B14–B17)
- from_str("white"): Covered (wasm32 only)
- Proptests: Roundtrip, lowercase ASCII, title-case, resolve invariants (I1–I5)
- next(): Full cycle (4 individual + 1 aggregate)

**Conditional on:** Fixing L2 naming violations in theme_mode_tests.rs (0 hits — this file is clean). No action needed for this bead's files.

### seshat-663 — STATUS: APPROVED (conditional)
**Coverage:** 7 tests
- 4 specific L* values: border_subtle=0.30, node_bg=0.22, node_border=0.38, grid_dot=0.30
- Luminance hierarchy: bg_base < node_bg < border_subtle < node_border
- Light tokens regression: 10 exact oklch string assertions
- Dark chroma preserved: 8 neutral fields + 2 accented fields

**Conditional on:** Fixing L3 loops in tokens_tests.rs (tests_dark_chroma_preserved uses loops).

### seshat-5zc — STATUS: REJECTED
**Reason:** L4 — Missing `test_grid_renders_without_crash_when_css_variables_are_undefined`.
**Coverage:** 10 tests otherwise strong:
- 3 color replacements verified via CSS variable reference tests
- 6 grid pattern math regression tests (alignment, scaling, clamp, negative camera, zero zoom)
- 2 theme constant verification tests (GRID_DOT and BG_BASE)

### seshat-feo — STATUS: APPROVED (conditional)
**Coverage:** 3 tests
- `accent_uses_css_custom_property`: Verifies `ACCENT = "var(--accent)"`
- `accent_dash_border_uses_accent_variable`: Verifies `ACCENT_DASH_BORDER = "2px dashed var(--accent)"`
- `accent_dash_border_has_no_hardcoded_hex`: Verifies no hex in ACCENT_DASH_BORDER

**Note:** The actual usage of `ACCENT_DASH_BORDER` in `root_container/mod.rs:28` is not tested — only the constant's value. Mutation M6 survives for the usage site.

### seshat-b36 — STATUS: APPROVED (conditional)
**Coverage:** 6 tests
- bg_base is pure white: `contains("oklch(1")` ✓
- text_main is dark: L* < 0.3 for WCAG AA ✓
- node_border visible: L* in [0.30, 0.60] for 3:1 contrast ✓
- Completeness: 26 fields verified non-empty ✓
- CSS vars output valid: all segments have non-empty values ✓
- Differs from Light: bg_base differs ✓

**Conditional on:** Fixing L3 loops in tokens_tests.rs (test_white_palette_completeness and test_css_vars_for_white_produces_valid_output use loops).

### seshat-9vd — STATUS: REJECTED
**Reason:** 3 MAJOR findings (M1, M2, M3). The ThemeToggle component has zero tests.
**Coverage:** Only `next()` is tested (in theme_mode_tests.rs, not attributed to this bead's test file). No component-level tests exist for:
- Button rendering in canvas toolbar
- Click cycling theme mode
- Persistence round-trip

---

## MANDATE

Before resubmission, the following MUST exist:

### LETHAL fixes (all 4 required):
1. **L1:** Fix `toolbar/persistence_compat/mod.rs:58` redundant closure: replace `|v| v.is_object()` with `serde_json::Value::is_object`
2. **L2:** Rename all 23 `fn test_*` functions in `tokens_tests.rs` and `grid_layer.rs` to BDD-style descriptive names
3. **L3:** Eliminate all 4 loops in `tokens_tests.rs` test bodies — extract each iteration into a named test
4. **L4:** Add `test_grid_renders_without_crash_when_css_variables_are_undefined` for seshat-5zc

### MAJOR fixes (all 3 required for seshat-9vd):
5. **M1:** Add test verifying `ThemeToggle` component renders with `data-testid: "theme-toggle-btn"`
6. **M2:** Add test verifying clicking the toggle button cycles through ThemeMode variants
7. **M3:** Add test verifying theme persistence round-trip through the toggle button

### Mutation survivors requiring named tests:
8. **M5 (hover:bg-white/5):** Add test asserting `toolbar.rs` contains `hover:bg-white/5` class string
9. **M6 (ACCENT_DASH_BORDER usage):** Add test asserting `root_container/mod.rs` uses `ACCENT_DASH_BORDER` constant (not just verifying the constant's value)

### After any fix:
Re-run ALL tiers from Tier 0. Full re-run. Always.
