# seshat-pw3 Defects

## Critical Issues (Must Fix)

### Phase 2: Farley Rigor Violations

1. **EXCEEDS 25-LINE LIMIT** - `apply_group_selection` (mod.rs:123-214)
   - 92 lines - 368% over limit
   - Contains: filtering, bounds calculation, node creation, reparenting
   - **FIX:** Break into: `collect_group_bounds()`, `create_group_node()`, `reparent_children()`

2. **EXCEEDS 25-LINE LIMIT** - `apply_align_selection` (alignment.rs:50-181)
   - 132 lines - 528% over limit
   - Contains: node filtering, bounding box calc (duplicated for H/V), alignment logic
   - **FIX:** Extract `compute_horizontal_bounds()`, `compute_vertical_bounds()`, `compute_alignment_target()`

3. **EXCEEDS 25-LINE LIMIT** - `apply_distribute_selection` (alignment.rs:202-314)
   - 113 lines - 452% over limit
   - **FIX:** Extract `collect_node_data()`, `compute_distribution_spacing()`, `apply_distribution()`

4. **EXCEEDS 25-LINE LIMIT** - `apply_ungroup_selection` (mod.rs:216-264)
   - 49 lines - 196% over limit

5. **EXCEEDS 25-LINE LIMIT** - `apply_delete_selected` (mod.rs:76-121)
   - 46 lines - 184% over limit

### Phase 3: Types as Documentation Violation

6. **BOOLEAN PARAMETER FLAG** - `apply_nudge_selection` (nudge.rs:33)
   ```rust
   pub fn apply_nudge_selection(..., push_undo: bool) -> bool
   ```
   - Control flag creates invisible state branches
   - **FIX:** Either split into two functions or use enum `enum UndoBehavior { WithUndo, WithoutUndo }`

### Phase 4: I/O Mixed with Calculations

7. **HISTORY PUSH EMBEDDED IN CALC** - Multiple files
   - `nudge.rs`:44, `alignment.rs`:137, `z_order.rs`:117, `clipboard.rs`:240,268
   - `push_history()` is called inside pure calculation functions
   - **FIX:** Move history push to shell layer, pass pure functions to caller

---

## Moderate Issues

8. **DUPLICATED ZOOM LOGIC** (mod.rs:266-328)
   - Three functions with identical bounds-checking patterns
   - **FIX:** Extract `zoom_by_factor(doc, factor, viewport) -> bool`

9. **INTERNAL MODULE BLOAT** (mod.rs:364-411)
   - `zoom` module contains only two 15-line functions
   - **FIX:** Inline or merge with main module

---

## Summary

| Severity | Count | Phase |
|----------|-------|-------|
| Critical | 7 | Phase 2-3 |
| Moderate | 2 | Phase 5 |

**Root Cause:** The refactoring extracted code into submodules but did NOT refactor the extracted code to follow the 25-line limit. Functions were simply moved, not decomposed.
