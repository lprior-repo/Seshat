# Implementation Summary: Toolbar Add Node Dispatch

See `.beads/seshat-6pi/implementation.md` for full details.

## This Bead: seshat-8xu

Added "Add Node" button to toolbar that dispatches `DomainOp::NodeAdd` to db_tx.

### Key Changes

- **`diagram_tool/src/ui/toolbar/actions.rs`**: Added `add_node()` function
- **`diagram_tool/src/ui/toolbar.rs`**: Added button with onclick wired to dispatch

### Contract Adherence

- P1: db_tx available via Dioxus context
- P3: Valid DiagramDocument via context
- Q1-Q4: EventEnvelope created with valid fields
- Q5: Dispatch to db_tx via `dispatch_node_add()`
- Q6-Q9: Local document state updated after dispatch
