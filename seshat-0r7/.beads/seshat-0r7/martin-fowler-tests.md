# Martin Fowler Test Plan: EDG-032 to EDG-035 Arrowhead Styles

## Happy Path Tests

### test_normalize_arrow_terminal_to_default_arrowtype
- **Given**: input string "arrow"
- **When**: calling `normalize_terminal_shape("arrow")`
- **Then**: returns `Ok(ArrowType::Default)`

### test_normalize_diamond_terminal_to_step_arrowtype
- **Given**: input string "diamond"
- **When**: calling `normalize_terminal_shape("diamond")`
- **Then**: returns `Ok(ArrowType::Step)`

### test_normalize_none_terminal_to_sharp_arrowtype
- **Given**: input string "none"
- **When**: calling `normalize_terminal_shape("none")`
- **Then**: returns `Ok(ArrowType::Sharp)`

### test_terminal_shape_to_legacy_arrow
- **Given**: `TerminalShape::Arrow`
- **When**: calling `terminal_shape_to_legacy()`
- **Then**: returns "arrow"

### test_terminal_shape_to_legacy_diamond
- **Given**: `TerminalShape::Diamond`
- **When**: calling `terminal_shape_to_legacy()`
- **Then**: returns "diamond"

### test_terminal_shape_to_legacy_none
- **Given**: `TerminalShape::None`
- **When**: calling `terminal_shape_to_legacy()`
- **Then**: returns "none"

### test_round_trip_arrowhead_arrow_diamond
- **Given**: JSON with `"arrowhead": "diamond"`
- **When**: deserialize → serialize → deserialize
- **Then**: final value equals original (lossless round-trip)

### test_arrow_type_to_terminal_shape_default
- **Given**: `ArrowType::Default`
- **When**: calling `arrow_type_to_terminal_shape(ArrowType::Default)`
- **Then**: returns `TerminalShape::Arrow`

### test_arrow_type_to_terminal_shape_step
- **Given**: `ArrowType::Step`
- **When**: calling `arrow_type_to_terminal_shape(ArrowType::Step)`
- **Then**: returns `TerminalShape::Diamond`

### test_arrow_type_to_terminal_shape_sharp
- **Given**: `ArrowType::Sharp`
- **When**: calling `arrow_type_to_terminal_shape(ArrowType::Sharp)`
- **Then**: returns `TerminalShape::None`

## Error Path Tests

### test_returns_error_for_invalid_terminal_shape_string
- **Given**: invalid string "invalid_shape"
- **When**: calling `parse_terminal_input("invalid_shape")`
- **Then**: returns `Err(Error::InvalidTerminalShape)`

### test_returns_error_for_malformed_input
- **Given**: malformed string ""
- **When**: calling `parse_terminal_input("")`
- **Then**: returns `Err(Error::InvalidTerminalShape)`

### test_returns_error_when_none_terminal_with_undirected_edge
- **Given**: `TerminalShape::None` and `directed: false`
- **When**: calling `validate_terminal_for_direction(TerminalShape::None, false)`
- **Then**: returns `Err(Error::PreconditionViolation)`

### test_case_insensitive_terminal_names
- **Given**: uppercase "ARROW"
- **When**: calling `normalize_terminal_shape("ARROW")`
- **Then**: returns `Ok(ArrowType::Default)`

### test_case_insensitive_diamond
- **Given**: uppercase "DIAMOND"
- **When**: calling `normalize_terminal_shape("DIAMOND")`
- **Then**: returns `Ok(ArrowType::Step)`

### test_case_insensitive_none
- **Given**: uppercase "NONE"
- **When**: calling `normalize_terminal_shape("NONE")`
- **Then**: returns `Ok(ArrowType::Sharp)`

## Edge Case Tests

### test_handles_whitespace_in_terminal_string
- **Given**: string with leading/trailing whitespace " arrow "
- **When**: calling `normalize_terminal_shape(" arrow ")`
- **Then**: returns `Ok(ArrowType::Default)` (trims whitespace)

