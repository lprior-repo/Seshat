# Martin Fowler Test Plan: Marquee Performance (seshat-kgc)

## Context & ATDD DSL
Tests must use a Domain-Specific Language (DSL) interacting with the diagram document directly. We test real execution (Testing Trophy) and durable state updates, strictly avoiding tight coupling to internal spatial index implementations, code mechanics, or internal synchronization flags. All scenarios are expressed in pure domain language.

## Happy Path Tests
- `should_report_fully_enclosed_nodes_as_selected_in_contain_mode`
- `should_report_intersecting_nodes_as_selected_in_intersect_mode`
- `should_successfully_process_3000_node_grid_without_crashing`
- `should_accurately_select_rotated_nodes_within_marquee`

## Error Path Tests
- `should_reject_marquee_with_negative_dimensions`

## Edge Case & Combinatorial Tests
- `should_evaluate_node_inclusion_across_rotation_and_boundary_and_mode_permutations`
  *(Parameterized test matrix: [0 deg, 90 deg, arbitrary] x [inside, partial, outside, exactly on edge, exactly on corner] x [Contain, Intersect])*
- `should_handle_zero_area_marquee`
- `should_handle_extremely_large_marquee_bounds`

## Property-Based Tests (Fuzzing)
- `should_always_match_linear_scan_results_under_random_layouts`
  Given: A property-based fuzzer generating random node layouts, rotations, and marquee bounds
  When: A marquee selection is evaluated against both the optimized spatial index and a linear scan
  Then: The resulting selection states are perfectly identical

## Contract Verification Tests
- `should_reject_marquee_when_dimensions_are_negative`
- `should_match_linear_scan_results_identically`
- `should_strictly_exclude_partially_intersecting_nodes_in_contain_mode`
- `should_accurately_select_rotated_nodes_within_bounds`
- `should_maintain_unaltered_observable_document_state_excluding_selection`

## Contract Violation Tests
- `should_return_invalid_bounds_error_for_negative_dimensions`
  Given: A diagram document ready for selection
  When: The user attempts a marquee selection with negative width dimensions: `ValidRect::new(0.0, 0.0, -10.0, 10.0)`
  Then: The operation is rejected with `Err(Error::InvalidMarqueeBounds)`

## Given-When-Then Scenarios
### Scenario 1: End-to-End Real Execution Large Scale Selection
Given: A diagram document populated with 3000 nodes arranged in a deterministic grid layout
When: The user executes a marquee selection covering a subset of the diagram area
Then: 
- The diagram correctly reports the enclosed nodes as selected
- The document remains stable and observable layout state is unchanged

### Scenario 2: Rejection of Invalid Marquee Bounds
Given: A diagram document
When: The user attempts to define a marquee selection with negative width and height
Then:
- The operation is rejected
- The observable document state remains unchanged