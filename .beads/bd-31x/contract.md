# Contract: bd-31x - selection: close remaining hit-test regressions

## bead_id: bd-31x
## bead_title: selection: close remaining hit-test regressions
## phase: p0
## updated_at: 2026-02-28T22:27:01Z

## ears_requirements
- **ubiquitous**: THE SYSTEM SHALL produce deterministic selection results for identical pointer input.
- **event_driven**: 
  - WHEN user clicks overlapping selectable geometry, SHALL: THE SYSTEM SHALL apply stable tie-break ordering.
- **unwanted**:
  - IF thin edges are clicked at valid tolerance, SHALL NOT: THE SYSTEM SHALL NOT drop selection due to zoom-dependent jitter (because: precision regressions make core editing unreliable)

## contracts
### preconditions
- auth_required: false
- required_inputs: []
- system_state: find_edge_at and selection reducers are reachable by unit and E2E tests

### postconditions
- state_changes:
  - Overlap and thin-edge tests pass at multiple zoom levels

### invariants
- Hit radius semantics remain screen-consistent by zoom
- Selection transitions remain deterministic

## implementation_tasks
### phase_1_tests_first
- Gate: gate_0_research
- Tasks:
  1. Add missing deterministic hit-test scenarios (parallel_group: tests)

### phase_2_implementation
- Gate: gate_1_tests
- Tasks:
  1. Fix tie-break and tolerance defects (done_when: Tests pass)

## Known Failing Test
- Test: `given_low_zoom_when_clicking_near_edge_then_hit_test_uses_screen_consistent_radius`
- Location: diagram_tool/src/ui/canvas/canvas_view.rs:538
- Issue: Hit-test at low zoom (0.5x) fails to detect edges - assertion fails (left: None, right: Some(EdgeId("e1")))

## Context Files to Read
- diagram_tool/src/ui/canvas/canvas_view.rs
- diagram_tool/e2e/diagram.edges-and-routing.spec.ts
- diagram_tool/e2e/diagram.nodes-and-selection.spec.ts
- diagram_tool/src/ui/interaction.rs
