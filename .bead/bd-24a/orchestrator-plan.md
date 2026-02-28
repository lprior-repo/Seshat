# bd-24a: Grid Core State and Coordinate Conversion

**Bead ID**: bd-24a
**Status**: Planning
**Effort**: 2hr
**Priority**: 2

---

## 1. Clarifications

### Problem Statement
The codebase has grid snapping logic scattered across `ui/interaction.rs` using raw `f64` values for grid size. The `EditorState` stores `grid_size` as `OrderedFloat<f64>` without validation. There is no centralized grid module with proper type safety.

### Scope
- Create new `ui/grid/` module with validated `GridSize` newtype
- Move snap functions to grid module with `GridSize` parameter
- Update `EditorState` to use `GridSize` instead of `OrderedFloat`
- Update `canvas.rs` to import from new grid module

### Out of Scope
- Changes to `layout/grid.rs` (layout calculations, different concern)
- UI components for grid settings
- Grid rendering/visualization

---

## 2. EARS Requirements

### Ubiquitous
- THE SYSTEM SHALL provide a `GridSize` newtype that wraps a positive finite `f64` value
- THE SYSTEM SHALL provide `snap_point` and `snap_value` functions that accept `GridSize`
- THE SYSTEM SHALL validate grid size values are positive and finite

### Event-Driven
- WHEN a grid size is deserialized from JSON, THE SYSTEM SHALL validate and clamp to valid range
- WHEN an invalid grid size is provided, THE SYSTEM SHALL substitute the default value (20.0)
- WHEN snap functions are called, THE SYSTEM SHALL use the validated grid size

### Unwanted
- IF a NaN or infinite value is provided for grid size, THE SYSTEM SHALL NOT store it
- IF a zero or negative grid size is provided, THE SYSTEM SHALL NOT accept it
- IF the grid module is imported, THE SYSTEM SHALL NOT expose raw f64 grid parameters

---

## 3. KIRK Contracts

### Preconditions
- `GridSize::new(value)` requires `value > 0.0 && value.is_finite()`
- `snap_point(point, snap_to_grid, grid_size)` requires `grid_size` is a valid `GridSize`
- Deserialization of `EditorState.grid_size` must handle invalid values gracefully

### Postconditions
- `GridSize::new()` returns `Some(GridSize)` for valid input, `None` for invalid
- `GridSize::validated_or_default()` always returns a valid `GridSize`
- `snap_value` returns a value that is a multiple of the grid size when snapping is enabled
- `snap_point` returns a point where both coordinates are multiples of grid size when snapping is enabled

### Invariants
- `GridSize` inner value is always `> 0.0` and finite
- Default grid size is always `20.0`
- Grid size range is `[1.0, 1000.0]`

---

## 4. Research Requirements

### Already Understood
- Current `snap_point` implementation in `ui/interaction.rs` (lines 97-112)
- `EditorState` structure in `models/document.rs` (lines 264-301)
- Import patterns in `canvas.rs` (line 27-30)

### Needs Investigation
- [ ] Serde serialization behavior for newtypes with validation
- [ ] Whether `GridSize` should implement `FromStr` for CLI/config parsing
- [ ] Impact on existing serialization format (backwards compatibility)

---

## 5. Inversions (What Could Go Wrong)

### Type Mismatches
- Risk: `GridSize` vs `f64` confusion at call sites
- Mitigation: Use explicit type annotations, deprecate old functions

### Serialization Breakage
- Risk: Existing saved documents with invalid grid_size values fail to load
- Mitigation: Use `#[serde(deserialize_with)]` with fallback to default

### Import Conflicts
- Risk: Both old and new `snap_point` imported simultaneously
- Mitigation: Remove old import, use new module exclusively

### Circular Dependencies
- Risk: `ui/grid` imports from `models/document` which imports `GridSize`
- Mitigation: Define `GridSize` in `models/document.rs` or separate module

---

## 6. ATDD Tests (Unit)

### Happy Path
```rust
#[test]
fn given_valid_value_when_grid_size_new_then_returns_some() {
    assert!(GridSize::new(20.0).is_some());
    assert!(GridSize::new(1.0).is_some());
    assert!(GridSize::new(100.0).is_some());
}

#[test]
fn given_grid_size_when_snapping_then_rounds_to_multiple() {
    let grid = GridSize::new(20.0).unwrap();
    let result = snap_value(29.0, true, grid);
    assert!((result - 20.0).abs() < f64::EPSILON);
}

#[test]
fn given_snap_disabled_when_snapping_then_returns_original() {
    let grid = GridSize::new(20.0).unwrap();
    let result = snap_value(29.0, false, grid);
    assert!((result - 29.0).abs() < f64::EPSILON);
}
```

