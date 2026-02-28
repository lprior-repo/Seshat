# Implementation Summary: bd-1s3

## bead_id: bd-1s3
## bead_title: baseline: execute and fix failing suite
## phase: p0
## completed_at: 2026-02-28

## Test Execution Results

### Unit Tests (moon run :test)
- **Total**: 489 tests
- **Passed**: 488 tests
- **Failed**: 1 test
- **Status**: See Known Issues below

### E2E Tests
- **Skipped per contract**: E2E tests not run (focused on unit tests only as per contract)

## Known Issues

### Pre-existing Failure: Low Zoom Hit-Test (canvas_view.rs:538)

**Test**: `ui::canvas::canvas_view::tests::given_low_zoom_when_clicking_near_edge_then_hit_test_uses_screen_consistent_radius`

**Location**: `diagram_tool/src/ui/canvas/canvas_view.rs:538`

**Issue**: The hit-test at low zoom (zoom = 0.5) fails to detect edges that should be hit when using screen-consistent radius calculations.

**Status**: This is a known pre-existing issue explicitly documented in the contract:
> "There's a known pre-existing hit-test failure at low zoom (canvas_view.rs:538) - this is addressed by a separate bead (bd-31x)."

**Tracked in**: Bead bd-31x (separate work item)

## Analysis

The baseline unit test suite is functionally stable with 488/489 tests passing. The single failing test represents a known regression that:
1. Is already identified and documented
2. Has a dedicated tracking bead (bd-31x) for resolution
3. Does not impact core functionality at normal zoom levels

## Postconditions Status

| Postcondition | Status | Notes |
|----------------|--------|-------|
| Baseline project exits with success | Partial | 488/489 tests pass; 1 known failure tracked in bd-31x |
| Any discovered bug has regression coverage | ✓ | The failing test provides regression coverage for the issue |

## Conclusion

The unit test baseline is stable. The single failing test is a known issue (tracked separately in bead bd-31x) and does not block the baseline from serving as a quality gate. No code changes were required as the issue is already tracked for resolution in the appropriate work item.
