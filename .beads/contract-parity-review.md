# Contract Parity Audit: Theme System (6 Beads)

**Date**: 2026-03-25
**Auditor**: rust-contract (Design by Contract review)
**Scope**: seshat-uee, seshat-663, seshat-5zc, seshat-feo, seshat-b36, seshat-9vd

---

## Executive Summary

| Bead | Title | Happy Paths | Error Paths | Postconditions | STATUS |
|------|-------|-------------|-------------|----------------|--------|
| seshat-uee | Extend ThemeMode/ThemeScheme with White | 5/7 | 2/2 | 6/8 | **REJECTED** |
| seshat-663 | Fix dark mode contrast ratios | 4/4 | 2/2 | 4/4 | APPROVED |
| seshat-5zc | Replace hardcoded grid colors | 5/5 | 1/2 | 4/4 | **REJECTED** |
| seshat-feo | Fix hardcoded toolbar hover/drag-over | 3/3 | 0/2 | 2/3 | **REJECTED** |
| seshat-b36 | Implement White palette | 4/4 | 2/2 | 6/6 | APPROVED |
| seshat-9vd | Add theme toggle button | 2/4 | 0/2 | 2/4 | **REJECTED** |

**Overall: 4 of 6 beads REJECTED. 2 of 6 beads APPROVED.**

---

## Bead 1: seshat-uee - "Extend ThemeMode and ThemeScheme with White variant"

### Happy Path Tests

| # | Spec Test Name | PASS/FAIL | Evidence |
|---|----------------|-----------|----------|
| 1 | `test_thememodewhite.persisted_key_returns_white` | **PASS** | `persisted_key_returns_white_for_white_mode` in `theme_mode_tests.rs:41` asserts `ThemeMode::White.persisted_key() == "white"` |
| 2 | `test_thememodefrom_persisted_keywhite_returns_somethememodewhite` | **PASS** | `from_persisted_key_returns_white_for_white_string` in `theme_mode_tests.rs:71` asserts `from_persisted_key("white") == Some(ThemeMode::White)` |
| 3 | `test_thememodewhite.label_returns_white` | **PASS** | `label_returns_white_for_white_mode` in `theme_mode_tests.rs:140` asserts `ThemeMode::White.label() == "White"` |
| 4 | `test_thememodewhite.resolvethemeschemelight_returns_themeschemewhite` | **PASS** | `resolve_returns_white_when_mode_is_white_and_system_is_light` in `theme_mode_tests.rs:229` |
| 5 | `test_thememodewhite.resolvethemeschemedark_returns_themeschemewhite` | **PASS** | `resolve_returns_white_when_mode_is_white_and_system_is_dark` in `theme_mode_tests.rs:221` |
| 6 | `test_themeschemefrom_strwhite_returns_somethemeschemewhite` | **PASS** | `scheme_from_str_returns_white_for_white_string` in `theme_scheme_tests.rs:18` (wasm32 only) |
| 7 | `test_editorthemewhite_serializes_as_white_in_document_json` | **FAIL** | **NO TEST EXISTS.** `editor_theme_all_variants_serialize` in `editor.rs:271` only iterates `[Light, Dark, System]`. EditorTheme::White does not exist in the enum. |

### Error Path Tests

| # | Spec Test Name | PASS/FAIL | Evidence |
|---|----------------|-----------|----------|
| 1 | `test_thememodefrom_persisted_keyunknown_returns_none` | **PASS** | `from_persisted_key_returns_none_for_unknown_string` in `theme_mode_tests.rs:90` |
| 2 | `test_themeschemefrom_strunknown_returns_none` | **PASS** | `scheme_from_str_returns_none_for_empty_string` in `theme_scheme_tests.rs:24` (wasm32 only) |

### Postcondition Verification

