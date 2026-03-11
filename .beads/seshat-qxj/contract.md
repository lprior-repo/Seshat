# Contract Specification: INP-001 to INP-003 (Touch Input Basics)

## Context
- **Feature**: Basic Touch Input parsing and handling for the Diagram Canvas (INP-001 to INP-003).
- **Domain terms**:
  - `RawEvent`: The incoming untyped event from the JS/Dioxus boundary.
  - `CanvasEvent`: The typed event enum representing canvas interactions.
  - `InteractionState`: The core state machine of the canvas.
- **Assumptions**:
  - Touch input does not have a "Hover" state. A touch down event must immediately transition from Idle to a dragging or selecting state.
  - Basic touch events will map to new event types: `"touch_down_target"`, `"touch_down_background"`, `"touch_move"`, `"touch_up"`.
  - For the MVP, touch interactions (tap, drag, release) map closely to mouse semantics but bypass hover logic.
- **Open questions**:
  - Do we need to track pointer IDs for multi-touch (e.g., pinch to zoom) now, or is single-point tracking sufficient for INP-001 to INP-003? (Assuming single-point tracking for these three basics).

## Preconditions
- [ ] P1: `RawEvent` representing a touch down or touch move must contain valid, finite `x` and `y` coordinates.
- [ ] P2: `RawEvent` representing a touch move (`touch_move`) must contain valid, finite `dx` and `dy` deltas if translated to vector movements.
- [ ] P3: `RawEvent::event_type` must be one of the recognized touch strings to be successfully parsed into a touch `CanvasEvent`.

## Postconditions
- [ ] Q1: `parse_event` maps `"touch_down_target"` to a `CanvasEvent::TouchDownTarget` (or equivalent touch wrapper) containing a valid `CanvasPoint`.
- [ ] Q2: `parse_event` maps `"touch_down_background"` to a `CanvasEvent::TouchDownBackground`.
- [ ] Q3: `parse_event` maps `"touch_move"` to a `CanvasEvent::TouchMove`.
- [ ] Q4: `parse_event` maps `"touch_up"` to a `CanvasEvent::TouchUp`.
- [ ] Q5: The interaction reducer immediately ignores a `TouchMove` event if the `InteractionState` is `Idle` (touch has no hover state).

## Invariants
- [ ] I1: `CanvasPoint` and `CanvasVector` parsed from touch events must never contain NaN or infinite values.

## Error Taxonomy
- `CanvasError::CoordinateOutOfBounds` - when `x`, `y`, `dx`, or `dy` on the incoming touch `RawEvent` are non-finite (NaN or Infinity).
- `CanvasError::UnparseableEvent` - when the `event_type` is an unknown touch string (e.g., `"touch_cancel"`, `"touch_pinch"` if unsupported).

## Contract Signatures
- `pub fn parse_event(raw: RawEvent) -> Result<CanvasEvent, CanvasError>`
- `pub fn reduce(state: InteractionState, event: CanvasEvent) -> Result<InteractionState, CanvasError>`

## Type Encoding
For each precondition, specify the strongest possible type enforcement:
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Finite x, y | Runtime-checked constructor | `CanvasPoint::new(x, y) -> Result<CanvasPoint, CanvasError>` |
| P2: Finite dx, dy | Runtime-checked constructor | `CanvasVector::new(dx, dy) -> Result<CanvasVector, CanvasError>` |
| P3: Valid event_type | Error variant | `Result<CanvasEvent, CanvasError::UnparseableEvent>` |

## Violation Examples
- VIOLATES P1: `parse_event(RawEvent { event_type: "touch_down_target".into(), x: f64::NAN, y: 0.0, ... })` -- should produce `Err(CanvasError::CoordinateOutOfBounds)`
- VIOLATES P2: `parse_event(RawEvent { event_type: "touch_move".into(), x: 0.0, y: 0.0, dx: f64::INFINITY, dy: 0.0, ... })` -- should produce `Err(CanvasError::CoordinateOutOfBounds)`
- VIOLATES P3: `parse_event(RawEvent { event_type: "touch_hover".into(), ... })` -- should produce `Err(CanvasError::UnparseableEvent)`
- VIOLATES Q5 (No Hover invariant): `reduce(InteractionState::Idle, CanvasEvent::TouchMove { .. })` -- should produce `Ok(InteractionState::Idle)` directly without entering a `Hovering` state.

## Ownership Contracts (Rust-specific)
- Ownership transfer: `parse_event(raw: RawEvent)` -- caller gives up ownership of the raw JS/Dioxus string-based event, returning an owned `CanvasEvent`.
- Shared borrow: No shared borrows involved directly in parsing.
- Exclusive borrow: `apply_drag_delta(drag: &mut DragState, delta: CanvasVector)` -- modifies `cumulative_offset` and `current` fields during touch dragging.