### Error Path
```rust
#[test]
fn given_invalid_value_when_grid_size_new_then_returns_none() {
    assert!(GridSize::new(0.0).is_none());
    assert!(GridSize::new(-5.0).is_none());
    assert!(GridSize::new(f64::NAN).is_none());
    assert!(GridSize::new(f64::INFINITY).is_none());
}

#[test]
fn given_invalid_deserialized_value_when_validated_then_uses_default() {
    let grid = GridSize::validated_or_default(f64::NAN);
    assert_eq!(grid.get(), 20.0);
}
```

### Edge Cases
```rust
#[test]
fn given_min_value_when_grid_size_new_then_succeeds() {
    assert!(GridSize::new(1.0).is_some());
}

#[test]
fn given_just_below_min_when_grid_size_new_then_fails() {
    assert!(GridSize::new(0.999).is_none());
}
```

---

## 7. E2E Tests (Integration)

### Canvas Interaction
1. Load diagram with grid_size in JSON
2. Verify canvas snaps nodes to grid when moving
3. Verify snap toggle works correctly

### Serialization Round-Trip
1. Create document with custom grid size
2. Serialize to JSON
3. Deserialize and verify grid_size preserved

### Backwards Compatibility
1. Load old document without grid_size field
2. Verify default grid_size (20.0) is used
3. Load document with invalid grid_size (e.g., -10)
4. Verify default is substituted

---

## 8. Verification Checkpoints

### Checkpoint 1: GridSize Type Created
- [ ] `GridSize` newtype defined in `models/document.rs` or new module
- [ ] `GridSize::new()` validates and returns Option
- [ ] `GridSize::validated_or_default()` always returns valid value
- [ ] `GridSize::get()` returns inner f64

### Checkpoint 2: Snap Functions Migrated
- [ ] `snap_value(value, snap, GridSize)` defined in `ui/grid/`
- [ ] `snap_point(point, snap, GridSize)` defined in `ui/grid/`
- [ ] Functions match existing behavior

### Checkpoint 3: EditorState Updated
- [ ] `EditorState.grid_size` type changed to `GridSize`
- [ ] Serialization uses validated deserializer
- [ ] Default value is `GridSize::default()` (20.0)

### Checkpoint 4: Canvas Integration
- [ ] `canvas.rs` imports from `ui/grid/`
- [ ] All call sites use `GridSize` type
- [ ] No direct `f64` grid parameters remain

### Checkpoint 5: Compilation & Tests Pass
- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

---

## 9. Implementation Tasks

### Task 1: Define GridSize Newtype (15min)
**File**: `diagram_tool/src/models/document.rs`

```rust
/// Validated grid size for canvas snapping.
/// Guarantees: value is finite and in range [1.0, 1000.0]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct GridSize(f64);

impl GridSize {
    const MIN: f64 = 1.0;
    const MAX: f64 = 1000.0;
    const DEFAULT: f64 = 20.0;

    #[must_use]
    pub fn new(value: f64) -> Option<Self> {
        if value.is_finite() && value >= Self::MIN && value <= Self::MAX {
            Some(Self(value))
        } else {
            None
        }
    }

    #[must_use]
    pub fn validated_or_default(value: f64) -> Self {
        Self::new(value).unwrap_or_default()
    }

    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl Default for GridSize {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}
```

### Task 2: Create ui/grid Module (30min)
**File**: `diagram_tool/src/ui/grid/mod.rs`

```rust
mod types;
mod snap;

pub use snap::{snap_point, snap_value};
pub use types::GridSize;
```

**File**: `diagram_tool/src/ui/grid/snap.rs`

```rust
use super::types::GridSize;

#[must_use]
pub fn snap_value(value: f64, snap_to_grid: bool, grid_size: GridSize) -> f64 {
    if !snap_to_grid {
        return value;
    }
    let step = grid_size.get().max(1.0);
    (value / step).round() * step
}

#[must_use]
pub fn snap_point(point: (f64, f64), snap_to_grid: bool, grid_size: GridSize) -> (f64, f64) {
    (
        snap_value(point.0, snap_to_grid, grid_size),
        snap_value(point.1, snap_to_grid, grid_size),
    )
}
```

### Task 3: Add Serde Support for GridSize (15min)
**File**: `diagram_tool/src/models/document.rs`

```rust
impl Serialize for GridSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for GridSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Ok(Self::validated_or_default(value))
    }
}
```

### Task 4: Update EditorState (15min)
**File**: `diagram_tool/src/models/document.rs`

