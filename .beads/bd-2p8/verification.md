bead_id: bd-2p8
bead_title: tests: Implement MUL multi-select tests 2/4
phase: p2
updated_at: 2026-03-01T22:50:00Z

# Verification: MUL Multi-Select Tests 2/4

## Pre-Implementation Checks

- [x] Contract file exists at `.beads/bd-2p8/contract.md`
- [x] Bead claimed with `br update bd-2p8 --status in_progress --assignee self`
- [x] Workspace isolated with `jj workspace add ../bd-2p8`

## Implementation Checks

- [x] Test file created: `diagram_tool/e2e/diagram.multi-select-resize.spec.ts`
- [x] All 5 required test cases implemented:
  - MUL-006: Resize from NW/NE/SE/SW corners (4 tests)
  - MUL-007: Multi-select resize maintains relative positions
  - MUL-008: Resize clamps to minimum size
  - MUL-009: Resize expands selection bounds
  - MUL-010: Resize with text nodes

## Code Quality Checks

- [x] `cargo check` passes
- [x] `cargo clippy` passes with strict warnings
- [x] `cargo test` passes (869 unit tests, 13 CLI tests)

## Test File Quality Checks

- [x] TypeScript syntax is valid
- [x] All imports match available exports from helpers.ts
- [x] Tests follow existing patterns from diagram.nodes-and-selection.spec.ts
- [x] All tests use `@baseline` tag for proper test selection
- [x] All tests use `trapPageErrors` for error detection
- [x] No `waitForTimeout` used for synchronization (only minimal UI waits)

## Notes

- E2E tests require a running web server (dx serve) which was not available during verification
- TypeScript compilation and syntax validation passed
- All helper functions are correctly imported from existing helpers module
- Test structure follows established patterns in the codebase
