# Implementation: seshat-4a8 - BringToFront Dispatch

## Contract Summary
Wired the BringToFront toolbar button to construct `DomainOp::BringToFront` and dispatch to `db_tx` coroutine.

## Changes Made

### 1. `diagram_tool/src/ui/dispatch.rs`
- Added `create_bring_to_front_envelope(ids: Vec<String>) -> EventEnvelope` function
- Added `dispatch_bring_to_front(db_tx, node_ids) -> Result<DispatchResult, DispatchError>` function

### 2. `diagram_tool/src/ui/toolbar/actions.rs`
- Updated `bring_to_front()` to accept `db_tx: Option<Coroutine<EventEnvelope>>` parameter
- Added selection extraction: filters selected_items to only include node IDs that exist in document.nodes
- Dispatches to db_tx if available, falls back to direct apply_bring_to_front mutation

### 3. `diagram_tool/src/ui/toolbar.rs`
- Updated toolbar button onclick handlers to pass db_tx context to actions::bring_to_front()

## Constraint Adherence

| Constraint | Implementation |
|------------|----------------|
| Zero panics/unwrap | All functions return Result/Option, no unwrap() in core logic |
| Zero mut | Uses doc_signal.read() for immutable access, apply_* functions handle mutation at boundary |
| Data→Calc→Actions | Dispatch functions are pure calculations; actual mutation pushed to apply_* |
| Expression-based | Uses iter().filter().collect() pipeline patterns |
| Clippy flawless | Code compiles with -D warnings |

## Files Changed
- `diagram_tool/src/ui/dispatch.rs` - Added z-order envelope creation and dispatch functions
- `diagram_tool/src/ui/toolbar/actions.rs` - Updated bring_to_front action to dispatch
- `diagram_tool/src/ui/toolbar.rs` - Updated toolbar button to pass db_tx
