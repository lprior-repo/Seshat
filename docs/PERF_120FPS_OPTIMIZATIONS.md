# Drag Rendering Performance — 120fps Optimization Targets

## Context

- **Current**: ~16.6ms/frame (60fps) during node drag on 50-node diagram
- **Target**: <8.33ms/frame (120fps)
- **Frame budget to reclaim**: ~8.3ms minimum
- **Control**: `125393f` (pre-opt) | **Treatment**: `0efe302` (post-opt)
- **Benchmark**: `diagram_tool/e2e/drag_frame_perf.spec.ts`
- **Run**: `npx playwright test --project=perf-latency --grep "drag-frame" --reporter=list`

### Empirical baseline (N=10 runs each)

| Metric | Treatment (0efe302) | Control (125393f) |
|--------|---------------------|-------------------|
| DOM mutations/trial | 42 (σ=0) | 127 (σ=0) |
| Avg step time | 16.63ms (σ=0.022) | 16.76ms (σ=0.329) |
| Jank frames/250 | 33.9 | 34.1 |

---

## Dioxus Reactivity Ground Truth

Before touching any code, understand how Dioxus 0.7 Memo actually works (verified from `dioxus-core-0.7.3/src/reactivity/memo.rs:130`):

```rust
if new_value != *peak {  // PartialEq guard
    // ONLY propagates to subscribers if output changed
    self.inner.set(new_value);
}
```

**Key insight**: When `doc_signal.with_mut()` updates node positions during drag, `node_viewport_trigger` memo closure DOES re-run (wasted CPU), but it does NOT propagate to subscribers because the output tuple `(cam_x, cam_y, zoom, selected_items)` is identical during drag. This means:

- NodeLayer, EdgeLayer, GridLayer do NOT re-render via the Memo path during drag
- The memo closure cost (`doc_signal.read()` + tuple construction + PartialEq) is wasted but small
- **The real re-render trigger during drag is something else** — likely `editor_state` signal changes, or the nodes update via a different mechanism entirely

**Rejected findings** from prior audit (claimed impact but disproven by Dioxus source):
- ~~Memo false-firing causes unnecessary re-renders~~ — PartialEq guard prevents propagation
- ~~NodeLayer clones all visible nodes every frame~~ — NodeLayer may not re-render during drag at all
- ~~EdgeLayer/GridLayer false re-renders~~ — Same Memo guard applies

---

## Validated Optimizations

### OPT-1: `left/top` → `transform: translate()` for node positioning

**Impact**: HIGH (est. 2-4ms/frame)  
**File**: `diagram_tool/src/ui/canvas/node_layer/node_element.rs:127`  
**Status**: ✅ Verified in code

**Problem**: Nodes are positioned with `left: {x}px; top: {y}px`. Changing `left`/`top` triggers the browser's full **layout pipeline** — the browser must recalculate geometry of the element AND potentially its siblings. For 50 nodes moving every frame, this is 50 layout recalculations per frame.

`will-change: transform` is already set on the element but is **completely wasted** because we're not using `transform`.

**Fix**:
```rust
// BEFORE (node_element.rs:127):
style: "left: {left}px; top: {top}px; width: {width}px; height: {height}px; z-index: {z_index}; contain: layout style; will-change: transform;",

// AFTER:
style: "transform: translate({left}px, {top}px); width: {width}px; height: {height}px; z-index: {z_index}; contain: layout style paint; will-change: transform;",
```

**Complications**:
- `to_screen_coords` output is already `(left, top)` screen coordinates — no logic change needed there
- Event handlers in `node_element.rs` (mousedown, mouseenter, mouseleave) use `evt.client_x()` / `evt.client_y()` which are viewport-relative — unaffected by `transform` vs `left/top`
- `connection_dots.rs` and `node_content.rs` may have positioning that assumes `left/top` — check all `absolute` positioned children
- The label at `node_content.rs:157` uses `bottom: -18px` which positions relative to the parent — this works the same with `transform` since the parent still has `position: absolute`
- **Critical**: Dioxus sends `left`/`top` to the DOM. With `transform`, the element's `offsetLeft`/`offsetTop` will be 0. Any JS code reading these (including Playwright's `boundingBox()`) will still work because `getBoundingClientRect()` accounts for transforms. But if any Rust code reads DOM geometry back, it needs to account for this.

**Verification**: After fix, run benchmark and check step time drops. Also verify all E2E tests pass (especially `node_drag.debug.spec.ts` which asserts position changes).

---

### OPT-2: `opt-level = 'z'` → `opt-level = 3` for WASM release