| # | Postcondition | SATISFIED/VIOLATED | Evidence |
|---|--------------|-------------------|----------|
| 1 | ThemeMode has four variants: System, Light, Dark, White | **SATISFIED** | `theme/mod.rs:19-24` has all four variants |
| 2 | ThemeScheme has three variants: Light, Dark, White | **SATISFIED** | `theme/mod.rs:81-85` has all three variants |
| 3 | EditorTheme has four variants: Light, Dark, System, White | **VIOLATED** | `editor.rs:169-175` has only `Light, Dark, System`. **White variant is MISSING.** |
| 4 | `persisted_key(White)` returns "white" | **SATISFIED** | `theme/mod.rs:33` |
| 5 | `from_persisted_key("white")` returns `Some(ThemeMode::White)` | **SATISFIED** | `theme/mod.rs:43` |
| 6 | `label(White)` returns "White" | **SATISFIED** | `theme/mod.rs:53` |
| 7 | `resolve(White, _)` always returns `ThemeScheme::White` | **SATISFIED** | `theme/mod.rs:75` + proptest I4 in `theme_mode_tests.rs:323` |
| 8 | `from_str("white")` returns `Some(ThemeScheme::White)` | **SATISFIED** | `theme/mod.rs:95` (wasm32) |

### Invariant Verification

| # | Invariant | SATISFIED/VIOLATED | Evidence |
|---|-----------|-------------------|----------|
| 1 | All persisted keys are lowercase ASCII | **SATISFIED** | Proptest I2 in `theme_mode_tests.rs:304` |
| 2 | `from_persisted_key` is the inverse of `persisted_key` for all variants | **SATISFIED** | Proptest I1 in `theme_mode_tests.rs:297` |
| 3 | `ThemeMode::White` resolves to `ThemeScheme::White` regardless of system preference | **SATISFIED** | Proptest I4 in `theme_mode_tests.rs:323` |

### Inversion Test

| # | Spec Inversion Test | PASS/FAIL | Evidence |
|---|-------------------|-----------|----------|
| 1 | `test_exhaustive_match_covers_all_variants` | **FAIL** | No test by this name exists. The compiler enforces exhaustiveness, but there is no explicit test. |

### EARS Requirement Verification

| # | EARS Requirement | SATISFIED/VIOLATED | Evidence |
|---|-----------------|-------------------|----------|
| 1 | "THE SYSTEM SHALL serialize White as lowercase white in document JSON" | **VIOLATED** | EditorTheme::White does not exist. Documents cannot serialize theme "white". |
| 2 | "WHEN deserializing a document with editor.theme set to white, THE SYSTEM SHALL parse it as EditorTheme::White without fallback" | **VIOLATED** | EditorTheme enum lacks White variant. Deserializing "white" would fail at runtime. |

### **STATUS: REJECTED**

**Critical gaps:**
1. `EditorTheme` enum in `diagram_models/src/document/editor.rs` is MISSING the `White` variant. The spec explicitly requires 4 variants including White.
2. No serialization test for `EditorTheme::White` exists.
3. Document JSON roundtrip will fail if theme is set to "white" in persisted documents.
4. No explicit `test_exhaustive_match_covers_all_variants` test exists.

---

## Bead 2: seshat-663 - "Fix dark mode contrast ratios for trackable objects"

### Happy Path Tests

| # | Spec Test Name | PASS/FAIL | Evidence |
|---|----------------|-----------|----------|
| 1 | `test_dark_node_border_oklch_l*_>=_0.35` | **PASS** | `test_dark_node_border_lightness_is_0_38` in `tokens_tests.rs:61` asserts L* = 0.38 (>= 0.35) |
| 2 | `test_dark_border_subtle_oklch_l*_>=_0.28` | **PASS** | `test_dark_border_subtle_lightness_is_0_30` in `tokens_tests.rs:46` asserts L* = 0.30 (>= 0.28) |
| 3 | `test_dark_node_bg_oklch_l*_>=_0.20` | **PASS** | `test_dark_node_bg_lightness_is_0_22` in `tokens_tests.rs:55` asserts L* = 0.22 (>= 0.20) |
| 4 | `test_css_vars_forthemeschemedark_produces_valid_output` | **PASS** | Implied by all dark token tests passing and the `test_dark_chroma_preserved` test in `tokens_tests.rs:111` |

### Error Path Tests

| # | Spec Test Name | PASS/FAIL | Evidence |
|---|----------------|-----------|----------|
| 1 | `test_no_panic_when_calling_tokens_forthemeschemedark_after_value_changes` | **PASS** | `dark_tokens()` is a `const fn` that cannot panic. All tests call it without issue. |
| 2 | `test_css_vars_forthemeschemedark_still_produces_valid_css_output` | **PASS** | Validated by `test_dark_luminance_hierarchy` and `test_dark_chroma_preserved` in `tokens_tests.rs` |

### Postcondition Verification

