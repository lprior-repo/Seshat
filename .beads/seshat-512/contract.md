# Contract Specification

## Context
- **Feature**: Stylus and Gesture Input Processing (INP-004 to INP-007)
- **Domain terms**: 
  - `PointerType`: Categorization of input source (`Mouse`, `Touch`, `Pen`).
  - `Action`: The domain action emitted by input reduction (e.g., `PanCamera`, `MoveShape`, `DoubleTap`).
  - `InputState`: The immutable tracking of active pointers and tap history.
  - `InputConfig`: Configuration for timings and hit testing radii.
- **Assumptions**: 
  - Input reduction follows the Data -> Calc -> Actions pattern. 
  - Raw pointer events are pure calculations that produce new states and a list of actions.
  - The UI framework accurately reports `PointerType` and `PointerId`.
- **Open questions**:
  - Should palm rejection be handled at the platform layer or within this diagram tool's input reducer? (Assuming platform for MVP).

## Preconditions
- [ ] P1: Gesture timing thresholds (`double_tap_timeout_ms`) must be strictly greater than zero.
- [ ] P2: Touch hit area padding (`touch_padding`) must be a non-negative value.
- [ ] P3: A two-finger pan calculation requires exactly two distinct, active `PointerId`s.
- [ ] P4: Pointer events cannot be processed if their `PointerId` is not tracked in the current active `InputState` (except for `PointerDown`).

## Postconditions
- [ ] Q1 (INP-004): Processing a two-finger move event MUST NOT emit a `MoveShape` action. It MUST exclusively emit a `PanCamera` action.
- [ ] Q2 (INP-005): Processing a `PointerType::Pen` event MUST NOT use the `touch_padding` for hit testing; it MUST use standard or high-precision radius.
- [ ] Q3 (INP-006): Sequential `PointerDown` events within the `double_tap_timeout_ms` and distance threshold MUST emit a `DoubleTap` action.
- [ ] Q4 (INP-007): Hit testing a handle with `PointerType::Touch` MUST use the expanded hit radius (`base_radius + touch_padding`).

## Invariants
- [ ] INV1: The touch input hit testing radius is ALWAYS greater than or equal to the mouse/pen hit testing radius.
- [ ] INV2: The `InputState` never tracks more than 10 simultaneous pointers (to prevent overflow/malicious input floods).

## Error Taxonomy
- `Error::InvalidTimingThreshold` - when a configuration timing is 0 or negative.
- `Error::NegativeHitPadding` - when touch hit padding is configured as < 0.
- `Error::UntrackedPointer` - when a `PointerMove` or `PointerUp` is received for an unknown `PointerId`.
- `Error::TooManyPointers` - when exceeding the maximum allowed simultaneous pointers.

## Contract Signatures
```rust
pub fn process_pointer_event(
    state: &InputState,
    event: &PointerEvent,
    config: &InputConfig
) -> Result<(InputState, Vec<Action>), Error>;

pub fn hit_test_handle(
    handle: &Handle,
    point: Point,
    pointer_type: PointerType,
    config: &InputConfig
) -> Result<bool, Error>;
```

## Type Encoding
For each precondition, specify the strongest possible type enforcement:

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: `double_tap_timeout_ms > 0` | Compile-time (strongest) | `NonZeroU64` for duration |
| P2: `touch_padding >= 0` | Compile-time | `u32` or `NonNegativeF32` (newtype) |
| P3: Two-finger pan requires 2 IDs | Runtime-checked constructor | `TwoFingerGesture::new(id1, id2) -> Result` |
| P4: Pointer must be tracked | Error variant | `Result<..., Error::UntrackedPointer>` |

## Violation Examples
- VIOLATES P1: `InputConfig::new(0, ...)` -- should fail to compile (if `NonZeroU64`) or produce `Err(Error::InvalidTimingThreshold)`.
- VIOLATES P2: `InputConfig::new(..., -5.0)` -- should fail to compile (if strictly typed) or produce `Err(Error::NegativeHitPadding)`.
- VIOLATES P3: `TwoFingerGesture::new(id1, id1)` (same IDs) -- should produce `Err(Error::DuplicatePointerId)`.
- VIOLATES P4: `process_pointer_event(state, PointerMove(id: 99), ...)` where 99 is not in state -- should produce `Err(Error::UntrackedPointer)`.
- VIOLATES Q1: `process_pointer_event` for two fingers returns `[Action::MoveShape(...)]` -- should produce `Err(Error::PostconditionViolation)` in tests.
- VIOLATES Q2: `hit_test_handle` with `Pen` uses `touch_padding` -- should produce `Err(Error::PostconditionViolation)` in tests.
- VIOLATES Q3: Two taps within 100ms return `[Action::SingleTap]` instead of `DoubleTap` -- should produce `Err(Error::PostconditionViolation)`.
- VIOLATES Q4: `hit_test_handle` with `Touch` fails to hit a point within `base + touch_padding` -- should produce `Err(Error::PostconditionViolation)`.

## Ownership Contracts (Rust-specific)
- Shared borrow: `fn process_pointer_event(state: &InputState, ...)` -- Calculates the next state immutably without altering the previous state. Returns an owned `(InputState, Vec<Action>)`.
- Shared borrow: `config: &InputConfig` -- Read-only reference to timings and properties.
- Clone policy: `InputState` will likely implement `Clone` to allow creating the modified next state easily, though immutable data structures or struct update syntax are preferred. `PointerEvent` and `Action` should be small enough to `Clone`.

## Non-goals
- [ ] Recognizing complex multi-finger gestures beyond simple two-finger pan/zoom (e.g., three-finger swipe).
- [ ] Advanced stylus tilt and barrel roll (we only care about basic `Pen` vs `Touch` differentiation and pressure if needed).
