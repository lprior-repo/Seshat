# Output Formatting Rules

## 🔴 PHASE 1: Contract Violations
- `canvas_event.rs:27`: `parse_event` unconditionally parses `raw.x` and `raw.y` into `CanvasPoint::new` BEFORE matching the event type. This causes valid `drag_move` and `mouse_up` events with meaningless or invalid coordinates to fail with `CoordinateOutOfBounds` instead of mapping successfully, violating the preconditions of `RawEvent` boundary parsing.

## 🟠 PHASE 2: Farley Rigor Flaws
- `transition.rs:28-107`: HARD REJECT. The `transition` function is 80 lines long, flagrantly violating the strict <25 lines rule. Break it up into smaller discrete state-handler functions.

## 🟡 PHASE 3: Functional Rust Flaws (The Big 6)
- `interaction_state.rs:31-33`: "Parse, Don't Validate" violation. `apply_drag_delta` manually asserts `!delta.dx.is_finite()` even though the type boundary (`CanvasVector`) already strictly guarantees finite coordinates on construction. Stop validating trusted types!
- `types.rs:7-8`: Stringly-typed domain errors. `CanvasError::InvalidTransition { state: String, event: String }` forces strings into a pure domain error type instead of preserving the explicit discriminant enum or `&'static str`, muddying the domain representation.

## 🔵 PHASE 4: Simplicity & DDD Failures
- `transition.rs:90-94`: Sloppy modeling. `InteractionState::Dragging { drag }` combined with `CanvasEvent::MouseMove` silently drops the point and returns the unchanged state, papering over an unhandled transition case with a weak comment instead of explicitly resolving the domain logic.

## 🟣 PHASE 5: The Bitter Truth (Cleverness & Bloat)
- `transition.rs:32-40`: The state machine is allocating. The pure UI `transition` loop unconditionally heap-allocates `.to_string()` for both `state_str` and `event_str` on **every single event** before the match block even begins. This is junior-level "cleverness" to make error formatting "easy", completely ruining the fast path. Use `&'static str` lazily inside the error closure.

## Verdict
This code attempts to look functional but hides stringly-typed errors, unnecessary boundary validations, and an 80-line monolithic transition function that pointlessly heap-allocates on every mouse movement. REJECT the code and rewrite immediately.