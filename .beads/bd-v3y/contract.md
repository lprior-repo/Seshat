# Contract: selection: Multi-select with Ctrl+Click and Shift+Click

bead_id: bd-v3y
bead_title: selection: Multi-select with Ctrl+Click and Shift+Click
phase: p0
updated_at: 2026-02-28T19:35:00Z

## EARS Requirements

### Ubiquitous
- THE SYSTEM SHALL maintain a set of selected node IDs
- THE SYSTEM SHALL display selection highlight on all selected nodes

### Event-Driven
- WHEN user Ctrl+Clicks a node, THE SYSTEM SHALL toggle that node in the selection set
- WHEN user clicks without modifier, THE SYSTEM SHALL replace selection with clicked node

### Unwanted
- IF selection becomes inconsistent with actual nodes, THE SYSTEM SHALL NOT retain stale node IDs in selection, because: Stale selections cause undefined behavior in operations

## Preconditions
- auth_required: false
- required_inputs: []
- system_state:
  - Selection state exists as a Signal of HashSet<NodeId>
  - Nodes have stable unique IDs

## Postconditions
- state_changes:
  - Selection contains only valid node IDs
  - All selected nodes show visual selection indicator

## Invariants
- Selection is always a subset of existing nodes
- Empty selection is valid state

## Implementation Tasks (COMPLETED)
1. Selection state is im::HashSet<String> - ✅ Already exists
2. toggle_selection() function exists - ✅ Implemented in interaction.rs
3. Ctrl detection in node click handler - ✅ Implemented (line 2044)
4. Visual selection highlight - ✅ Implemented via selected_items.contains()

## Acceptance Tests
- Multi-select toggle works with Ctrl+Click
- Plain click clears previous selection
- Shift+Click also acts as additive (like Ctrl)
