# Martin Fowler Test Plan

## Edge Labels (EDG-022 to EDG-026)

## Happy Path Tests
- **test_edge_label_renders_at_midpoint_by_default**
  Given: An edge with label "test" and default label_offset_t (0.5)
  When: Computing label position
  Then: Returns the geometric midpoint between source and target

- **test_edge_label_renders_at_custom_offset**
  Given: An edge with label "test" and label_offset_t = 0.25
  When: Computing label position
  Then: Returns position at 25% along the edge path

- **test_edge_label_serializes_to_json**
  Given: An edge with label "calls" and label_offset_t = 0.5
  When: Serializing to JSON
  Then: JSON contains `"label":"calls"` and `"label_offset_t":0.5`

- **test_edge_label_deserializes_from_json**
  Given: JSON with `"label":"test"` and `"label_offset_t":0.5`
  When: Deserializing to Edge
  Then: Edge has label "test" and label_offset_t = 0.5

- **test_update_edge_label_changes_label**
  Given: An edge with empty label
  When: Applying update to set label to "new label"
  Then: Edge label is "new label"

## Error Path Tests
- **test_edge_label_offset_clamped_to_valid_range**
  Given: An edge with label_offset_t = 1.5
  When: Computing label position
  Then: Position is clamped to endpoint (t=1.0)

- **test_edge_label_offset_negative_clamped**
  Given: An edge with label_offset_t = -0.5
  When: Computing label position
  Then: Position is clamped to source (t=0.0)

- **test_edge_label_nan_replaced_with_default**
  Given: An edge with label_offset_t = NaN
  When: Computing label position
  Then: Falls back to default 0.5

## Edge Case Tests
- **test_empty_edge_label_not_rendered**
  Given: An edge with empty label ""
  When: Canvas rendering checks
  Then: Label is not rendered

- **test_unicode_edge_label_renders_correctly**
  Given: An edge with unicode label "→ connects 🔗"
  When: Serializing and deserializing
  Then: Label roundtrips correctly

- **test_edge_label_visible_above_zoom_threshold**
  Given: An edge with non-empty label at zoom 0.3
  When: Canvas rendering checks visibility
  Then: Label is visible

- **test_edge_label_hidden_below_zoom_threshold**
  Given: An edge with non-empty label at zoom 0.2
  When: Canvas rendering checks visibility
  Then: Label is hidden

## Contract Verification Tests
- **test_label_offset_t_finite_validation**
  Given: An edge with infinite label_offset_t
  When: Schema validation
  Then: Returns validation error

- **test_label_offset_t_range_validation**
  Given: An edge with label_offset_t = 2.0
  When: Schema validation
  Then: Returns validation error

## Given-When-Then Scenarios

### Scenario 1: Edge label positioned at midpoint
Given: Two nodes at (0, 0) and (100, 0) with an edge labeled "flow"
When: Computing label position with default offset (0.5)
Then: Returns (50, 0)

### Scenario 2: Edge label on curved path
Given: Two nodes with a curved edge (quadratic bezier)
When: Computing label position at t=0.5
Then: Returns point on the bezier curve at midpoint

### Scenario 3: Edge with bend points
Given: An edge with bend points [(25, 0), (75, 0)]
When: Computing label position at t=0.5
Then: Returns interpolated point along the polyline

### Scenario 4: Edge label persistence
Given: A document with an edge containing label "test" at offset 0.3
When: Saving to file and loading back
Then: Edge label and offset are preserved
