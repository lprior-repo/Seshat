# Contract Specification: Grid Core Module (bd-24a)

## Context

- **Feature**: Create `ui/grid/` module with `GridSize` newtype and grid snapping functions
- **Domain terms**:
  - **GridSize**: Validated grid spacing value in pixels (10.0..=100.0)
  - **Snap**: Rounding a coordinate to the nearest grid intersection
  - **Grid Multiple**: A value that is an integer multiple of the grid size
- **Assumptions**:
  - Backward compatibility with existing JSON documents that store `grid_size` as raw `f64`
  - The existing `OrderedFloat` wrapper in `EditorState` will be replaced with `GridSize`
  - `snap_point` and `snap_value` will be moved from `interaction.rs` to the new grid module
- **Open questions**: None - requirements are clear from orchestrator plan

## Preconditions

### P1: GridSize Construction (validated_grid_size)
- Input `value` must be finite (`value.is_finite()`)
- Input `value` must be in range `[10.0, 100.0]`

### P2: snap_value Function
- `grid_size` parameter: if non-positive or non-finite, treats as `1.0` (existing behavior)
- `value` parameter: any `f64` (including NaN/Infinity - handled gracefully)

### P3: snap_point Function
- Inherits `snap_value` preconditions for each coordinate
- `point` tuple: any `(f64, f64)` (including NaN/Infinity - handled gracefully)

### P4: Deserialization (serde)
- JSON value must be a valid JSON number
- After parsing as `f64`, must satisfy P1 validation constraints

## Postconditions

### Q1: GridSize Construction (validated_grid_size)
- Returns `Ok(GridSize(v))` where `v` equals the input value
- The inner value is guaranteed to be in range `[10.0, 100.0]`

### Q2: snap_value Function
- If `snap_to_grid == false`: returns `value` unchanged (identity)
- If `snap_to_grid == true`: returns a value that is a multiple of `grid_size.max(1.0)`
- Result is always finite if input is finite (NaN propagates as NaN)

### Q3: snap_point Function
- Returns `(snap_value(point.0, snap_to_grid, grid_size), snap_value(point.1, snap_to_grid, grid_size))`
- Each coordinate independently satisfies Q2

### Q4: Serialization (serde)
- Serializes as a raw `f64` number for backward compatibility
- `GridSize(20.0)` serializes to JSON `20.0` (not `{"inner": 20.0}`)

### Q5: Default Value
- `GridSize::default()` returns `GridSize(20.0)` (existing default)

## Invariants

### I1: GridSize Range
- For all `GridSize(v)`: `10.0 <= v <= 100.0`

### I2: GridSize Finite
- For all `GridSize(v)`: `v.is_finite() == true`

### I3: Snap Idempotency
- `snap_value(snap_value(x, true, g), true, g) == snap_value(x, true, g)` for all valid x, g

### I4: Snap Grid Alignment
- For finite `x` and positive finite `g`: `(snap_value(x, true, g) / g.max(1.0)).round() * g.max(1.0) == snap_value(x, true, g)`

## Error Taxonomy

```rust
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GridError {
    /// Grid size value is outside the valid range [10.0, 100.0]
    #[error("grid size must be between 10.0 and 100.0, got {value}")]
    OutOfRange {
        value: String, // Stored as String to avoid f64 in enum
    },

    /// Grid size value is not a finite number (NaN or Infinity)
    #[error("grid size must be a finite number, got {kind}")]
    NotFinite {
        kind: String, // "NaN", "Infinity", or "-Infinity"
    },

    /// Deserialization failed - value is not a valid JSON number
    #[error("grid size must be a number, got {raw}")]
    InvalidType {
        raw: String,
    },
}
```

## Contract Signatures

```rust
/// Validated grid size newtype - guarantees 10.0 <= value <= 100.0
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "f64", into = "f64")]
pub struct GridSize(f64);

impl GridSize {
    /// Minimum allowed grid size (inclusive)
    pub const MIN: f64 = 10.0;
    
    /// Maximum allowed grid size (inclusive)
    pub const MAX: f64 = 100.0;
    
    /// Default grid size (20.0)
    pub const DEFAULT: f64 = 20.0;

    /// Creates a new GridSize, returning error if out of range
    /// 
    /// # Errors
    /// - `GridError::OutOfRange` if value < 10.0 or value > 100.0
    /// - `GridError::NotFinite` if value is NaN or Infinity
    pub fn new(value: f64) -> Result<Self, GridError>;

    /// Creates a GridSize from an f64 during deserialization
    /// Clamps to valid range instead of erroring (for backward compatibility with existing docs)
    /// 
    /// This is the serde entry point - use `try_from` pattern.
    pub fn try_from_f64(value: f64) -> Result<Self, GridError>;

    /// Returns the inner f64 value
    #[must_use]
    pub const fn inner(self) -> f64;

    /// Returns the default GridSize (20.0)
    #[must_use]
    pub const fn default_value() -> Self;
}

impl Default for GridSize {
    fn default() -> Self;
}

/// Validates and creates a GridSize from a raw f64
/// 
/// # Errors
/// - `GridError::OutOfRange` if value < 10.0 or value > 100.0
/// - `GridError::NotFinite` if value is NaN or Infinity
#[must_use]
pub fn validated_grid_size(value: f64) -> Result<GridSize, GridError>;

/// Snaps a single value to the grid if snapping is enabled
/// 
/// # Guarantees
/// - If `snap_to_grid == false`, returns `value` unchanged
/// - If `grid_size <= 0` or non-finite, treats grid_size as 1.0
/// - Result is always finite if input is finite
/// - NaN input returns NaN
#[must_use]
pub fn snap_value(value: f64, snap_to_grid: bool, grid_size: f64) -> f64;

/// Snaps a point (x, y) to the grid if snapping is enabled
/// 
/// # Guarantees
/// - Applies `snap_value` independently to each coordinate
/// - See `snap_value` for additional guarantees
#[must_use]
pub fn snap_point(point: (f64, f64), snap_to_grid: bool, grid_size: f64) -> (f64, f64);
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| 10.0 <= value <= 100.0 | Runtime-checked constructor | `GridSize::new(value) -> Result<GridSize, GridError>` |
| value is finite | Runtime-checked constructor | `GridSize::new(value) -> Result<GridSize, GridError>` |
| GridSize invariant I1 | Type invariant | Private field, only constructable via `new()` |
| GridSize invariant I2 | Type invariant | Private field, only constructable via `new()` |
| snap_value grid_size fallback | Runtime behavior | `grid_size.max(1.0)` in function body |
| Deserialization valid number | Serde error | `#[serde(try_from = "f64")]` pattern |

