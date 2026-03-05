# Contract Specification

## Context
- **Feature**: Finite numeric validation for OrderedFloat - reject NaN/Infinity at construction
- **Domain terms**: OrderedFloat (f64 wrapper), finite (not NaN/Inf), schema validation, validation issues
- **Assumptions**: Existing codebase uses OrderedFloat throughout; serde deserialization creates nodes directly
- **Open questions**: Should we add a new `FiniteFloat` type or modify `OrderedFloat`? How to handle serde deserialization?

## Preconditions
- [P1] OrderedFloat constructor must reject NaN values
- [P2] OrderedFloat constructor must reject positive Infinity
- [P3] OrderedFloat constructor must reject negative Infinity
- [P4] All numeric fields (x, y, width, height, label_offset_t, thickness, camera_x, camera_y, zoom, font_size) must be finite

## Postconditions
- [Q1] OrderedFloat::new() returns Ok(_) for finite f64 values
- [Q2] OrderedFloat::new() returns Err(OrderedFloatError::NaN) when given NaN
- [Q3] OrderedFloat::new() returns Err(OrderedFloatError::Infinite) when given Inf/-Inf
- [Q4] Schema validation returns errors for non-finite node coordinates
- [Q5] Schema validation returns errors for non-finite edge properties
- [Q6] Validation returns issues for NaN/Inf in any numeric field

## Invariants
- [I1] OrderedFloat always contains a finite f64 value (is_finite() == true)
- [I2] No arithmetic on OrderedFloat can produce NaN/Inf (unless given invalid input)
- [I3] Comparison operations are total (no panic on NaN)

## Error Taxonomy
- `OrderedFloatError::NaN` - when a NaN value is provided to constructor
- `OrderedFloatError::Infinite` - when positive or negative Infinity is provided
- `ValidationIssue::InvalidNumeric` - for non-finite values in document validation

## Contract Signatures
```rust
impl OrderedFloat {
    pub fn new(value: f64) -> Result<Self, OrderedFloatError>;
    pub const fn new_unchecked(value: f64) -> Self; // for trusted deserialization
}

pub enum OrderedFloatError {
    NaN,
    Infinite,
}
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| value is finite | Runtime-checked constructor | `OrderedFloat::new() -> Result<Self, OrderedFloatError>` |
| serde accepts finite only | Custom deserialize | Custom `deserialize` that validates |
| schema catches remaining | Runtime validation | `validate_schema()` checks all fields |

## Violation Examples (REQUIRED)
- VIOLATES P1: `OrderedFloat::new(f64::NAN)` → `Err(OrderedFloatError::NaN)`
- VIOLATES P2: `OrderedFloat::new(f64::INFINITY)` → `Err(OrderedFloatError::Infinite)`
- VIOLATES P3: `OrderedFloat::new(f64::NEG_INFINITY)` → `Err(OrderedFloatError::Infinite)`
- VIOLATES Q2: `OrderedFloat::new(f64::NAN)` should NOT return Ok(OrderedFloat(NAN))
- VIOLATES Q3: `OrderedFloat::new(f64::INFINITY)` should NOT return Ok(OrderedFloat(INF))

## Ownership Contracts
- OrderedFloat is Copy for performance (wraps primitive f64)
- No mutation of external state in any method

## Non-goals
- [ ] Handle subnormal numbers (keep them valid)
- [ ] Add compile-time guarantees for all callers (too many call sites)
