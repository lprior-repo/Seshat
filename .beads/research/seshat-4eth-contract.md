# Architectural Specification: seshat-4eth Core Phase 3 Pure Reducers

## 1. EARS Requirements (Eliminate Requirements Ambiguity)
**Ubiquitous Language:**
- `CanvasEvent`: A strongly-typed representation of user intent (e.g., `NodeSelected`, `EdgeDrawn`, `PanStarted`).
- `Reducer`: A pure, deterministic function `apply_event(state: DiagramDocument, event: CanvasEvent) -> Result<DiagramDocument, CanvasError>`.
- `UI Handler`: A Dioxus DOM event listener that ONLY constructs `CanvasEvent`s and dispatches them to the `Reducer`.

**Event-Driven (Triggers):**
- WHEN a user clicks a node (Primary Mouse Button), THE SYSTEM SHALL dispatch a `NodeSelected(NodeId, Additive)` event.
- WHEN a user clicks a node in Edge Mode, THE SYSTEM SHALL dispatch an `EdgeDrawingStarted(NodeId, CanvasCoord)` event.
- WHEN a user releases the mouse while drawing an edge over a target node, THE SYSTEM SHALL dispatch an `EdgeDrawingFinished(SourceNodeId, TargetNodeId)` event.
- WHEN a user pans the canvas, THE SYSTEM SHALL dispatch a `PanStarted` or `PanUpdated` event.

**State-Driven:**
- WHILE in `InteractionMode::DrawingEdge`, THE SYSTEM SHALL validate DAG constraints upon receiving `EdgeDrawingFinished`.
- WHILE in `ToolMode::Pan`, THE SYSTEM SHALL NOT dispatch selection events.

**Unwanted Behavior:**
- IF a Dioxus event handler fires, THE SYSTEM SHALL NOT mutate `Signal<DiagramDocument>`, `Signal<History>`, or `Signal<InteractionMode>` directly. All mutations MUST pass through the pure reducer.

## 2. KIRK Contracts (Domain Modeling)

**Data -> Calc -> Actions boundary:**
- **Data:** `DiagramDocument`, `InteractionMode`, `ToolMode`, `History`.
- **Calc:** The pure reducer `apply_event` function.
- **Actions:** The UI Layer setting the new state into Dioxus signals.

**Preconditions (Type-Level Enforced):**
- `CanvasEvent` must encapsulate all required data (e.g., world coordinates, not DOM client coordinates). Coordinate transformation must occur in the UI layer (Actions) before calling the pure reducer (Calc).

**Postconditions:**
- `apply_event` returns `Ok(DiagramDocument)` indicating a successful state transition.
- `apply_event` returns `Err(CanvasError)` indicating a rejected transition (e.g., circular DAG connection).

**Invariants:**
- `DiagramDocument` remains a valid DAG at all times.
- `apply_event` must be 100% deterministic and isolated from the DOM, `window`, or Dioxus `Event<T>`.
- The UI layer contains zero business logic.

## 3. Inversion (Error Taxonomy)

**Exhaustive Error Taxonomy (`CanvasError` enum):**
- `CanvasError::CircularConnectionRejected`: Thrown when an edge connection violates the DAG invariant.
- `CanvasError::InvalidStateTransition(InteractionMode, CanvasEvent)`: Thrown when an event is dispatched in an incompatible state (e.g., `EdgeDrawingFinished` while in `InteractionMode::Panning`).
- `CanvasError::NodeNotFound(NodeId)`: Thrown when an operation references a node that no longer exists in the document.

**Failure Guarantee:**
- If the reducer returns an error, the `DiagramDocument` state remains UNCHANGED. The UI layer may choose to display a toast (e.g., "Cannot create circular connection"), but the core data is protected.

## 4. Second-Order Consequence Tracing

- **Blast Radius of Pure Reducers:** By moving logic out of `handlers.rs`, we enable property-based testing and "fuzzing" of `DiagramDocument` states by sending random streams of `CanvasEvent`s.
- **Concurrent Throughput:** Since `apply_event` is a pure function taking a state and returning a new state, it allows future implementation of conflict resolution (e.g., OT or CRDT) by rebasing a sequence of `CanvasEvent`s on top of new remote states.
- **Performance:** Cloning the entire `DiagramDocument` on every mouse move might be expensive. The system currently uses `im::HashMap` which provides structural sharing, so cloning is cheap. The reducer must continue utilizing `im::HashMap`'s structural sharing to prevent O(N) allocations per frame.

## 5. Pre-Mortem (The 3 AM Red Build)

**The Disaster:** It is 3 months from now. Production is broken because users report that drawing edges sometimes "freezes" the application or connects to the wrong nodes when the user scrolls during an edge draw.

**The Cause:** The UI handler passed raw DOM Client Coordinates into the `CanvasEvent` instead of transforming them into World Coordinates using the current camera position before dispatching. The pure reducer calculated the intersection using stale camera data.

**The Fix / Telemetry:**
- Ensure `CanvasEvent` strictly requires strongly-typed `CanvasCoord` (World Space) and NEVER `ScreenCoord` (Client Space).
- The coordinate transformation `to_canvas_coords` must happen in the UI handler, passing only pure domain types to the reducer.