Change line 271:
```rust
// Before
#[serde(default = "default_grid_size")]
pub grid_size: OrderedFloat,

// After
#[serde(default)]
pub grid_size: GridSize,
```

Remove `default_grid_size()` function (no longer needed).

### Task 5: Update ui/mod.rs (5min)
**File**: `diagram_tool/src/ui/mod.rs`

Add:
```rust
pub mod grid;
```

### Task 6: Update canvas.rs Imports (10min)
**File**: `diagram_tool/src/ui/canvas.rs`

Change line 27-30:
```rust
// Before
use crate::ui::interaction::{
    drag_original_positions, dragged_positions_with_snap, has_drag_threshold, node_ids_in_rect,
    select_single, snap_point, snap_value, toggle_selection, with_auto_selected_edges,
};

// After
use crate::ui::grid::{snap_point, snap_value};
use crate::ui::interaction::{
    drag_original_positions, dragged_positions_with_snap, has_drag_threshold, node_ids_in_rect,
    select_single, toggle_selection, with_auto_selected_edges,
};
```

### Task 7: Update Call Sites in canvas.rs (20min)
**File**: `diagram_tool/src/ui/canvas.rs`

Find all usages of `grid_size.0` and replace with `grid_size.get()`:
- Line ~202: `doc.editor_state.grid_size.0` -> `doc.editor_state.grid_size.get()`
- Line ~381: `doc.editor_state.grid_size.0` -> `doc.editor_state.grid_size.get()`
- Line ~435: `doc.editor_state.grid_size.0` -> `doc.editor_state.grid_size.get()`
- Line ~1039-1041: Update snap_point calls

### Task 8: Update interaction.rs (15min)
**File**: `diagram_tool/src/ui/interaction.rs`

Option A: Keep old functions as deprecated wrappers:
```rust
#[deprecated(since = "0.1.0", note = "Use crate::ui::grid::snap_value instead")]
#[must_use]
pub fn snap_value(value: f64, snap_to_grid: bool, grid_size: f64) -> f64 {
    crate::ui::grid::snap_value(value, snap_to_grid, GridSize::validated_or_default(grid_size))
}
```

Option B: Remove functions entirely (preferred for clean migration).

Update `dragged_positions_with_snap` to accept `GridSize`.

### Task 9: Run Quality Gates (10min)
```bash
cd diagram_tool && cargo check
cd diagram_tool && cargo test
cd diagram_tool && cargo clippy -- -D warnings
```

---

## 10. Failure Modes

### Mode 1: Serialization Format Change
- **Symptom**: Old documents fail to load
- **Cause**: `GridSize` serialization differs from raw f64
- **Fix**: Ensure `Serialize` impl outputs raw f64

### Mode 2: Type Mismatch at Call Sites
- **Symptom**: Compilation errors about `GridSize` vs `f64`
- **Cause**: Missed call site update
- **Fix**: Use compiler errors as guide, update each call site

### Mode 3: Infinite Recursion in Default
- **Symptom**: Stack overflow when creating default GridSize
- **Cause**: `Default` impl calls `validated_or_default` which calls `default`
- **Fix**: Use const DEFAULT value directly

---

## 11. Anti-Hallucination

### Verified Facts
- `snap_point` exists at `ui/interaction.rs:107-112`
- `EditorState.grid_size` is `OrderedFloat` at `models/document.rs:271`
- `canvas.rs` imports `snap_point` at line 29
- Default grid size is `20.0` at `models/document.rs:303-305`

### Unverified Assumptions
- [ ] All call sites can be found by searching for `grid_size.0`
- [ ] No other modules use `snap_point` from `interaction.rs`

### Commands to Verify
```bash
rg "snap_point" --type rust
rg "grid_size\.0" --type rust
rg "OrderedFloat.*grid" --type rust
```

---

## 12. Context Survival

### Key Files
1. `diagram_tool/src/models/document.rs` - GridSize definition, EditorState
2. `diagram_tool/src/ui/grid/mod.rs` - New grid module
3. `diagram_tool/src/ui/grid/snap.rs` - Snap functions
4. `diagram_tool/src/ui/canvas.rs` - Primary consumer
5. `diagram_tool/src/ui/interaction.rs` - Legacy snap functions

### Key Types
- `GridSize` - newtype for validated grid size
- `EditorState` - contains `grid_size: GridSize`
- `OrderedFloat` - existing float wrapper, being replaced for grid_size

### Key Functions
- `GridSize::new(f64) -> Option<GridSize>`
- `GridSize::validated_or_default(f64) -> GridSize`
- `snap_value(f64, bool, GridSize) -> f64`
- `snap_point((f64, f64), bool, GridSize) -> (f64, f64)`

---