**Impact**: HIGH (est. 1-3ms/frame across all WASM computation)  
**File**: `Cargo.toml:6` (workspace root)  
**Status**: ✅ Verified in code

**Problem**: `opt-level = 'z'` tells LLVM to optimize for **minimum binary size**, not maximum speed. This can be 10-30% slower than `opt-level = 3` for hot paths. For a 120fps target this is leaving performance on the table.

The current config:
```toml
[profile.release]
opt-level = 'z'
lto = true
strip = true
codegen-units = 1
panic = 'abort'
```

**Fix**: Create a diagram_tool-specific release profile so the native CLI can stay small:
```toml
# In diagram_tool/Cargo.toml — add at the bottom:

[profile.release]
opt-level = 3
lto = true
strip = true
codegen-units = 1
panic = 'abort'
```

Or keep workspace profile but change `opt-level`:
```toml
# In Cargo.toml (workspace root):
[profile.release]
opt-level = 3    # was 'z'
lto = true
strip = true
codegen-units = 1
panic = 'abort'
```

**Tradeoff**: WASM binary size will increase (typically 10-20% larger). For a diagram tool that's loaded once, this is acceptable.

**Verification**: Compare `cargo build --release -p diagram_tool --target wasm32-unknown-unknown` binary size before/after. Run benchmark.

---

### OPT-3: Eliminate full document clone in `handle_dragging`

**Impact**: MEDIUM (est. 0.5-1ms/frame)  
**File**: `diagram_tool/src/ui/canvas/document_ops/mutations/pointer_drag.rs:20`  
**Status**: ✅ Verified in code

**Problem**: Line 20 clones the entire `DiagramDocument` just to read `camera_x`, `camera_y`, `zoom`, `snap_to_grid`, and `grid_size`. Then line 35 clones it AGAIN for history push. The clone at line 20 is purely for reading — the data is used in `to_canvas_coords()` and `dragged_positions_with_snap()`.

```rust
// CURRENT (line 20):
let doc = doc_signal.read().clone();  // clones EVERYTHING

// Used for:
// - doc.editor_state.camera_x.0 (line 23)
// - doc.editor_state.camera_y.0 (line 23)
// - doc.editor_state.zoom.0 (line 24)
// - doc.editor_state.snap_to_grid (line 45)
// - doc.editor_state.grid_size (line 46)
// - doc.document.nodes.get(id) for has_movable_nodes check (lines 28-31)
// - doc.document.nodes.get(id) for has_changes check (lines 48-53)
// - doc.clone() for history push (line 36)
```

**Fix**: Read only the needed fields, and defer the full clone to the history push path:
```rust
// Replace line 20 with targeted reads:
let (cam_x, cam_y, zoom, snap_to_grid, grid_size) = doc_signal.with(|doc| {
    let es = &doc.editor_state;
    (es.camera_x.0, es.camera_y.0, es.zoom.0, es.snap_to_grid, es.grid_size)
});

let current_pos = to_canvas_coords(
    canvas_domain::ScreenCoord(client_x, client_y),
    canvas_domain::CanvasCoord(cam_x, cam_y),
    zoom,
);

// For has_movable_nodes (lines 27-32), peek instead of clone:
let has_movable_nodes = original_positions.keys().any(|id| {
    doc_signal.peek().document.nodes.get(id)
        .is_some_and(|node| node.lock_state.is_movable(&node.kind))
});

// For history push (line 35-36), clone only when needed:
if !*did_move && has_movable_nodes && has_drag_threshold(*anchor_client, (client_x, client_y)) {
    let history = history_signal.read().clone();
    let doc_for_history = doc_signal.read().clone();  // clone only here
    *history_signal.write() = history.push(doc_for_history);
    *did_move = true;
}

// For has_changes check (lines 48-53), peek:
let has_changes = positions.iter().any(|(id, (nx, ny))| {
    doc_signal.peek().document.nodes.get(id).is_some_and(|node| {
        node.lock_state.is_movable(&node.kind)
            && ((node.x.0 - *nx).abs() > f64::EPSILON
                || (node.y.0 - *ny).abs() > f64::EPSILON)
    })
});
```

**Complications**: `doc_signal.with()` borrows the signal immutably and drops the borrow before returning. `doc_signal.peek()` also borrows. Both are safe here. The `to_canvas_coords` function takes `CanvasCoord` and `ScreenCoord` — no changes needed there.

**Verification**: `cargo clippy -p diagram_tool -- -D warnings`. Run benchmark.

---