## Violation Examples (REQUIRED)

### GridSize::new violations

- **VIOLATES P1 (below minimum)**: `GridSize::new(5.0)` -> `Err(GridError::OutOfRange { value: "5" })`
- **VIOLATES P1 (above maximum)**: `GridSize::new(150.0)` -> `Err(GridError::OutOfRange { value: "150" })`
- **VIOLATES P1 (exactly at boundary below)**: `GridSize::new(9.999999)` -> `Err(GridError::OutOfRange { value: "9.999999" })`
- **VIOLATES P1 (negative)**: `GridSize::new(-20.0)` -> `Err(GridError::OutOfRange { value: "-20" })`
- **VIOLATES P1 (zero)**: `GridSize::new(0.0)` -> `Err(GridError::OutOfRange { value: "0" })`
- **VIOLATES P1.1 (NaN)**: `GridSize::new(f64::NAN)` -> `Err(GridError::NotFinite { kind: "NaN" })`
- **VIOLATES P1.1 (positive infinity)**: `GridSize::new(f64::INFINITY)` -> `Err(GridError::NotFinite { kind: "Infinity" })`
- **VIOLATES P1.1 (negative infinity)**: `GridSize::new(f64::NEG_INFINITY)` -> `Err(GridError::NotFinite { kind: "-Infinity" })`

### validated_grid_size violations (same as GridSize::new)

- **VIOLATES P1**: `validated_grid_size(5.0)` -> `Err(GridError::OutOfRange { value: "5" })`
- **VIOLATES P1.1**: `validated_grid_size(f64::NAN)` -> `Err(GridError::NotFinite { kind: "NaN" })`

### Deserialization violations

- **VIOLATES P4 (string instead of number)**: `serde_json::from_str::<GridSize>(r#""twenty""#)` -> `Err(GridError::InvalidType { raw: "\"twenty\"" })`
- **VIOLATES P4 (object instead of number)**: `serde_json::from_str::<GridSize>(r#"{"value": 20}"#)` -> `Err(GridError::InvalidType { raw: "{\"value\":20}" })`
- **VIOLATES P4 (out of range number)**: `serde_json::from_str::<GridSize>("5.0")` -> `Err(GridError::OutOfRange { value: "5" })`

### Boundary Cases (NOT violations - these should succeed)

- `GridSize::new(10.0)` -> `Ok(GridSize(10.0))` (minimum boundary - valid)
- `GridSize::new(100.0)` -> `Ok(GridSize(100.0))` (maximum boundary - valid)
- `GridSize::new(20.0)` -> `Ok(GridSize(20.0))` (default - valid)
- `GridSize::new(50.5)` -> `Ok(GridSize(50.5))` (fractional - valid)

## Ownership Contracts

### GridSize
- **Copy type**: `GridSize` derives `Copy` - no ownership transfer concerns
- **No heap allocation**: Inner value is `f64` on stack
- **No mutation**: Type is immutable after construction

### snap_value / snap_point
- **Pure functions**: No side effects, no mutation
- **No ownership transfer**: Takes `f64` and `(f64, f64)` by value (Copy types)
- **Return by value**: Returns `f64` and `(f64, f64)` respectively

### EditorState Integration
- **Mutation**: When updating `editor_state.grid_size`, the field is replaced entirely
- **No &mut required for read**: Grid size access is read-only via `.inner()`

## Non-goals

- [ ] Changing the JSON serialization format (must remain raw f64 for backward compatibility)
- [ ] Adding grid subdivision or sub-grid snapping
- [ ] Supporting non-uniform grid spacing (different x/y grid sizes)
- [ ] Changing existing `snap_value` behavior for edge cases (must maintain backward compatibility)
- [ ] Adding grid origin offset support

## Backward Compatibility Requirements

### CRITICAL: Existing Document Compatibility

1. **JSON Deserialization**: Documents with `grid_size: 20.0` must continue to deserialize
2. **JSON Serialization**: New documents must serialize to the same format (`grid_size: 20.0`)
3. **Default Value**: Default must remain `20.0`
4. **Snap Behavior**: `snap_value(29.0, true, 20.0)` must return `20.0` (unchanged)
5. **Edge Case Behavior**: `snap_value(x, true, 0.0)` must treat grid as 1.0 (unchanged)

### Migration Path

1. Replace `OrderedFloat` with `GridSize` in `EditorState`
2. Update all call sites using `grid_size.0` to use `grid_size.inner()`
3. Move `snap_point` and `snap_value` from `interaction.rs` to `ui/grid/mod.rs`
4. Re-export from `interaction.rs` for backward compatibility during transition
