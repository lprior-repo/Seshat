# Architecture Spec: seshat-m4mv (Phase 4 DDD State Machines)

## Phase 1: EARS (Eliminate Requirements Ambiguity)
*   **Ubiquitous:** "The system shall enforce valid states at compile time using enums, completely eliminating the use of boolean flags (e.g., `active: bool`) for representing state."
*   **Event-Driven:** "When snapping evaluation completes, the system shall return an explicit `SnapResult::Snapped` or `SnapResult::Unsnapped` variant."
*   **State-Driven:** "While executing core domain operations (routing, grouping, transforming), the system shall evaluate all steps through a `Result`-based pipeline."
*   **Unwanted:** "If a domain operation encounters a missing entity or invalid mathematical state, the system shall NOT panic, `unwrap()`, or `expect()`. The system shall propagate a strongly-typed domain error."

## Phase 2: KIRK Contracts (Domain Modeling)
### `SnapResult` State Machine
The pure logic boundary for snapping MUST enforce that illegal states are unrepresentable.
*   **Precondition:** Snapping evaluation receives a valid point and context.
*   **Postcondition:** The result guarantees that `snapped_position` and `target_node_id` DO NOT EXIST in memory if the state is unsnapped.
*   **Invariant:** It is a compile-time impossibility to access a snapped coordinate on a failed snap.

**Type Contract:**
```rust
pub enum SnapResult {
    Snapped {
        snap_type: SnapType,
        target_node_id: NodeId,
        snapped_position: Point,
    },
    Unsnapped,
}
```

### Core Mutation & Routing Pipelines
*   **Preconditions:** Core operations (cut/copy/paste, grouping, routing, nudging, transformations) must explicitly state their required dependencies (e.g., `NodeId` must be present).
*   **Postconditions:** All state transitions either succeed entirely, or fail returning the exact failure mode without mutating the global state.
*   **Invariants:** Zero panics. `unwrap()` count in `src/core/` source files (excluding tests) MUST be exactly 0.

## Phase 3: Inversion (How do we guarantee failure?)
### The Exact Error Taxonomy
We must map every possible way a core pipeline or routing operation can fail, guaranteeing that no missing node or bad math crashes the WASM module.

```rust
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CoreError {
    #[error("Node not found in document: {0}")]
    NodeNotFound(NodeId),
    
    #[error("Edge not found in document: {0}")]
    EdgeNotFound(EdgeId),
    
    #[error("Invalid transformation math (e.g., NaN, Infinity): {0}")]
    InvalidTransformation(String),
    
    #[error("Cyclic dependency detected; operation would create infinite loop for node: {0}")]
    CyclicDependency(NodeId),
    
    #[error("Operation requires selection, but selection is empty")]
    EmptySelection,
    
    #[error("Clipboard operation failed: {0}")]
    ClipboardError(String),
    
    #[error("Snap domain failure")]
    SnapError(#[from] crate::geometry::snap::SnapError),
    
    #[error("Routing domain failure: {0}")]
    RoutingError(String),
}
```
*Failure Modes Handled:*
*   **Node deleted concurrently?** `CoreError::NodeNotFound`
*   **Group cycle?** `CoreError::CyclicDependency`
*   **Divide by zero in routing?** `CoreError::InvalidTransformation`

## Phase 4: Second-Order Consequence Tracing
*   **Blast Radius 1:** Changing `SnapResult` to an enum forces every single UI interaction and geometry module that checks `if result.active` to use pattern matching (`if let SnapResult::Snapped { .. } = result`). This is highly desirable as it removes dummy data logic (`Point::new(0.0, 0.0)`).
*   **Blast Radius 2:** Removing `unwrap()` in pipelines means functions like `align_selection`, `group_selection`, and `compute_straight_line_route` must return `Result<(), CoreError>`.
*   **Blast Radius 3:** The UI boundary (e.g., event handlers mapping interactions to core mutations) must sink these `Result`s. If a `Result::Err` bubbles up to the UI, what happens? Does it crash? No. It must be logged or presented to the user, meaning the top-level WASM/Dioxus action handler needs an `if let Err(e) = action() { log::warn!(...) }` catch-all.

## Phase 5: Pre-Mortem (The 3 AM Red Build)
**Disaster Scenario:**
It is 3 months from now. A user selects 50 nodes and presses "Group". Nothing happens. The UI is completely unresponsive to that action, but the app hasn't crashed.

**Why did it happen?**
We successfully removed `unwrap()` and replaced it with `CoreError::NodeNotFound`. However, the UI event handler silently consumed the `Result` by using `.ok()` or discarding the return value entirely. The error occurred because one of the 50 nodes was a stale ID, but the user got zero feedback.

**Mitigation (The Fix Required Now):**
1. Ensure the UI layer NEVER silently drops `CoreError`. It must use `tracing::error!()` at minimum.
2. The core pipeline must be transactional: if grouping fails on node 49 of 50, nodes 1-48 MUST NOT be left in a partially grouped state. The pipeline must validate ALL nodes exist *before* applying any mutations.