### OPT-4: Remove `box-shadow` during drag

**Impact**: MEDIUM-HIGH (est. 1-3ms/frame)  
**File**: `diagram_tool/assets/tailwind.css:1377-1403`  
**Status**: ✅ Verified in code

**Problem**: Every node has `box-shadow: 0 6px 18px oklch(0 0 0 / 0.24)`. When node geometry changes (via `left`/`top` or `transform`), the browser must re-render the shadow. For 50 visible nodes, that's 50 shadow Gaussian blurs per frame. Shadow rendering forces the element out of the GPU compositing fast-path.

All 4 node classes have the same shadow:
```css
.diagram-node { box-shadow: 0 6px 18px oklch(0 0 0 / 0.24); }
.diagram-node-hovered { box-shadow: 0 6px 18px oklch(0 0 0 / 0.24); }
.diagram-node-selected { box-shadow: 0 6px 18px oklch(0 0 0 / 0.24); }
.diagram-node-selected-hovered { box-shadow: 0 6px 18px oklch(0 0 0 / 0.24); }
```

**Fix Option A** (simplest — remove shadows during drag via CSS class):
```css
/* Add to tailwind.css */
.diagram-canvas--dragging .diagram-node,
.diagram-canvas--dragging .diagram-node-hovered,
.diagram-canvas--dragging .diagram-node-selected,
.diagram-canvas--dragging .diagram-node-selected-hovered {
    box-shadow: none;
}
```

Then in `root_container/mod.rs`, add the `diagram-canvas--dragging` class when `InteractionMode::DraggingSelection`:
```rust
// In RootContainer's canvas div:
class: if *interaction_mode.read() == InteractionMode::DraggingSelection {
    "diagram-canvas--dragging"
} else {
    ""
}
```

**Fix Option B** (more aggressive — remove shadows always):
```css
.diagram-node,
.diagram-node-hovered,
.diagram-node-selected,
.diagram-node-selected-hovered {
    /* box-shadow removed */
}
```

**Complications**: Option A requires knowing the interaction mode in the RootContainer render path, which it already has. Option B changes the visual design. Users may prefer shadows for depth perception.

**Verification**: Visual check that shadows disappear during drag and reappear after. Run benchmark.

---

### OPT-5: Hoist UUID and Date::now() out of per-node loop

**Impact**: MEDIUM when `db_tx` is Some, ZERO when None  
**File**: `diagram_tool/src/ui/canvas/document_ops/mutations/pointer_drag.rs:57-106`  
**Status**: ✅ Verified in code

**Problem**: Inside the `with_mut` loop (lines 58-105), each node iteration:
- Line 77: `uuid::Uuid::new_v4()` — crypto RNG via WASM→JS bridge
- Lines 88-101: `js_sys::Date::now()` — WASM→JS bridge call

Both are inside `if let Some(tx) = db_tx` but still waste bridge crossings.

**Fix**:
```rust
if has_changes {
    // Compute once before the loop
    let timestamp = {
        #[cfg(target_arch = "wasm32")]
        { js_sys::Date::now() as i64 }
        #[cfg(not(target_arch = "wasm32"))]
        { std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64 }
    };

    doc_signal.with_mut(|doc_mut| {
        for (id, (nx, ny)) in positions.iter() {
            let should_update = /* ... same as before ... */;
            if should_update {
                doc_mut.document.nodes = doc_mut.document.nodes.alter(
                    |n| { n.map(|node| Node { x: OrderedFloat(*nx), y: OrderedFloat(*ny), ..node }) },
                    id.clone(),
                );
                if let Some(tx) = db_tx {
                    tx.send(diagram_models::envelope::EventEnvelope {
                        op_id: uuid::Uuid::new_v4().to_string(),  // still per-node for uniqueness
                        // ... rest same ...
                        timestamp,  // reuse pre-computed timestamp
                    });
                }
            }
        }
    });
}
```

**Note**: UUID stays per-node (each event needs a unique ID). Only timestamp is hoisted. If you want to eliminate UUID bridge crossings too, consider `fastrand::u64()` or `nanoid::nanoid!()` which don't cross the WASM bridge.

**Verification**: `cargo clippy`. Run benchmark with db_tx connected.

---

### OPT-6: `contain: layout style` → `contain: layout style paint`

**Impact**: LOW-MEDIUM  
**File**: `diagram_tool/src/ui/canvas/node_layer/node_element.rs:127`  
**Status**: ✅ Verified in code, but blocked by label positioning

