# Contract Specification

## Context
- Feature: Refactor Canvas UI layer (interaction state machine, drag state, handlers, rendering, and selection logic) to enforce strict types and DDD principles.
- Domain terms: `InteractionState`, `CanvasEvent`, `DragState`, `SelectionMode`, `CanvasPoint`, `CanvasVector`.
- Assumptions: Refactoring will remove implicit boolean states and `Option<DragState>` from the canvas. The canvas acts as a functional state machine taking parsed inputs.
- Constraints: Strict module boundaries must be maintained. NO single source or test file may exceed **300 lines of code**.
- Open questions: None.

## Preconditions
- [P1] All incoming raw UI inputs must be parsed into constrained `CanvasEvent` variants at the boundaries before entering the core state machine.
- [P2] Dragging must start from a valid semantic `CanvasPoint`, not raw f64 coordinates.
- [P3] Selection bounds must be valid and explicitly modeled, preventing negative or zero-area bounding boxes implicitly.
- [P4] The `SelectionMode` must be explicitly passed as a strongly-typed enum, not derived from boolean flags (e.g., `is_selecting` vs `is_additive`).

## Postconditions
- [Q1] Any `InteractionState` transition strictly consumes the old state (by value) to prevent illegal overlapping state representations.
- [Q2] When an `&mut DragState` is updated via a valid `CanvasVector` delta, the cumulative offset must correctly reflect the change.
- [Q3] Every interaction event must map explicitly to a transition result: `Result<InteractionState, CanvasError>`, ensuring exhaustiveness.

## Invariants
- [I1] The interaction state machine is always in exactly one valid state (e.g., `Idle`, `Hovering`, `Dragging`, `Selecting`). No concurrent/overlapping boolean state flags.
- [I2] Coordinates in the domain core are always represented by semantic types (`CanvasPoint`, `CanvasVector`), never primitive `f64`/`f32`.
- [I3] Boolean control flags are completely banned in domain signatures.
- [I4] Property Invariant: Any sequence of valid `CanvasEvent`s folded into the `transition` state machine must never result in a panic or an unrepresentable state.

## Error Taxonomy
- `CanvasError::UnparseableEvent` - when raw UI inputs cannot be interpreted into a `CanvasEvent`.
- `CanvasError::InvalidTransition { state: StateDiscriminant, event: EventDiscriminant }` - when a parsed event cannot be processed in the current interaction state.
- `CanvasError::CoordinateOutOfBounds` - when numerical operations result in invalid geometry (e.g., NaN, infinite, or exceeding boundaries).
- `CanvasError::InvalidSelectionBounds` - when an invalid selection area is attempted (e.g., non-positive dimensions).

## Contract Signatures
- `fn parse_event(raw: RawEvent) -> Result<CanvasEvent, CanvasError>`
- `fn transition(state: InteractionState, event: CanvasEvent) -> Result<InteractionState, CanvasError>`
- `fn apply_drag_delta(drag: &mut DragState, delta: CanvasVector) -> Result<(), CanvasError>`

## Type Encoding
For each precondition, specify the strongest possible type enforcement:
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| [P1] Parsed boundaries | Compile-time | `CanvasEvent` enum |
| [P2] Semantic coordinates | Compile-time | `CanvasPoint` newtype |
| [P3] Valid selection bounds | Runtime-checked constructor | `SelectionBounds::new() -> Result<Self, CanvasError>` |
| [P4] Explicit selection mode | Compile-time | `SelectionMode` enum (no booleans) |

## Violation Examples (REQUIRED)
- VIOLATES [P1]: `parse_event(RawEvent { type: "unknown_click", x: 0.0, y: 0.0 })` -- should produce `Err(CanvasError::UnparseableEvent)`
- VIOLATES [P2]: `CanvasPoint::new(f64::NAN, f64::INFINITY)` -- should produce `Err(CanvasError::CoordinateOutOfBounds)`
- VIOLATES [P3]: `SelectionBounds::new(CanvasPoint(10, 10), CanvasPoint(5, 5))` -- should produce `Err(CanvasError::InvalidSelectionBounds)`
- VIOLATES [P4]: `transition(state, CanvasEvent::Select { additive: true })` -- compile-time violation, MUST use `SelectionMode::Additive` instead of a bool.
- VIOLATES [Q1]: `transition(&mut state, CanvasEvent::Click)` -- compile-time violation, `transition` MUST consume `InteractionState` by value.
- VIOLATES [Q2]: `apply_drag_delta(&mut drag, CanvasVector::new(f64::NAN, 0.0))` -- should produce `Err(CanvasError::CoordinateOutOfBounds)`
- VIOLATES [Q3]: `transition(InteractionState::Idle, CanvasEvent::DragMove(delta))` -- should produce `Err(CanvasError::InvalidTransition { state: Idle, event: DragMove })`

## Ownership Contracts (Rust-specific)
- Ownership transfer: `fn transition(state: InteractionState, event: CanvasEvent) -> Result<InteractionState, CanvasError>` takes ownership of the previous state to guarantee that invalid or intermediate states cannot be reused.
- Exclusive borrow: `fn apply_drag_delta(drag: &mut DragState, delta: CanvasVector)` mutates the current cumulative offsets securely.
- Clone policy: `InteractionState` is expected to be relatively lightweight. UI components may clone the current state for rendering (`Overlays`, `View`), but mutations always happen via the consumed transition pipeline.