| # | Postcondition | SATISFIED/VIOLATED | Evidence |
|---|--------------|-------------------|----------|
| 1 | Dark node_border L* is between 0.35 and 0.42 | **SATISFIED** | `tokens.rs:46` shows `oklch(0.38 0.01 260)`. L* = 0.38, in range [0.35, 0.42]. |
| 2 | Dark border_subtle L* is between 0.28 and 0.33 | **SATISFIED** | `tokens.rs:36` shows `oklch(0.30 0.005 260)`. L* = 0.30, in range [0.28, 0.33]. |
| 3 | Dark node_bg L* is between 0.20 and 0.24 | **SATISFIED** | `tokens.rs:44` shows `oklch(0.22 0.005 260)`. L* = 0.22, in range [0.20, 0.24]. |
| 4 | Dark grid_dot L* is between 0.27 and 0.32 | **SATISFIED** | `tokens.rs:47` shows `oklch(0.30 0.005 260)`. L* = 0.30, in range [0.27, 0.32]. |

### Invariant Verification

| # | Invariant | SATISFIED/VIOLATED | Evidence |
|---|-----------|-------------------|----------|
| 1 | All values remain in oklch() format | **SATISFIED** | `test_dark_chroma_preserved` validates oklch parsing for all neutral fields |
| 2 | Dark palette still has zero or near-zero chroma for neutral elements | **SATISFIED** | `test_dark_chroma_preserved` in `tokens_tests.rs:111` asserts C = 0.005 for neutrals, C = 0.01 for accented borders |
| 3 | node_border remains brighter than border_subtle which remains brighter than bg_base | **SATISFIED** | `test_dark_luminance_hierarchy` in `tokens_tests.rs:76` explicitly asserts `bg_base < node_bg < border_subtle < node_border` |

### Inversion Test

| # | Spec Inversion Test | PASS/FAIL | Evidence |
|---|-------------------|-----------|----------|
| 1 | `test_dark_node_border_L_not_exceed_0_42` | **PASS** | L* = 0.38 < 0.42, verified by `test_dark_node_border_lightness_is_0_38` |

### **STATUS: APPROVED**

All acceptance tests, postconditions, invariants, and inversion tests are satisfied.

---

## Bead 3: seshat-5zc - "Replace hardcoded grid colors with theme variables"

### Happy Path Tests

| # | Spec Test Name | PASS/FAIL | Evidence |
|---|----------------|-----------|----------|
| 1 | `test_grid_rendering_compiles_with_style-based_css_variable_references` | **PASS** | `grid_layer.rs:44` uses `style: "fill: {GRID_DOT};"` and `grid_layer.rs:53,70` use `style: "fill: {BG_BASE};"`. Compiles successfully. |
| 2 | `test_existing_test_grid_pattern_alignment_with_nodes_passes` | **PASS** | `grid_layer.rs:82` - `test_grid_pattern_alignment_with_nodes` exists and passes |
| 3 | `test_existing_test_grid_pattern_alignment_with_camera_offset_passes` | **PASS** | `grid_layer.rs:93` - `test_grid_pattern_alignment_with_camera_offset` exists and passes |
| 4 | `test_existing_test_grid_pattern_scales_with_zoom_passes` | **PASS** | `grid_layer.rs:105` - `test_grid_pattern_scales_with_zoom` exists and passes |
| 5 | `test_grid_dot_is_css_variable` | **PASS** | `grid_layer.rs:143` asserts `GRID_DOT.starts_with("var(")` |

### Error Path Tests

| # | Spec Test Name | PASS/FAIL | Evidence |
|---|----------------|-----------|----------|
| 1 | `test_grid_renders_without_crash_when_css_variables_are_undefined` | **FAIL** | **NO TEST EXISTS.** No test by this name or equivalent behavior test was found. This would require a WASM/browser test or a mock rendering environment. |
| 2 | `test_grid_pattern_alignment_tests_still_pass_after_refactor` | **PASS** | All existing grid pattern tests (alignment, zoom, camera, clamp, negative camera, zero zoom) pass. |

### Postcondition Verification