**Problem**: `contain: layout style` isolates layout and style recalculation but NOT paint. Adding `paint` would prevent shadow/color repaints from affecting siblings. But `node_content.rs:157` positions the label at `bottom: -18px` (outside the node bounds), which would be clipped by `paint` containment.

**Fix** (requires label position change first):
1. Move label inside node bounds (e.g., `bottom: 2px` instead of `bottom: -18px`)
2. Increase default node height by ~18px to accommodate
3. Then change `contain: layout style` → `contain: layout style paint`

**Blocked by**: Label redesign. Not a quick win. Skip for now unless label positioning is already being changed.

---

### OPT-7: Eliminate `color-mix()` in `.diagram-node-hovered` border

**Impact**: LOW (only fires on hover, not during drag)  
**File**: `diagram_tool/assets/tailwind.css:1390`  
**Status**: ✅ Verified in code

**Problem**: `.diagram-node-hovered` uses `color-mix(in oklch, var(--accent) 50%, transparent)` for its border. During drag, nodes are selected (not hovered), so this rarely fires. But if you hover a non-selected node during drag, it triggers.

**Fix**: Pre-compute the 50% mix in the Rust theme module and expose it as a CSS variable:
```css
.diagram-node-hovered {
    border: 1px solid var(--accent-50);  /* pre-computed */
}
```

```rust
// In theme/tokens.rs or wherever CSS vars are set:
// Compute oklch accent at 50% opacity and set --accent-50
```

**Verification**: Minimal. Low priority.

---

### OPT-8: `ordered_node_cache` Memo closure optimization

**Impact**: LOW  
**File**: `diagram_tool/src/ui/canvas/state/canvas_state.rs:81-84`  
**Status**: ✅ Verified in code

**Problem**: The memo reads `doc_signal.read()` and calls `ordered_node_ids(&doc)` (which sorts all node IDs) every time doc changes. During drag, doc changes every frame but node IDs don't change. The Memo's PartialEq guard prevents downstream propagation (the sorted Vec is identical), but the closure still wastes CPU on sorting.

**Fix**: This is tricky because there's no separate "node list changed" signal. Options:
1. Accept the wasted sort (it's O(n log n) which is fast for n<1000)
2. Create a separate `node_ids_signal` that only updates when nodes are added/removed
3. Cache the memo value and skip recompute if doc revision hasn't changed

**Verification**: Minimal. Low priority. The sort is fast for realistic node counts.

---

## Implementation Order (Recommended)

Based on impact × effort:

| Order | ID | Impact | Effort | Risk |
|-------|----|--------|--------|------|
| 1 | OPT-2 | HIGH | 5 min | Low (config change, binary size tradeoff) |
| 2 | OPT-3 | MEDIUM | 30 min | Low (targeted field reads) |
| 3 | OPT-5 | MEDIUM | 15 min | Low (hoist timestamp, optional UUID) |
| 4 | OPT-1 | HIGH | 2-4 hrs | **HIGH** (transform vs left/top is a deep change — event coords, bounding boxes, connection dots, edge endpoints all potentially affected) |
| 5 | OPT-4 | MEDIUM-HIGH | 30 min | Low (CSS class toggle) |
| 6 | OPT-7 | LOW | 15 min | Low (pre-compute CSS var) |
| 7 | OPT-8 | LOW | 30 min | Medium (needs new signal) |
| 8 | OPT-6 | LOW-MEDIUM | 1-2 hrs | Medium (blocked by label redesign) |

**OPT-1 (transform) is the biggest win but highest risk.** It touches the coordinate system that ALL interaction code depends on. Do it last after the safe wins are in, so you have a clean baseline to measure against.

**Quick wins first** (OPT-2, OPT-3, OPT-5) should save ~2-4ms combined with minimal risk. Then OPT-4 (box-shadow) for another 1-3ms. Then OPT-1 (transform) for the final 2-4ms.

## Estimated Total Savings

| Source | Est. Savings | Confidence |
|--------|-------------|------------|
| OPT-1: transform translate | 2-4ms | High (well-established browser optimization) |
| OPT-2: opt-level 3 | 1-3ms | High (LLVM benchmark data) |
| OPT-3: eliminate doc clone | 0.5-1ms | Medium (depends on im::HashMap clone cost) |
| OPT-4: remove box-shadow | 1-3ms | Medium (depends on GPU/browser) |
| OPT-5: hoist timestamp/UUID | 0.3-0.5ms | Medium (only when db_tx connected) |
| **Total** | **~5-12ms** | |

From 16.6ms → **~5-12ms**, well under the 8.33ms 120fps budget.