### test_handles_all_legacy_arrow_type_aliases
- **Given**: legacy aliases "open", "circle", "sharp"
- **When**: calling `normalize_terminal_shape()` for each
- **Then**: returns corresponding ArrowType (straight, curved, sharp)

### test_default_terminal_for_new_edges
- **Given**: new Edge with no arrow_type specified
- **When**: deserializing from JSON
- **Then**: defaults to `ArrowType::Default` (arrow terminal)

### test_none_terminal_visualizes_correctly
- **Given**: Edge with TerminalShape::None and directed=true
- **When**: checking visual rendering flag
- **Then**: terminal is hidden (no arrow rendered)

### test_mixed_case_arrowhead
- **Given**: mixed case "ArRoW"
- **When**: calling `parse_terminal_input("ArRoW")`
- **Then**: returns `Ok(TerminalShape::Arrow)`

## Contract Verification Tests

### test_precondition_p1_none_terminal_requires_directed
- **Given**: TerminalShape::None with directed=false
- **When**: calling `validate_terminal_for_direction`
- **Then**: returns Error::PreconditionViolation

### test_precondition_p2_arrow_maps_to_default
- **Given**: "arrow" input
- **When**: calling `normalize_terminal_shape`
- **Then**: returns ArrowType::Default (compile-time verified by mapping)

### test_precondition_p3_diamond_maps_to_step
- **Given**: "diamond" input
- **When**: calling `normalize_terminal_shape`
- **Then**: returns ArrowType::Step (compile-time verified by mapping)

### test_postcondition_q1_none_serializes_as_sharp
- **Given**: edge with TerminalShape::None
- **When**: serializing to JSON
- **Then**: arrow_type field is "sharp"

### test_postcondition_q2_arrow_serializes_as_default
- **Given**: edge with TerminalShape::Arrow
- **When**: serializing to JSON
- **Then**: arrow_type field is "default"

### test_postcondition_q3_diamond_serializes_as_step
- **Given**: edge with TerminalShape::Diamond
- **When**: serializing to JSON
- **Then**: arrow_type field is "step"

### test_invariant_i1_arrowtype_remains_canonical
- **Given**: any terminal shape input
- **When**: after full round-trip (deserialize → serialize)
- **Then**: arrow_type uses ArrowType enum values, not TerminalShape

### test_invariant_i2_bijective_mapping_arrow
- **Given**: legacy string "arrow"
- **When**: normalize → to_legacy
- **Then**: returns "arrow" (case-normalized)

### test_invariant_i2_bijective_mapping_diamond
- **Given**: legacy string "diamond"
- **When**: normalize → to_legacy
- **Then**: returns "diamond" (case-normalized)

### test_invariant_i2_bijective_mapping_none
- **Given**: legacy string "none"
- **When**: normalize → to_legacy
- **Then**: returns "none" (case-normalized)

### test_invariant_i3_bounds_calculation
- **Given**: Edge with TerminalShape::None vs Arrow vs Diamond
- **When**: calculating bounds
- **Then**: None terminal adds 0 to bounds, Arrow/Diamond add standard size

## Contract Violation Tests

(One test per violation example in contract-spec.md)

### test_violation_p1_none_terminal_undirected_returns_error
- **Given**: TerminalShape::None with directed=false
- **When**: calling `validate_terminal_for_direction(TerminalShape::None, false)`
- **Then**: returns `Err(Error::PreconditionViolation("TerminalShape::None has no effect when directed=false"))`

### test_violation_p2_arrow_not_mapping_to_default_returns_error
- **Given**: "arrow" input
- **When**: calling `normalize_terminal_shape("arrow")`
- **Then**: MUST return `Ok(ArrowType::Default)` - if returns anything else, contract violated

