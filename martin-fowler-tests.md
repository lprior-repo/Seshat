# Martin Fowler Test Plan

## Happy Path Tests
- `test_ordered_float_new_accepts_valid_finite_value`
  Given: A valid finite f64 value (e.g., 42.0, -10.5, 0.0)
  When: OrderedFloat::new() is called
  Then: Returns Ok(OrderedFloat(value))

- `test_ordered_float_constructor_valid_value`
  Given: Valid positive and negative floats
  When: Creating OrderedFloat instances
  Then: All operations work correctly

## Error Path Tests
- `test_ordered_float_new_rejects_nan`
  Given: f64::NAN
  When: OrderedFloat::new(NAN) is called
  Then: Returns Err(OrderedFloatError::NaN)

- `test_ordered_float_new_rejects_positive_infinity`
  Given: f64::INFINITY
  When: OrderedFloat::new(INFINITY) is called
  Then: Returns Err(OrderedFloatError::Infinite)

- `test_ordered_float_new_rejects_negative_infinity`
  Given: f64::NEG_INFINITY
  When: OrderedFloat::new(NEG_INFINITY) is called
  Then: Returns Err(OrderedFloatError::Infinite)

## Edge Case Tests
- `test_ordered_float_accepts_zero`
  Given: 0.0 and -0.0
  When: OrderedFloat::new() is called
  Then: Returns Ok (both are finite)

- `test_ordered_float_accepts_extreme_finite_values`
  Given: f64::MIN, f64::MAX, very small subnormal
  When: OrderedFloat::new() is called
  Then: Returns Ok (all are finite)

- `test_ordered_float_arithmetic_preserves_finiteness`
  Given: Two valid OrderedFloat values
  When: Adding, subtracting, multiplying, dividing
  Then: Result is valid (may be inf/nan from arithmetic, but that's expected from operations)

## Schema Validation Tests
- `test_schema_rejects_nan_node_coordinates`
  Given: Node with x=NAN, y=NAN
  When: validate_schema() is called
  Then: Returns Err with "non-finite" message

- `test_schema_rejects_inf_node_dimensions`
  Given: Node with width=INFINITY
  When: validate_schema() is called
  Then: Returns Err with "invalid width" message

- `test_schema_rejects_inf_edge_properties`
  Given: Edge with label_offset_t=INFINITY or thickness=INFINITY
  When: validate_schema() is called
  Then: Returns Err with appropriate message

- `test_schema_rejects_inf_editor_state`
  Given: EditorState with camera_x=NaN or zoom=INFINITY
  When: validate_schema() is called
  Then: Returns Err with "non-finite" message

## Contract Violation Tests
- `test_precondition_nan_violation_returns_error`
  Given: OrderedFloat::new(f64::NAN)
  Then: Returns Err(OrderedFloatError::NaN) -- NOT Ok, NOT panic

- `test_precondition_inf_violation_returns_error`
  Given: OrderedFloat::new(f64::INFINITY)
  Then: Returns Err(OrderedFloatError::Infinite) -- NOT Ok, NOT panic

- `test_precondition_neg_inf_violation_returns_error`
  Given: OrderedFloat::new(f64::NEG_INFINITY)
  Then: Returns Err(OrderedFloatError::Infinite) -- NOT Ok, NOT panic

## Given-When-Then Scenarios

### Scenario 1: Creating a valid node
Given: A Node with x=100.0, y=200.0, width=80.0, height=40.0
When: Schema validation runs
Then: No errors are returned

### Scenario 2: Deserializing document with NaN
Given: JSON with "x": NaN
When: serde_json::from_str<Node> is called
Then: Should fail with deserialization error (or use new_unchecked at call site)

### Scenario 3: User enters Infinity in UI
Given: User enters Infinity as node width
When: Document is validated
Then: Schema returns error "Node has invalid width: inf"
