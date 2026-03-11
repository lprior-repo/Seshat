# Implementation Summary: Backspace Key Dispatch

See `.beads/seshat-6pi/implementation.md` for full details.

## This Bead: seshat-5y5

Modified Backspace key handler to dispatch `DomainOp::NodeDelete` to db_tx before local delete.

### Key Changes

- **`diagram_tool/src/ui/canvas.rs`**: Same keydown handler handles both Delete and Backspace (line 791: `match key { "Delete" | "Backspace" => ... }`)

### Contract Adherence

- Same as seshat-3ee (Delete key) since both keys use the same handler
- EARS-1: Construct and dispatch NodeDelete for each selected node
- EARS-2: No-op when no selection (handled by empty check)
- EARS-3: No trigger when input focused (existing `editing` check)
- EARS-4: Fall back to local mutation when db_tx unavailable

### Note

seshat-3ee and seshat-5y5 share the same implementation since both Delete and Backspace keys are handled by the same match arm in canvas.rs.
