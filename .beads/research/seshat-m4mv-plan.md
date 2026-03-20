# Refactoring Plan: seshat-m4mv (Phase 4 DDD State Machines)

## 1) Refactor Contract (Target Invariants + States)
**Goal:** Enforce domain correctness using types, eliminating boolean soup and implicit state encodings ("Option-as-state"). Achieve strict "Zero Panics" in core by removing the 13 remaining `unwrap()` calls.

### Target Invariants:
- **`SnapResult`**: Cannot contain positional data or target nodes if snapping is disabled/inactive.
- **Keyboard Actions**: No boolean parameters for branching behavior (`ctrl_pressed`, `shift`, etc.). Will use typed enums and bitflags.
- **Zero Panics**: No `unwrap()` or `expect()` in core domain logic. Fallible operations must return `Result<T, DomainError>`.

## 2) Typed Model Diffs (Before/After)

### `SnapResult` Refactor
**Before (Boolean Soup):**
```rust
pub struct SnapResult {
    pub active: bool,
    pub snap_type: SnapType,
    pub target_node_id: NodeId,
    pub snapped_position: Point,
}
```

**After (Explicit State Enum):**
```rust
pub enum SnapResult {
    Snapped {
        snap_type: SnapType,
        target: NodeId,
        pos: Point,
    },
    Unsnapped
}
```

### `core::keyboard` Refactor
**Before:**
```rust
pub fn map_key_to_action(key: &str, ctrl_or_meta: bool, shift: bool, is_editing_text: bool) -> KeyAction
```

**After:**
```rust
pub enum Modifiers {
    None,
    CtrlOrMeta,
    Shift,
    CtrlAndShift,
}

pub enum EditorContext {
    Canvas,
    EditingText,
}

pub fn map_key_to_action(key: KeyCode, modifiers: Modifiers, context: EditorContext) -> KeyAction
```

### Other Detected Option-as-State / Boolean Flags
The following boolean states will be reviewed and converted to explicit enums:
- `query_active: bool` in `diagram_tool/src/ui/sidebar/mod.rs` & `ui/sidebar_primitives/group.rs` -> `enum QueryState { Active(QueryData), Inactive }`
- `multi_touch_active: bool` in `diagram_tool/src/ui/canvas/node_layer/handlers.rs` -> explicit InteractionState transitions.

## 3) Transition Map
- **Snap Transitions:** Driven by `compute_snap` turning a `RawPoint` into either `SnapResult::Snapped` or `SnapResult::Unsnapped`.
- **Keyboard Transitions:** DOM Keyboard events parse at the boundary into `(KeyCode, Modifiers)`.

## 4) Boundary Parsing Plan (Parse, Don't Validate)
All raw input (e.g. `web_sys::MouseEvent` or raw `x, y` strings) will be constrained into types *before* entering core functions. Any fallible parse step will return a variant from the error taxonomy.

## 5) Error Taxonomy & `unwrap()` Purge
We identified ~33 instances of `.unwrap()` in non-test files (like `routing.rs`, `png.rs`, `auto_save.rs`, `canonical_json.rs`, `node_resize.rs`). Although some reside in embedded `#[cfg(test)]`/Kani modules, the 13 production unwraps must be purged.

**New Errors to Introduce/Expand:**
- `DomainError::CoordinateOutOfBounds` (Replacing geometry unwraps)
- `DomainError::NodeNotFound(NodeId)` (Replacing `nodes.get(&id).unwrap()`)
- `SerializationError::InvalidJson` (Replacing `serde_json::to_string(&state).unwrap()`)
- `ExportError::IoError` (Replacing temp file path unwraps)

## 6) Test Plan for Transitions/Invariants
- **Kani Proofs:** Update `verify_orthogonal_route_wrapper` and related proofs to expect `Result` instead of empty vectors when errors occur.
- **Unit Tests:** Update all `SnapResult` assertions to match `SnapResult::Snapped { .. }`.
- **Interaction Fuzzing:** Verify that invalid boolean combinations can no longer be constructed in fuzzing setups.

## 7) Migration Notes and Risks
- **Risk:** Existing legacy callers expecting a default `SnapResult::inactive()` will break. We must migrate callers to match on `SnapResult`.
- **Risk:** `orthogonal_route` backward compatibility wrapper discards errors. Callers need to be updated to handle `Result<OrthogonalRoute, RoutingError>` before the wrapper is removed.