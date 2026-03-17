# ADR-012: Red Queen Adversarial Testing

## Status
Accepted

## Date
2026-03-15

## Context
Standard tests verify the happy path. Adversarial testing discovers edge cases, boundary conditions, and failure modes that happy-path tests miss.

## Decision
We will use **Red Queen testing** - an evolutionary adversarial testing methodology where each generation of tests must defeat all previous implementations.

## Methodology

### The Red Queen Principle
> "It takes all the running you can do, to keep in the same place."

Each test generation must:
1. Pass against current implementation
2. Have found at least one bug in previous implementations
3. Be more adversarial than the previous generation

### Test Categories

| Category | Focus |
|----------|-------|
| Boundary | Min/max values, edge of valid domain |
| Malformed | Invalid inputs, corrupted data |
| Concurrency | Race conditions, interleaving |
| Resource | Memory limits, disk full |
| State | Invalid state transitions |

### Generation Process

```
Generation N:
  1. Analyze previous test failures
  2. Generate adversarial test cases
  3. Run against implementation
  4. Record any new failures
  5. Output: red-queen-report.md
```

### Test Case Template

```rust
#[test]
fn adversarial_<category>_<id>() {
    // Adversarial input from generation N
    let input = <extreme_or_malformed_value>;
    
    // Should handle gracefully, not panic
    let result = function_under_test(input);
    
    // Verify: no panic, correct error variant
    assert!(matches!(result, Err(ExpectedError::Variant)));
}
```

## Report Format

```markdown
# Red Queen Report - Generation N

## Tests Generated
- adversarial_boundary_001: f64::MAX coordinate
- adversarial_malformed_002: NaN input handling
- adversarial_state_003: invalid transition

## Failures Found
- <test_name>: <failure_description>

## Previous Generations Defeated
- Generation N-1: 3/3 tests now pass
- Generation N-2: 5/5 tests now pass

## Recommendations
- <recommendation for fixes>
```

## Integration with GO Skill

Red Queen runs at **State 5** of the GO Skill lifecycle:
- After Moon Gate (State 4)
- After QA Execution (State 4.5)
- Before Black Hat Review (State 5.5)

## Rules
- Each generation must be more adversarial than the last
- Record all failures for regression prevention
- Never mark a test as "known failure" - fix or delete