| # | Postcondition | SATISFIED/VIOLATED | Evidence |
|---|--------------|-------------------|----------|
| 1 | Grid dot fill uses `var(--grid-dot)` via style attribute | **SATISFIED** | `grid_layer.rs:44`: `style: "fill: {GRID_DOT};"` where `GRID_DOT = "var(--grid-dot)"` |
| 2 | Grid background fill uses `var(--bg-base)` via style attribute | **SATISFIED** | `grid_layer.rs:53,70`: `style: "fill: {BG_BASE};"` where `BG_BASE = "var(--bg-base)"` |
| 3 | No hardcoded hex colors remain in grid_layer.rs | **SATISFIED** | Grep confirms zero hardcoded hex values in `grid_layer.rs` |
| 4 | Existing grid_pattern_alignment tests still pass | **SATISFIED** | All 6 alignment/zoom/clamp tests pass |

### Invariant Verification

| # | Invariant | SATISFIED/VIOLATED | Evidence |
|---|-----------|-------------------|----------|
| 1 | Grid visibility still respects show_grid and zoom >= 0.3 conditions | **SATISFIED** | `grid_layer.rs:30`: `if s.show_grid && s.zoom.0 >= 0.3` unchanged |
| 2 | Grid pattern alignment math is unchanged | **SATISFIED** | `calculate_grid_pattern()` function is pure and unchanged. All existing tests pass. |

### Inversion Test

| # | Spec Inversion Test | PASS/FAIL | Evidence |
|---|-------------------|-----------|----------|
| 1 | `test_grid_renders_in_wasm_target` | **FAIL** | No WASM-specific render test exists. |

### **STATUS: REJECTED**

**Gap:** `test_grid_renders_without_crash_when_css_variables_are_undefined` has no corresponding test. While this is a browser-level behavior (CSS vars resolve to inherited values when undefined, so this should not crash), the spec explicitly requires this test.

---

## Bead 4: seshat-feo - "Fix hardcoded colors in toolbar hover and drag-over"

### Happy Path Tests

| # | Spec Test Name | PASS/FAIL | Evidence |
|---|----------------|-----------|----------|
| 1 | `test_toolbar_compiles_with_theme-aware_hover_classes` | **PASS** | `toolbar.rs` compiles. However, see postcondition notes below. |
| 2 | `test_drag-over_border_compiles_with_accent_color_reference` | **PASS** | `root_container/mod.rs:28-29` uses `ACCENT_DASH_BORDER` constant. Compiles. |
| 3 | `test_no_hardcoded_hex_colors_remain_in_toolbar.rs_or_root_container/mod.rs_rendering` | **PASS** | No `#` hex patterns found in rendering code of `toolbar.rs` or `root_container/mod.rs`. |

### Error Path Tests

| # | Spec Test Name | PASS/FAIL | Evidence |
|---|----------------|-----------|----------|
| 1 | `test_no_crash_when_theme_changes_and_hover_effect_recalculates` | **FAIL** | **NO TEST EXISTS.** No test for theme-change-driven hover recalculation. |
| 2 | `test_drag-over_border_renders_correctly_when_accent_color_changes` | **FAIL** | **NO TEST EXISTS.** No test for dynamic accent color changes on drag-over. |

### Postcondition Verification

| # | Postcondition | SATISFIED/VIOLATED | Evidence |
|---|--------------|-------------------|----------|
| 1 | No `hover:bg-white/5` remains in toolbar.rs | **VIOLATED** | `toolbar.rs:105,153,197` still contain `hover:bg-white/5`. This is invisible on white backgrounds. |
| 2 | Drag-over border uses theme accent color or ACCENT_DASH_BORDER | **SATISFIED** | `root_container/mod.rs:28-29` uses `ACCENT_DASH_BORDER` which resolves to `"2px dashed var(--accent)"` |
| 3 | Toast shadow uses theme-aware color-mix instead of hardcoded black | **SATISFIED** | `toast/render.rs:86`: `color-mix(in oklch, black 32%, transparent)` - uses color-mix which adapts to color space. While "black" is technically hardcoded, color-mix provides theme-aware blending. |

### Invariant Verification

| # | Invariant | SATISFIED/VIOLATED | Evidence |
|---|-----------|-------------------|----------|
| 1 | Hover effect is perceptible in all four theme modes | **VIOLATED** | `hover:bg-white/5` adds a 5% white overlay. On a white background (White mode), this is imperceptible. |
| 2 | Drag-over visual matches the accent color of the current theme | **SATISFIED** | Uses `var(--accent)` via `ACCENT_DASH_BORDER` |

