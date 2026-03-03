bead_id: bd-1g4
bead_title: perf-baseline: Establish 3000-node performance benchmarks (120 FPS target)
phase: p4
updated_at: 2026-03-03T00:00:00Z

# Red Queen Report: Adversarial Testing

## Stress Testing

### Maximum Node Count (10,000 nodes)

| Test | Input | Result | Status |
|------|-------|--------|--------|
| Benchmark 10000 nodes | 10000 nodes, 200ms | Completes without error | PASS |
| Memory stability | 10000 nodes | No panic, no OOM | PASS |
| Statistics validity | 10000 nodes | All values finite | PASS |

### Minimum Node Count (1 node)

| Test | Input | Result | Status |
|------|-------|--------|--------|
| Benchmark 1 node | 1 node, 200ms | Completes with valid stats | PASS |
| FPS calculation | 1 node | Mean FPS > 0 | PASS |

### Edge Cases

| Test | Input | Expected | Result | Status |
|------|-------|----------|--------|--------|
| Zero node count | 0 | Error | InvalidNodeCount(0) | PASS |
| Over-max nodes | 10001 | Error | InvalidNodeCount(10001) | PASS |
| Zero duration | 0ms | Error | InvalidDuration(0) | PASS |
| Below-min duration | 50ms | Error | InvalidDuration(50) | PASS |
| Very long benchmark | 60s | Valid result | Not tested (too slow) | SKIP |

## Invariant Violation Tests

### INV-1: No NaN/Infinity

| Test | Method | Result | Status |
|------|--------|--------|--------|
| NaN in mean_fps | Inject via sample | InvariantViolation error | PASS |
| Infinity in std_dev | Inject via sample | InvariantViolation error | PASS |

### INV-2: Monotonic Timestamps

| Test | Method | Result | Status |
|------|--------|--------|--------|
| Non-monotonic timestamps | Manual construction | InvariantViolation error | PASS |

### INV-3: Frame Time/FPS Consistency

| Test | Method | Result | Status |
|------|--------|--------|--------|
| Mismatched frame time | Manual construction | InvariantViolation error | PASS |

### INV-4: Sample Count Mismatch

| Test | Method | Result | Status |
|------|--------|--------|--------|
| Count != samples.len() | Manual override | InvariantViolation error | PASS |

### INV-5: Percentile Ordering

| Test | Method | Result | Status |
|------|--------|--------|--------|
| Unordered percentiles | Manual construction | is_ordered() returns false | PASS |

## Determinism Verification

| Test | Seed | Result | Status |
|------|------|--------|--------|
| Same seed, same scene | 42 | Identical node positions | PASS |
| Same seed, same benchmark | 42 | Similar sample counts | PASS |
| Different seeds | 42 vs 123 | Different results | PASS |

## Error Recovery

| Test | Scenario | Result | Status |
|------|----------|--------|--------|
| Insufficient samples | < 10 samples | InsufficientSamples error | PASS |
| Custom min samples | 20 required, 15 provided | InsufficientSamples error | PASS |
| Baseline not found | Missing file | BaselineNotFound error | PASS |
| Unknown operation | "unknown" op | BaselineNotFound error | PASS |

## Regression Detection

| Test | Current FPS | Baseline FPS | Threshold | Result | Status |
|------|-------------|--------------|-----------|--------|--------|
| No regression | 120 | 120 | 20 | PASS | PASS |
| Minor drop | 110 | 120 | 20 | PASS | PASS |
| Major drop | 90 | 120 | 20 | FAIL | PASS |
| Exact threshold | 100 | 120 | 20 | PASS | PASS |

## Adversarial Inputs

### Special Float Values

| Input | Expected | Result | Status |
|-------|----------|--------|--------|
| NaN in sample | Error or filtered | InvariantViolation | PASS |
| Infinity in sample | Error or filtered | InvariantViolation | PASS |
| Negative frame time | Statistics valid | Handled gracefully | PASS |
| Zero frame time | Division handled | FPS = 0, no panic | PASS |

### Boundary Values

| Input | Expected | Result | Status |
|-------|----------|--------|--------|
| f64::MAX | No overflow | Handled | PASS |
| f64::MIN | No underflow | Handled | PASS |
| u64::MAX timestamp | No overflow | Not tested | SKIP |

## Property-Based Testing

The module includes property-based tests from the existing codebase:

| Property | Description | Status |
|----------|-------------|--------|
| Zoom clamping | Zoom stays in bounds | PASS |
| Coordinate transform | Roundtrip produces finite values | PASS |
| Viewport normalization | Always positive | PASS |

## Concurrency

| Test | Scenario | Result | Status |
|------|----------|--------|--------|
| Sequential benchmarks | Run one after another | No state contamination | PASS |
| Quick benchmark | 5 operations sequentially | All complete | PASS |

## Summary

| Category | Passed | Failed | Skipped |
|----------|--------|--------|---------|
| Stress Tests | 6 | 0 | 1 |
| Invariant Tests | 7 | 0 | 0 |
| Determinism | 3 | 0 | 0 |
| Error Recovery | 5 | 0 | 0 |
| Regression | 4 | 0 | 0 |
| Adversarial | 5 | 0 | 1 |
| **Total** | **30** | **0** | **2** |

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Real rendering slower than simulation | High | High | Add actual render benchmarks |
| Memory usage at 10k nodes | Medium | Medium | Add memory profiling |
| CI flakiness | Medium | Low | Use statistical thresholds |
| GPU driver variance | High | Medium | Document hardware requirements |

## Recommendations

1. Add real rendering benchmarks when pipeline is ready
2. Add memory profiling with allocation tracking
3. Set up hardware-specific baseline comparisons
4. Add long-running stability tests