## 13. Completion Checklist

- [ ] `GridSize` newtype created with validation
- [ ] `GridSize` implements Serialize/Deserialize with validation
- [ ] `ui/grid/` module created with `mod.rs` and `snap.rs`
- [ ] `snap_value` and `snap_point` accept `GridSize`
- [ ] `EditorState.grid_size` changed to `GridSize`
- [ ] `canvas.rs` imports from `ui/grid/`
- [ ] All `grid_size.0` replaced with `grid_size.get()`
- [ ] `ui/mod.rs` exports `grid` module
- [ ] `interaction.rs` updated or deprecated
- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes
- [ ] Existing tests still pass

---

## 14. Context

### Why This Bead Exists
The original problem states that a `ui/grid/` module was "created" but `canvas.rs` still uses raw `f64` from `interaction.rs`. This bead creates the proper grid module with type-safe `GridSize` and integrates it throughout the codebase.

### Dependencies
- None (this is a foundational type)

### Dependents
- Future beads for grid UI controls
- Future beads for coordinate conversion utilities

---

## 15. AI Hints

### Pattern Recognition
This follows the "Parse, Don't Validate" pattern - `GridSize` is a parsed/validated type that guarantees invariants by construction.

### Common Mistakes to Avoid
1. Forgetting to update `interaction.rs` tests that use old `snap_point` signature
2. Using `GridSize::new().unwrap()` instead of `validated_or_default()`
3. Not handling the `Eq` trait (GridSize can't impl Eq due to f64)

### Incremental Approach
1. First define `GridSize` type
2. Then add serde impls
3. Then update `EditorState`
4. Then create `ui/grid/` module
5. Then update imports in `canvas.rs`
6. Finally update `interaction.rs`

### Test Strategy
Keep existing property tests in `interaction.rs` - they should pass unchanged after migration. Add new tests for `GridSize` validation.

---

## 16. Exit Criteria

### Must Have
- [ ] `GridSize` newtype with validation exists
- [ ] `ui/grid/` module with snap functions exists
- [ ] `EditorState.grid_size` uses `GridSize`
- [ ] `canvas.rs` uses new grid module
- [ ] All tests pass

### Should Have
- [ ] Property tests for `GridSize`
- [ ] Deprecation warnings on old functions

### Nice to Have
- [ ] `Display` impl for `GridSize`
- [ ] `FromStr` impl for `GridSize`

---

## Receipt

### Objective
Create `ui/grid/` module with `GridSize` newtype and snap functions; integrate into `EditorState` and `canvas.rs` to replace raw `f64` grid size handling with type-safe validated types.

### Allowed Scope
- `diagram_tool/src/models/document.rs` - Add `GridSize`, update `EditorState`
- `diagram_tool/src/ui/grid/` - New module (mod.rs, snap.rs)
- `diagram_tool/src/ui/mod.rs` - Export grid module
- `diagram_tool/src/ui/canvas.rs` - Update imports and call sites
- `diagram_tool/src/ui/interaction.rs` - Deprecate/remove old snap functions

### Files Touched
- `diagram_tool/src/models/document.rs` (modify)
- `diagram_tool/src/ui/grid/mod.rs` (create)
- `diagram_tool/src/ui/grid/snap.rs` (create)
- `diagram_tool/src/ui/mod.rs` (modify)
- `diagram_tool/src/ui/canvas.rs` (modify)
- `diagram_tool/src/ui/interaction.rs` (modify)

### Commands
```bash
# Verification
cd diagram_tool && cargo check
cd diagram_tool && cargo test
cd diagram_tool && cargo clippy -- -D warnings
```

### Exit Codes
- `cargo check`: 0
- `cargo test`: 0
- `cargo clippy`: 0

### Key stdout/stderr
- No warnings from clippy
- All existing tests pass
- New GridSize tests pass

### Diff Summary
- +80 lines: New `ui/grid/` module
- +50 lines: `GridSize` type and impls in `document.rs`
- -5 lines: Remove old snap functions from `interaction.rs`
- ~20 lines changed: `canvas.rs` imports and call sites
- ~5 lines changed: `EditorState` definition

### Risks/Unknowns
1. **Serialization compatibility**: Must verify old JSON documents load correctly
2. **Call site coverage**: May have missed some `grid_size.0` usages
3. **Test coverage**: Existing property tests may need parameter type updates

### Pass/Fail Recommendation
**PASS** when:
- All compilation errors resolved
- All tests pass (existing + new)
- Clippy reports zero warnings
- Manual verification: load existing document with grid_size

**FAIL** if:
- Any serialization format change breaks backward compatibility
- Any test regression
- Clippy warnings introduced