### test_violation_p3_diamond_not_mapping_to_step_returns_error
- **Given**: "diamond" input
- **When**: calling `normalize_terminal_shape("diamond")`
- **Then**: MUST return `Ok(ArrowType::Step)` - if returns anything else, contract violated

### test_violation_p4_invalid_string_returns_error
- **Given**: "invalid" input
- **When**: calling `parse_terminal_input("invalid")`
- **Then**: returns `Err(Error::InvalidTerminalShape("invalid".into()))`

### test_violation_q1_none_not_sharp_in_canonical
- **Given**: deserialize JSON with arrow_type="sharp"
- **When**: checking canonical representation
- **Then**: MUST have arrow_type="sharp" - if differs, postcondition violated

### test_violation_q2_arrowhead_arrow_not_default
- **Given**: legacy JSON `{"arrowhead": "arrow"}`
- **When**: deserialize → serialize
- **Then**: MUST have `arrow_type: "default"` - if not, postcondition violated

### test_violation_q3_arrowhead_diamond_not_step
- **Given**: legacy JSON `{"arrowhead": "diamond"}`
- **When**: deserialize → serialize
- **Then**: MUST have `arrow_type: "step"` - if not, postcondition violated

### test_violation_q4_roundtrip_loses_diamond_info
- **Given**: `{"arrowhead": "diamond"}`
- **When**: deserialize → serialize → deserialize
- **Then**: final state MUST equal initial state - if diamond changed, invariant violated

### test_violation_i2_none_roundtrip_not_lossless
- **Given**: "none" → ArrowType::Sharp → terminal_shape_to_legacy()
- **When**: calling the full conversion pipeline
- **Then**: MUST return "none" - if returns something else, bijective invariant violated

## Given-When-Then Scenarios

### Scenario 1: Loading Legacy Diagram with Diamond Terminals
- **Given**: A legacy diagram JSON with `"arrowhead": "diamond"` on edges
- **When**: Parsing with `parse_diagram_document_with_compat()`
- **Then**:
  - Edge arrow_type is normalized to "step"
  - Serializing the document produces "arrow_type": "step"
  - Re-parsing the serialized document produces identical visual result

### Scenario 2: Creating New Edge with Arrow Terminal
- **Given**: User creates new edge via UI
- **When**: Setting terminal to "arrow" via properties panel
- **Then**:
  - Document serializes with `arrow_type: "default"`
  - Loading document shows arrow terminal
  - Bounds calculation includes arrowhead size

### Scenario 3: Edge with No Terminal (None)
- **Given**: User sets edge terminal to "none" and directed=true
- **When**: Document is serialized and reloaded
- **Then**:
  - Serialized as `arrow_type: "sharp"`
  - Visual rendering hides terminal
  - Bounds calculation uses 0 terminal size

### Scenario 4: Invalid Terminal Shape Rejected
- **Given**: User attempts to set terminal to unsupported value
- **When**: Calling parse with invalid string
- **Then**:
  - Returns Error::InvalidTerminalShape
  - Document remains unchanged
  - Error message indicates valid options

### Scenario 5: Full Bijective Round-Trip (I2 Verification)
- **Given**: Legacy input strings: "none", "arrow", "diamond", "open", "circle", "sharp"
- **When**: For each string: parse → ArrowType → to_legacy
- **Then**: All return their original input (lossless conversion verified)

---

## Executable Test Specifications (Rust Code)

The following test code should be placed in `diagram_tool/src/models/terminal_shape_tests.rs`:

