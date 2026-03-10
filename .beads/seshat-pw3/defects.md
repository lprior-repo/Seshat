# seshat-pw3 Defects

## Critical Issues (Must Fix)

### Phase 2: Farley Rigor Violations (STILL PRESENT)

1. **EXCEEDS 25-LINE LIMIT** - `apply_align_selection` (alignment.rs:59-84)
   - 26 lines - exceeds by 1 line
   - **FIX:** Reduce by 1 line

2. **EXCEEDS 25-LINE LIMIT** - `create_group_node` (mod.rs:214-252)
   - 39 lines - 156% over limit
   - **FIX:** Break into smaller helpers

3. **EXCEEDS 25-LINE LIMIT** - `collect_group_bounds` (mod.rs:181-212)
   - 32 lines - 128% over limit
   - **FIX:** Extract fold logic into separate function

4. **EXCEEDS 25-LINE LIMIT** - `apply_alignment_to_nodes` (alignment.rs:157-190)
   - 34 lines - 136% over limit
   - **FIX:** Extract match arms into separate functions

5. **EXCEEDS 25-LINE LIMIT** - `apply_z_order_operation` (z_order.rs:63-120)
   - 58 lines - 232% over limit (HIGHEST PRIORITY)
   - **FIX:** Break into: filter logic, z-order calculation, apply logic

6. **EXCEEDS 25-LINE LIMIT** - `apply_z_order_to_ids` (z_order.rs:122-165)
   - 44 lines - 176% over limit
   - **FIX:** Extract each match arm into separate functions

7. **EXCEEDS 25-LINE LIMIT** - `set_zoom_centered` (mod.rs:416-444)
   - 29 lines - 116% over limit
   - **FIX:** Extract clamping logic

8. **EXCEEDS 25-LINE LIMIT** - `compute_horizontal_bounds` (alignment.rs:99-126)
   - 28 lines
   - **FIX:** Extract min/max fold into helper

9. **EXCEEDS 25-LINE LIMIT** - `compute_vertical_bounds` (alignment.rs:128-155)
   - 28 lines
   - **FIX:** Extract min/max fold into helper

---

## Fixed Issues ✅

1. ~~BOOLEAN PARAMETER FLAG~~ - Now uses `UndoBehavior` enum
2. ~~HISTORY PUSH IN CALC~~ - Now in shell layer
3. ~~apply_distribute_selection~~ - Now 16 lines
4. ~~apply_ungroup_selection~~ - Now 15 lines  
5. ~~apply_delete_selected~~ - Now 14 lines
6. Pure calculation functions extracted: `calculate_alignment()`, `calculate_distribution()`

---

## Summary

| Severity | Count | Phase |
|----------|-------|-------|
| Critical | 9 | Phase 2 |
| Fixed | 6 | Phase 2-3 |

**Root Cause:** The refactoring extracted code into submodules but did NOT decompose the extracted code to meet the 25-line limit. Functions were simply moved, not refactored.

**Next Steps:**
1. Decompose `apply_z_order_operation` (58 lines) - highest priority
2. Decompose `create_group_node` (39 lines)
3. Decompose `apply_alignment_to_nodes` (34 lines)
4. Fix remaining 5 functions that exceed 25 lines
