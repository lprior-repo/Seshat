# Contract Specification

bead_id: seshat-9nf
bead_title: EDG-022 to EDG-026: Edge labels
phase: 1
updated_at: 2026-03-15T13:00:00Z

## Preconditions
1. The `Edge` struct is well-formed.
2. An edge's `label` string can be non-empty.

## Postconditions
1. Edge labels are explicitly drawn at the midpoint of an edge, calculated via the edge's source and target nodes and the `label_offset_t` scalar.
2. Updating the edge label emits an update event and reflects in the UI.

## Invariants
- Edge geometry calculations for labels do not panic.
- Text offset parameters range from [0.0, 1.0].