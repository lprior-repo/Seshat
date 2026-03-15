# Test Plan

bead_id: seshat-9nf
bead_title: EDG-022 to EDG-026: Edge labels
phase: 1
updated_at: 2026-03-15T13:00:00Z

## Given/When/Then

### Edge Label Rendering
- **Given** an edge with `label: "A to B"` and `label_offset_t: 0.5`.
- **When** the diagram canvas is rendered.
- **Then** the label is visible at the midpoint coordinates `(source.x + target.x) / 2` and `(source.y + target.y) / 2`.

### Label Update
- **Given** an existing edge with label "A".
- **When** the `dispatch_update_label` event is fired for the edge.
- **Then** the document model reflects the new label without panics.