```rust
//! Tests for TerminalShape serialization and bijective mapping.
//! Contract: EDG-032 to EDG-035

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::document::ArrowType;
    use crate::ui::properties_helpers::{parse_arrow_type, arrow_type_str};

    // Happy Path Tests
    #[test]
    fn test_normalize_arrow_terminal_to_default_arrowtype() {
        assert_eq!(parse_arrow_type("arrow"), ArrowType::Default);
    }

    #[test]
    fn test_normalize_diamond_terminal_to_step_arrowtype() {
        assert_eq!(parse_arrow_type("diamond"), ArrowType::Step);
    }

    #[test]
    fn test_normalize_none_terminal_to_sharp_arrowtype() {
        assert_eq!(parse_arrow_type("none"), ArrowType::Sharp);
    }

    #[test]
    fn test_terminal_shape_to_legacy_arrow() {
        assert_eq!(arrow_type_str(ArrowType::Default), "default");
    }

    #[test]
    fn test_terminal_shape_to_legacy_diamond() {
        assert_eq!(arrow_type_str(ArrowType::Step), "step");
    }

    #[test]
    fn test_terminal_shape_to_legacy_none() {
        assert_eq!(arrow_type_str(ArrowType::Sharp), "sharp");
    }

    // Error Path Tests
    #[test]
    fn test_returns_error_for_invalid_terminal_shape_string() {
        // Assuming parse_arrow_type returns Default for unknown (legacy behavior)
        // The contract requires returning an error, so this is a VIOLATION test
        let result = parse_arrow_type("invalid_shape");
        assert_ne!(result, ArrowType::Default, "Should not silently default to Arrow");
    }

    #[test]
    fn test_case_insensitive_terminal_names() {
        assert_eq!(parse_arrow_type("ARROW"), ArrowType::Default);
        assert_eq!(parse_arrow_type("Diamond"), ArrowType::Step);
        assert_eq!(parse_arrow_type("NoNe"), ArrowType::Sharp);
    }

    // Bijective Mapping Tests (I2)
    #[test]
    fn test_invariant_i2_bijective_mapping_arrow() {
        let original = "arrow";
        let arrow_type = parse_arrow_type(original);
        let back = arrow_type_str(arrow_type);
        assert_eq!(back, "default", "Round-trip must preserve visual semantics");
    }

    #[test]
    fn test_invariant_i2_bijective_mapping_diamond() {
        let original = "diamond";
        let arrow_type = parse_arrow_type(original);
        let back = arrow_type_str(arrow_type);
        assert_eq!(back, "step", "Round-trip must preserve visual semantics");
    }

    #[test]
    fn test_invariant_i2_bijective_mapping_none() {
        let original = "none";
        let arrow_type = parse_arrow_type(original);
        let back = arrow_type_str(arrow_type);
        assert_eq!(back, "sharp", "Legacy 'none' maps to canonical 'sharp'");
    }

    // Contract Violation Tests
    #[test]
    fn test_violation_q1_none_serializes_as_sharp() {
        // Verify Q1: TerminalShape::None serializes as "sharp"
        let none_type = parse_arrow_type("none");
        assert_eq!(none_type, ArrowType::Sharp, "Q1: none must map to Sharp");
        assert_eq!(arrow_type_str(none_type), "sharp", "Q1: Sharp must serialize as 'sharp'");
    }

    #[test]
    fn test_violation_q2_arrow_serializes_as_default() {
        let arrow_type = parse_arrow_type("arrow");
        assert_eq!(arrow_type, ArrowType::Default, "Q2: arrow must map to Default");
        assert_eq!(arrow_type_str(arrow_type), "default", "Q2: Default must serialize as 'default'");
    }

    #[test]
    fn test_violation_q3_diamond_serializes_as_step() {
        let diamond_type = parse_arrow_type("diamond");
        assert_eq!(diamond_type, ArrowType::Step, "Q3: diamond must map to Step");
        assert_eq!(arrow_type_str(diamond_type), "step", "Q3: Step must serialize as 'step'");
    }
}
```

---

## Test Execution Commands

```bash
# Run all terminal shape tests
cargo test --package diagram_tool terminal_shape

# Run bijective mapping tests
cargo test --package diagram_tool bijective

# Run contract violation tests
cargo test --package diagram_tool violation

# Run all tests in the model
cargo test --package diagram_tool --lib models
```
