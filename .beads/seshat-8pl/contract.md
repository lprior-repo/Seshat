# Contract Specification

## Context
- Feature: SNP-001 to SNP-005 Grid snapping (Round node coordinates to nearest grid step, e.g. 10px).
- Domain terms: 
  - `GridSize`: A strictly validated, finite grid step value between 10.0 and 100.0.
  - `SnapMode`: Explicit enum state (`Enabled` or `Disabled`).
  - `GridSnapError`: Contract error taxonomy for snapping operations.
  - `GridError`: Contract error taxonomy for grid initialization.
- Assumptions:
  - Coordinate system is Cartesian f64.
  - Coordinates must be finite; non-finite coordinates are illegal states.
- Open questions:
  - None at this time.

## Preconditions
- [ ] P1: Input raw coordinates (`x` and `y`) must be finite.
- [ ] P2: Grid step size must be finite and within the inclusive range `[10.0, 100.0]`.

## Postconditions
- [ ] Q1: When `SnapMode::Disabled`, the returned coordinate exactly equals the input raw coordinate.
- [ ] Q2: When `SnapMode::Enabled`, the returned coordinate is a multiple of `GridSize`.
- [ ] Q3: The distance between the raw coordinate and the snapped coordinate is strictly `<= GridSize / 2` (round to nearest).
- [ ] Q4: Halfway values (exactly `GridSize / 2`) tie-break deterministically away from zero.
- [ ] Q5: All returned coordinates are strictly finite.

## Invariants
- [ ] I1: `GridSize` inner value never mutates and always remains finite and within `[10.0, 100.0]`. (Compiler-enforced via Rust `Copy` semantics and encapsulation).

## Error Taxonomy
- `GridError::OutOfRange` - when initializing `GridSize` with value < 10.0 or > 100.0.
- `GridError::NotFinite` - when initializing `GridSize` with NaN or Infinity.
- `GridSnapError::NotFinite` - when `snap_node_coordinate` or `snap_node_coordinates` receives a non-finite coordinate value.

## Contract Signatures
- `pub fn try_grid_size(raw_step: f64) -> Result<GridSize, GridError>`
- `pub fn snap_node_coordinate(raw_value: f64, mode: SnapMode, grid: GridSize) -> Result<f64, GridSnapError>`
- `pub fn snap_node_coordinates(raw_point: (f64, f64), mode: SnapMode, grid: GridSize) -> Result<(f64, f64), GridSnapError>`

## Type Encoding
For each precondition, specify the strongest possible type enforcement:
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: finite coordinates | Result error variant | `Result<f64, GridSnapError::NotFinite>` |
| P2: grid size bounds | Runtime-checked constructor | `try_grid_size(val) -> Result<GridSize, GridError>` |
| explicit snap state | Compile-time | `enum SnapMode { Enabled, Disabled }` |
| Q1-Q5: postconditions | Debug-only | `debug_assert!()` |

## Violation Examples (REQUIRED -- one per precondition and postcondition)
- VIOLATES P1: `snap_node_coordinate(f64::NAN, SnapMode::Enabled, valid_grid)` -- should produce `Err(GridSnapError::NotFinite)`
- VIOLATES P2: `try_grid_size(9.9)` -- should produce `Err(GridError::OutOfRange)`
- VIOLATES P2: `try_grid_size(f64::NAN)` -- should produce `Err(GridError::NotFinite)`
- VIOLATES Q1: `snap_node_coordinate(15.5, SnapMode::Disabled, grid_20)` returning `20.0` -- should panic (debug_assert failed)
- VIOLATES Q2: `snap_node_coordinate(15.0, SnapMode::Enabled, grid_20)` returning `15.0` -- should panic (debug_assert failed)
- VIOLATES Q3: `snap_node_coordinate(19.0, SnapMode::Enabled, grid_10)` returning `10.0` -- should panic (debug_assert failed)
- VIOLATES Q4: `snap_node_coordinate(15.0, SnapMode::Enabled, grid_10)` returning `10.0` -- should panic (debug_assert failed)
- VIOLATES Q5: `snap_node_coordinate(15.0, SnapMode::Enabled, grid_10)` returning `f64::NAN` -- should panic (debug_assert failed)

## Ownership Contracts (Rust-specific)
- Ownership transfer: None. Coordinate primitives (`f64`, `(f64, f64)`) and contract types (`GridSize`, `SnapMode`) implement `Copy`.
- Shared borrow: Not applicable.
- Exclusive borrow: None. Pure functional transformation (Data -> Calc).
- Clone policy: All input and output types `Copy` cleanly.

## Non-goals
- Multi-node alignment operations.
- Z-order snapping.
