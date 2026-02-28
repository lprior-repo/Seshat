# QA Report: bd-24a Grid Core State and Coordinate Conversion

Date: 2026-02-28
Workspace: `/home/lewis/src/bd-24a`
Scope: `diagram_tool/src/ui/grid/mod.rs` integration with editor/canvas, plus runtime smoke around related behavior.

## Verdict

`FAIL` (quality gate not met for full workspace runtime test)

- Required grid-focused checks/clippy/tests passed.
- A separate runtime/property-test failure exists in `ui::interaction` during full suite execution.

## Execution Evidence

### 1) Moon check

- Command: `moon run :check`
- Exit code: `0`
- Expected: build/check succeeds
- Actual: succeeded
- Key output:
  - `root:check ... Finished dev profile`
  - `Tasks: 1 completed`

### 2) Moon clippy

- Command: `moon run :clippy`
- Exit code: `0`
- Expected: clippy task succeeds
- Actual: succeeded
- Key output:
  - `root:clippy ... Finished dev profile`
  - `Tasks: 1 completed`

### 3) Grid-targeted tests

- Command: `cargo test ui::grid:: -- --nocapture`
- Exit code: `0`
- Expected: grid module tests pass
- Actual: succeeded
- Key output:
  - `running 40 tests`
  - `test result: ok. 40 passed; 0 failed`

## GridSize Validation Results

All checks below were executed through exact unit tests in `ui::grid::tests`.

1. `GridSize::new(5.0)` should return `Err(OutOfRange)`
   - Command: `cargo test ui::grid::tests::given_value_below_minimum_when_creating_grid_size_then_returns_out_of_range_error -- --exact --nocapture`
   - Exit code: `0`
   - Expected vs actual: `Err(OutOfRange)` expected, test passed.

2. `GridSize::new(150.0)` should return `Err(OutOfRange)`
   - Command: `cargo test ui::grid::tests::given_value_above_maximum_when_creating_grid_size_then_returns_out_of_range_error -- --exact --nocapture`
   - Exit code: `0`
   - Expected vs actual: `Err(OutOfRange)` expected, test passed.

3. `GridSize::new(f64::NAN)` should return `Err(NotFinite)`
   - Command: `cargo test ui::grid::tests::given_nan_value_when_creating_grid_size_then_returns_not_finite_error -- --exact --nocapture`
   - Exit code: `0`
   - Expected vs actual: `Err(NotFinite)` expected, test passed.

4. `GridSize::new(20.0)` should return `Ok`
   - Evidence command: `cargo test ui::grid::tests::given_snap_enabled_when_snapping_value_then_returns_grid_multiple -- --exact --nocapture`
   - Exit code: `0`
   - Expected vs actual: test constructs `GridSize::new(20.0).unwrap()`; if `new(20.0)` were not `Ok`, test would fail. Test passed.

5. `GridSize::default()` should return `20.0`
   - Command: `cargo test ui::grid::tests::given_default_when_getting_default_grid_size_then_returns_20 -- --exact --nocapture`
   - Exit code: `0`
   - Expected vs actual: `20.0` expected, test passed.

## Snap Function Results

1. `snap_value` when enabled snaps to grid multiple
   - Command: `cargo test ui::grid::tests::given_snap_enabled_when_snapping_value_then_returns_grid_multiple -- --exact --nocapture`
   - Exit code: `0`
   - Expected vs actual: expected snap to nearest multiple; passed.

2. `snap_value` when disabled is identity
   - Command: `cargo test ui::grid::tests::given_snap_disabled_when_snapping_value_then_returns_value_unchanged -- --exact --nocapture`
   - Exit code: `0`
   - Expected vs actual: unchanged expected; passed.

3. `snap_point` snaps coordinates independently
   - Command: `cargo test ui::grid::tests::given_point_when_snapping_then_each_coordinate_snapped_independently -- --exact --nocapture`
   - Exit code: `0`
   - Expected vs actual: independent x/y snapping expected; passed.

## Runtime/Adversarial Findings

### Finding 1 (MAJOR): Full test suite fails in interaction prop-test

- Severity: `MAJOR`
- Command: `moon run :test`
- Exit code: `1`
- Expected: full suite pass
- Actual: one failing property test
- Failing test:
  - `ui::interaction::proptests::prop_snap_value_enabled_is_multiple_of_grid`
  - Minimal failing input reported by test harness:
    - `value = 489921.7663134418`
    - `grid = 938.7793248099791`
- Error excerpt:
  - `assertion failed: remainder.abs() < f64::EPSILON || !result.is_finite()`
- Reproduction steps:
  1. `cd /home/lewis/src/bd-24a`
  2. Run `moon run :test`
  3. Observe failure in `ui::interaction::proptests::prop_snap_value_enabled_is_multiple_of_grid`.

### Observation (MINOR): Compile warnings in test builds

- Commands producing warning: multiple `cargo test ...` invocations and `moon run :test`
- Warnings:
  - `unused import: EditorState` in `diagram_tool/src/mutation/pipeline.rs:57`
  - `unused import: crate::ui::grid::GridSize` in `diagram_tool/src/mutation/pipeline.rs:61`
- Impact: non-blocking for execution, but should be cleaned up.

## Quality Gate Decision

- Grid bead-specific behavior: `PASS`
- Workspace runtime gate (`moon run :test`): `FAIL`
- Final recommendation for bead signoff: `CONDITIONAL FAIL` until interaction proptest failure is addressed or explicitly waived as pre-existing/non-scope.
