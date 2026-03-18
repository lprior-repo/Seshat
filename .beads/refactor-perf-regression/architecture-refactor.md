# Architecture Refactor Report

## Bead: refactor-perf-regression

### Refactor: Perf Regression Module Split

**Date**: 2026-03-17

**Original File**:
- `diagram_tool/src/perf/regression.rs` (512 lines)

**Issue**: File exceeded the strict 300 line limit.

**Solution**: Applied "Code is a Liability" mindset (DRY) and Scott Wlaschin DDD principles to split into a cohesive module.

**Files Changed/Created**:

#### `diagram_tool/src/perf/regression`
| File | Lines | Purpose |
|------|-------|---------|
| `result.rs` | 104 | Domain representation of `RegressionResult` and its logic. |
| `test.rs` | 228 | `RegressionTest` runner. Contains the logic for executing performance tests and comparing them against the baseline. |
| `report.rs` | 194 | Serialization and CI integration structs `PerformanceReport` and `MachineInfo`. |
| `mod.rs` | 9 | Module declarations and re-exports. |

### DDD Improvements Applied
- Separated domain representation (`RegressionResult`) from runner execution (`RegressionTest`) and IO/Reporting layer (`PerformanceReport`).
- Improved DRYness by separating concerns into multiple highly cohesive files, each holding exactly what it needs for its specific task.
- Ensured total clean compilation under `clippy` checks without warnings.