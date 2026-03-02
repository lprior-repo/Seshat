bead_id: bd-2b4
bead_title: commands: Add unit tests for copy/paste operations
phase: p0
updated_at: 2026-03-01T17:00:00Z

# Contract: Copy/Paste Operations Unit Tests

## System Under Test

Target: `diagram_tool/src/ui/commands.rs`
Functions: `apply_copy_selection`, `apply_paste_selection`

## Preconditions

1. `commands.rs` has `#[cfg(test)]` module
2. Test helper functions exist (`make_doc_with_node`, `clear_clipboard`, etc.)
3. Dioxus Signal infrastructure available for test usage

## Postconditions

1. `apply_copy_selection` has 3+ unit tests covering:
   - Empty selection (returns false)
   - Single node copy
   - Multiple nodes with edges
   - Partial edge selection (excluded)
   - Parent-child relationships

2. `apply_paste_selection` has 3+ unit tests covering:
   - Empty clipboard (returns false)
   - Empty nodes vector (returns false)
   - Single node paste with offset
   - Multiple paste iterations (offset accumulation)
   - Edge remapping
   - Parent remapping
   - Selection update
   - Revision increment

3. All tests pass with `cargo test --package diagram_tool`

## Invariants

1. Tests follow existing patterns in commands.rs
2. Tests use zero-unwrap policy (allow attribute at module level)
3. Clipboard state is isolated between tests
4. Test names follow `given_X_when_Y_then_Z` pattern

## Coverage Requirements

| Scenario | Copy | Paste |
|----------|------|-------|
| Empty selection/clipboard | Covered | Covered |
| Single node | Covered | Covered |
| Multiple nodes | Covered | Covered |
| Nodes with edges | Covered | Covered |
| Partial edge selection | Covered | N/A |
| Parent-child relationship | Covered | Covered |
| Multiple paste iterations | N/A | Covered |
