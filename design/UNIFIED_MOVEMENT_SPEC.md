# Seshat: Unified Movement Specification

## Goal
Achieve 120 FPS high-fidelity movement for selections of 500+ nodes, mimicking the exact Z-index hoisting and GPU offloading techniques used by Figma and Miro.

## Architecture Blueprint

### 1. The Transient Offset (`canvas_domain`)
The `DiagramDocument` represents the durable, truth-of-record state of the canvas. Updating it via `im::HashMap` is too expensive for an 8ms frame budget on 500+ nodes.
- `InteractionMode::DraggingSelection` will be expanded to hold a `current_offset: (f64, f64)` in Canvas Coordinates.
- During a drag (via `pointer_drag.rs`), we calculate the grid-snapped `dx` and `dy` from the origin anchor and update *only* the `InteractionMode` signal.
- The `DiagramDocument` nodes remain at their exact original `(x, y)` coordinates until `mouseup`.

### 2. The Ghost Layer (Z-Index Hoisting)
When the Dioxus UI renders `NodeLayer`, we will partition nodes into two DOM structures:
- **Static Nodes**: Rendered first. They do not react to the drag offset.
- **Ghost Layer**: We will render a `<div class="ghost-layer">` *after* the static nodes. This implicitly hoists the dragging nodes to the top of the canvas Z-order (the exact Figma/Miro visual behavior).
- **GPU Translation**: The `ghost-layer` div receives a hardware-accelerated transform: `transform: translate3d({dx * zoom}px, {dy * zoom}px, 0)`. The `NodeElement` components inside it are rendered using their original CSS `left`/`top` values, avoiding browser layout recalculations.

### 3. Elastic Edges (CPU Preservation)
Updating complex bezier curve calculations for 500 edges every 8ms will drop frames.
- `EdgeLayer` will read the `current_offset` from the `interaction_mode` signal.
- If **both** nodes of an edge are in the Ghost Layer, the edge is shifted by `dx, dy`.
- If **one** node is in the Ghost Layer and the other is static (a "Boundary Edge"), we bypass the `edge_path()` curve algorithm. Instead, we render a straight `<line>` "Elastic Preview" from the static pin to the moving pin.

### 4. The Commit Phase
When `mouseup` fires, the visual illusion collapses into reality:
- `finalize_motion_release` extracts the final `current_offset` and performs a single, batch `im::HashMap` mutation to apply `(nx, ny)` to all selected nodes.
- A batch of `DomainOp::NodeMove` events is dispatched to the backend `db_tx` log.
- `InteractionMode` reverts to `Select`, destroying the Ghost Layer, dropping the nodes back into their proper static Z-index, and restoring normal curved edges.

## Edge Cases & Failure Modes

1. **Locked Nodes in Selection**
   - *Failure:* A locked node gets hoisted into the Ghost Layer and visually moves, but snaps back on release.
   - *Solution:* The partition logic in `NodeLayer` must explicitly verify `node.lock_state.is_movable()`. Locked nodes remain in the Static Layer.
2. **Escape Key Cancellation**
   - *Failure:* Pressing `Escape` leaves nodes at the offset or increments document revision unnecessarily.
   - *Solution:* Because `doc_signal` is never mutated during the drag, catching the `Escape` key and setting `InteractionMode = Select` instantly destroys the Ghost Layer. Nodes instantly snap back to their original coordinates. No CRDT operations are logged.
3. **Subgraphs and Deep Hierarchies**
   - *Failure:* Dragging a subgraph leaves its children behind, or applies double-translation.
   - *Solution:* `drag_original_positions` already recursively fetches children of selected subgraphs. All children are placed into the Ghost Layer alongside the parent. Because their relative `left/top` CSS coordinates are absolute in Seshat, they all shift cleanly together via the parent container's `translate3d`.
4. **Panning While Dragging**
   - *Failure:* The user drags nodes, then presses Spacebar to pan the camera, causing the Ghost Layer to visually drift or disconnect from the mouse.
   - *Solution:* The transient `dx, dy` is calculated using Canvas Coordinates. If the camera pans, the internal `NodeElement` recalculates its base `left/top` relative to the new camera, but the Ghost Layer's `translate3d(dx*zoom, dy*zoom, 0)` remains mathematically accurate. The visual position remains locked to the mouse.

## Testing Strategy

1. **Domain Logic (Proptests & Unit Tests)**
   - Update `interaction_reducer/tests/basic_tests.rs`.
   - Assert `finalize_motion_release` correctly applies the `current_offset` tuple to the `original_positions` map and triggers the revision increment exactly once.
   - Assert that if `current_offset` is `(0.0, 0.0)`, no mutation occurs.
2. **E2E Playwright DOM Validation**
   - Create a specific Playwright test for Unified Movement.
   - Select 3 nodes and trigger a `mousemove`.
   - Assert that `<div class="ghost-layer">` appears in the DOM and is the last sibling (Z-index hoisted).
   - Assert `translate3d` updates dynamically on the container, but the `style="left: X; top: Y"` properties on the `<div data-testid="node">` elements inside it *do not change*.
   - Trigger `mouseup`, assert the Ghost Layer is removed, and the `left/top` CSS styles are permanently updated.