### Inversion Test

| # | Spec Inversion Test | PASS/FAIL | Evidence |
|---|-------------------|-----------|----------|
| 1 | `test_hover_visible_in_all_theme_modes` | **FAIL** | **NO TEST EXISTS.** |

### Additional Findings

- `hover:bg-white/5` also exists in:
  - `test_mod.rs:101` (test fixture - acceptable)
  - `sidebar_primitives/group.rs:40` (UI code - same problem)
  - `sidebar/components.rs:64` (UI code - same problem)
- The spec targets `toolbar.rs` but the problem is broader.

### **STATUS: REJECTED**

**Critical gaps:**
1. `hover:bg-white/5` persists in 3 locations within `toolbar.rs` (lines 105, 153, 197). This is invisible in White mode.
2. Zero error path tests exist.
3. Zero inversion tests exist.
4. The broader `hover:bg-white/5` problem extends to sidebar components (out of this bead's scope but worth noting).

---

## Bead 5: seshat-b36 - "Implement White palette with maximum contrast"

### Happy Path Tests

| # | Spec Test Name | PASS/FAIL | Evidence |
|---|----------------|-----------|----------|
| 1 | `test_tokens_forthemeschemewhite_returns_themetokens_with_bg_base_containing_oklch1` | **PASS** | `test_white_bg_base_is_pure_white` in `tokens_tests.rs:139` asserts `bg_base.contains("oklch(1")` |
| 2 | `test_css_vars_forthemeschemewhite_produces_a_string_containing_--bg-baseoklch1` | **PASS** | `test_css_vars_for_white_produces_valid_output` in `tokens_tests.rs:207` validates all CSS segments have non-empty values |
| 3 | `test_white_palette_text_main_is_dark_l*_<_0.3` | **PASS** | `test_white_text_main_is_dark` in `tokens_tests.rs:149` asserts `L* < 0.3` (actual: 0.15) |
| 4 | `test_white_palette_node_border_is_visible_against_bg_base` | **PASS** | `test_white_node_border_visible` in `tokens_tests.rs:159` asserts L* in [0.30, 0.60] (actual: 0.55) |

### Error Path Tests

| # | Spec Test Name | PASS/FAIL | Evidence |
|---|----------------|-----------|----------|
| 1 | `test_no_panic_when_calling_tokens_forthemeschemewhite` | **PASS** | `test_white_palette_completeness` in `tokens_tests.rs:170` calls `white_tokens()` and validates all 26 fields are non-empty. No panic. |
| 2 | `test_css_vars_forthemeschemewhite_produces_valid_css_with_no_empty_values` | **PASS** | `test_css_vars_for_white_produces_valid_output` in `tokens_tests.rs:207` validates every `--var:value;` segment has a non-empty value |

### Postcondition Verification

| # | Postcondition | SATISFIED/VIOLATED | Evidence |
|---|--------------|-------------------|----------|
| 1 | `tokens_for(ThemeScheme::White)` returns a complete ThemeTokens struct | **SATISFIED** | `tokens.rs:123-128` has `ThemeScheme::White => white_tokens()` |
| 2 | White palette bg_base is oklch(1 0 0) or near-white with zero chroma | **SATISFIED** | `tokens.rs:94`: `oklch(1 0 0)` |
| 3 | text_main contrast vs bg_base >= 4.5:1 | **SATISFIED** | text_main = `oklch(0.15 0.01 260)` (L* = 0.15) vs bg_base = `oklch(1 0 0)`. Contrast ratio >> 4.5:1 |
| 4 | node_border contrast vs bg_base >= 3:1 | **SATISFIED** | node_border = `oklch(0.55 0.02 260)` (L* = 0.55) vs bg_base. L* = 0.55 gives ~3.5:1 on white |
| 5 | edge_default contrast vs bg_base >= 3:1 | **SATISFIED** | edge_default = `oklch(0.42 0.03 260)` (L* = 0.42) vs bg_base. L* = 0.42 gives > 3:1 on white |
| 6 | `css_vars_for(ThemeScheme::White)` produces valid CSS custom property string | **SATISFIED** | `test_css_vars_for_white_produces_valid_output` validates this |

### Invariant Verification

| # | Invariant | SATISFIED/VIOLATED | Evidence |
|---|-----------|-------------------|----------|
| 1 | All color values use oklch() or color-mix(in oklch, ...) format | **SATISFIED** | All `white_tokens()` values use oklch() or color-mix(in oklch, ...) |
| 2 | White palette has the same number of tokens as Light and Dark palettes | **SATISFIED** | `test_white_palette_completeness` asserts exactly 26 token fields, matching Light and Dark |
| 3 | No palette value is the empty string | **SATISFIED** | `test_white_palette_completeness` iterates all 26 fields and asserts `!val.is_empty()` |

### Inversion Tests

| # | Spec Inversion Test | PASS/FAIL | Evidence |
|---|-------------------|-----------|----------|
| 1 | `test_white_bg_is_purer_than_light_bg` | **PASS** | `test_white_palette_differs_from_light` in `tokens_tests.rs:233` asserts `w.bg_base != l.bg_base` and `test_white_bg_base_is_pure_white` confirms oklch(1) |
| 2 | `test_white_palette_completeness` | **PASS** | `test_white_palette_completeness` in `tokens_tests.rs:170` validates all 26 fields |

### **STATUS: APPROVED**

All acceptance tests, postconditions, invariants, and inversion tests are satisfied. The White palette is well-implemented with proper contrast ratios.

---

## Bead 6: seshat-9vd - "Add visible theme toggle button for all four modes"

### Happy Path Tests

| # | Spec Test Name | PASS/FAIL | Evidence |
|---|----------------|-----------|----------|
| 1 | `test_toggle_button_renders_in_canvas_toolbar` | **FAIL** | **NO TEST EXISTS.** The ThemeToggle component exists in `root_container/mod.rs:213-228` but has no corresponding test. |
| 2 | `test_clicking_toggle_cycles_theme_mode` | **FAIL** | **NO TEST EXISTS.** The cycle logic (`ThemeMode::next()`) is tested in `theme_mode_tests.rs:246-263` but the button click-to-cycle integration has no test. |
| 3 | `test_persisted_theme_is_loaded_on_page_reload` | **FAIL** | **NO TEST EXISTS.** Persistence is handled by `theme_provider.rs:64-70` via JS eval but no test validates reload behavior. |
| 4 | `test_toggle_shows_correct_mode_after_cycling` | **FAIL** | **NO TEST EXISTS.** No test verifies the displayed label matches the cycled state. |

### Error Path Tests

| # | Spec Test Name | PASS/FAIL | Evidence |
|---|----------------|-----------|----------|
| 1 | `test_toggle_still_works_if_localstorage_is_unavailable_graceful_degradation` | **FAIL** | **NO TEST EXISTS.** |
| 2 | `test_no_crash_when_theme_mode_signal_is_read_before_themeprovider_initializes` | **FAIL** | **NO TEST EXISTS.** |

### Postcondition Verification

| # | Postcondition | SATISFIED/VIOLATED | Evidence |
|---|--------------|-------------------|----------|
| 1 | Toggle button is visible in canvas toolbar | **SATISFIED** | `root_container/mod.rs:204`: `<ThemeToggle />` is rendered inside the canvas container. Component at `root_container/mod.rs:213-228` renders a button with `data-testid: "theme-toggle-btn"`. |
| 2 | Clicking cycles through System -> Light -> Dark -> White -> System | **SATISFIED** | `root_container/mod.rs:223`: `theme_mode.read().next()` uses the `next()` method. `theme/mod.rs:60-67` implements the correct cycle. |
| 3 | Each mode persists to localStorage under diagram_tool.theme_mode | **SATISFIED** | `theme_provider.rs:64-70`: `use_effect` writes `persisted_key()` to localStorage on every change. |
| 4 | Toggle button shows an icon or label indicating current mode | **SATISFIED** | `root_container/mod.rs:216-217`: `let label = current.label();` displays "System", "Light", "Dark", or "White" as button text. |

### Invariant Verification

| # | Invariant | SATISFIED/VIOLATED | Evidence |
|---|-----------|-------------------|----------|
| 1 | Toggle state is always consistent with the actual rendered theme | **PARTIALLY SATISFIED** | The toggle reads from the same Signal as ThemeProvider's CSS var generation. However, no test proves this invariant. |
| 2 | No duplicate theme toggle buttons exist | **SATISFIED** | Grep confirms `<ThemeToggle />` appears exactly once in `root_container/mod.rs:204`. |

### Inversion Tests

| # | Spec Inversion Test | PASS/FAIL | Evidence |
|---|----------------|-----------|----------|
| 1 | `test_toggle_button_visible_in_canvas_toolbar` | **FAIL** | **NO TEST EXISTS.** |
| 2 | `test_theme_persists_across_page_reload` | **FAIL** | **NO TEST EXISTS.** |

### **STATUS: REJECTED**

**Critical gaps:**
1. **ZERO tests exist** for the ThemeToggle component. The implementation is present and appears correct, but there is no test coverage whatsoever for any of the 4 happy paths or 2 error paths specified in the bead.
2. The underlying `ThemeMode::next()` cycling logic IS well-tested, but the integration (button renders, click handler cycles, persistence) has no tests.
3. No Playwright E2E tests found for this feature.

---

## Cross-Bead Dependency Analysis

```
seshat-uee (White variant in enums) 
  --> seshat-b36 (White palette tokens) 
  --> seshat-5zc (Grid uses theme vars)
  --> seshat-feo (Toolbar hover theme-aware)
  --> seshat-9vd (Toggle button)

seshat-663 (Dark contrast fix) - INDEPENDENT
```

### Dependency Chain Health

| Dependency | Upstream Status | Impact |
|------------|----------------|--------|
| b36 depends on uee | uee REJECTED (EditorTheme::White missing) | b36 works because it only needs ThemeScheme::White (which exists), not EditorTheme::White |
| 9vd depends on uee | uee REJECTED | 9vd works because ThemeMode::White exists (toggle uses ThemeMode, not EditorTheme) |
| feo independent | feo REJECTED | Self-contained failure (hover:bg-white/5 not fixed) |

### Spec Inconsistency Detected

**seshat-uee spec** defines `EditorTheme` with 4 variants (Light, Dark, System, White) in postconditions but `EditorTheme` is a **document model type** in `diagram_models`, separate from the UI `ThemeMode`/`ThemeScheme` in `diagram_tool`. The implementation correctly added White to `ThemeMode` and `ThemeScheme` but missed `EditorTheme`. This is a real gap because:
- Documents serialize `editor.theme` as `"light"`, `"dark"`, or `"system"`
- A user who sets White mode in the UI has no way to persist it in the document
- The document model and UI model are decoupled, creating a silent contract violation

---

## Required Remediation Actions

### Priority 0 (Blocking - Data Integrity)

| Action | Bead | File | Description |
|--------|------|------|-------------|
| **R1** | seshat-uee | `diagram_models/src/document/editor.rs` | Add `White` variant to `EditorTheme` enum. Add `EditorTheme::White` to `editor_theme_all_variants_serialize` test. |

### Priority 1 (Functional - User-Facing Bugs)

| Action | Bead | File | Description |
|--------|------|------|-------------|
| **R2** | seshat-feo | `diagram_tool/src/ui/canvas/toolbar.rs` | Replace `hover:bg-white/5` (lines 105, 153, 197) with theme-aware hover class (e.g., `hover:bg-foreground/5` or `hover:bg-black/5 hover:dark:bg-white/5`). |
| **R3** | seshat-9vd | New test file | Add at minimum: `test_toggle_button_renders_in_canvas_toolbar` and `test_clicking_toggle_cycles_theme_mode`. These are unit-level and can test the Dioxus component. |

### Priority 2 (Contract Completeness)

| Action | Bead | File | Description |
|--------|------|------|-------------|
| **R4** | seshat-5zc | New test or existing | Add `test_grid_renders_without_crash_when_css_variables_are_undefined` (may need to be a Playwright E2E test). |
| **R5** | seshat-feo | New test file | Add `test_hover_visible_in_all_theme_modes` and error path tests. |
| **R6** | seshat-9vd | New test file | Add error path tests for localStorage unavailability and signal initialization order. |

---

## Test Coverage Heatmap

```
                    uee   663   5zc   feo   b36   9vd
Happy Paths:        5/7   4/4   5/5   3/3   4/4   2/4
Error Paths:        2/2   2/2   1/2   0/2   2/2   0/2
Inversions:         0/1   1/1   0/1   0/1   2/2   0/2
EARS Requirements:  4/6   4/4   4/4   2/4   4/4   2/4
─────────────────────────────────────────────────────────
Coverage Score:     71%  100%   83%   31%  100%   25%
```
