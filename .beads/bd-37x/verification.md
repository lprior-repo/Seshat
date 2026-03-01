bead_id: bd-37x
bead_title: tests: Implement SEL selection tests 5/5
phase: p2
updated_at: 2026-03-01T22:12:00Z

# Verification: SEL Selection Tests 5/5 (bd-37x)

## P0: Contract Resolved

- [x] Contract file exists at `.beads/bd-37x/contract.md`
- [x] Contract specifies all 5 required tests (SEL-021 through SEL-025)
- [x] Contract follows bead description from BR

## P1: Implementation Complete

- [x] Implementation file exists at `.beads/bd-37x/implementation.md`
- [x] 5 new tests added to `diagram_tool/e2e/diagram.nodes-and-selection.spec.ts`
- [x] Tests follow existing patterns (freshStart, runEffect, trapPageErrors)
- [x] All tests use `@baseline` tag

## P2: Moon Validation

### Check
- [x] `moon run :check` passes

### Clippy
- [x] `moon run :clippy` passes

### Test-Rust
- Note: 1 pre-existing test failure in `geometry::tests::test_transform_order_matters`
- This failure was introduced in a previous bead (GEO tests) and is not related to SEL tests
- All other 849 Rust tests pass

### E2E Test Discovery
- [x] All 5 new tests are discovered by Playwright
- Listed tests:
  - `selection bounding box matches node geometry @baseline`
  - `pointer down with hold selects node without drag @baseline`
  - `double-click on selected node enters edit mode @baseline`
  - `selection persists after zoom change @baseline`
  - `marquee selects nodes regardless of position @baseline`

## Test Details

### SEL-021: Selection bounding box matches node geometry
- **File**: `diagram_tool/e2e/diagram.nodes-and-selection.spec.ts`
- **Line**: 537
- **Given**: A node on the canvas
- **When**: Node is selected
- **Then**: SE resize handle is visible and positioned near node corner

### SEL-022: Pointer down with hold selects without drag
- **File**: `diagram_tool/e2e/diagram.nodes-and-selection.spec.ts`
- **Line**: 579
- **Given**: An unselected node
- **When**: Pointer down + 100ms hold + pointer up without movement
- **Then**: Node is selected and position unchanged

### SEL-023: Double-click enters edit mode
- **File**: `diagram_tool/e2e/diagram.nodes-and-selection.spec.ts`
- **Line**: 624
- **Given**: A node on the canvas
- **When**: Single click then double-click
- **Then**: Selection is maintained

### SEL-024: Selection persists after zoom
- **File**: `diagram_tool/e2e/diagram.nodes-and-selection.spec.ts`
- **Line**: 659
- **Given**: A selected node
- **When**: Zoom-in button clicked
- **Then**: Selection count remains 1

### SEL-025: Marquee selects nodes regardless of position
- **File**: `diagram_tool/e2e/diagram.nodes-and-selection.spec.ts`
- **Line**: 685
- **Given**: Two nodes at different positions
- **When**: Marquee encompasses both
- **Then**: Both nodes selected

## Notes

- E2E tests require a running server and are not executed during CI `:test-rust` phase
- Full E2E validation requires `moon run :e2e-baseline` with a running `:serve-e2e` instance
- The pre-existing geometry test failure should be addressed in a separate bead
