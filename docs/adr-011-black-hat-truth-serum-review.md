# ADR-011: Black Hat & Truth Serum Review

## Status
Accepted

## Date
2026-03-15

## Context
AI-generated code can contain hallucinations, skipped verification steps, and violations of architectural constraints. Manual review is insufficient to catch these issues systematically.

## Decision
We will use **two-stage adversarial review** before any merge to main.

## Stage 1: Black Hat Reviewer

### Purpose
Ruthlessly verify architectural constraints.

### Checks

| Check | Description |
|-------|-------------|
| Contract Parity | Implementation matches contract.md exactly |
| Functional Rust Big 6 | Uses itertools, tap, rpds, im, thiserror, anyhow correctly |
| Strict DDD | Scott Wlaschin patterns: parse don't validate, make illegal states unrepresentable |
| Zero Panics | No unwrap(), expect(), panic!() in source |
| Type Encoding | Preconditions enforced at compile-time where possible |
| File Size | All files < 300 lines |

### Passing Criteria
- 100% compliance required
- ANY violation = STATUS: REJECTED
- Output: `defects.md` listing all violations

### Invocation
```
Load black-hat-reviewer skill. Read contract.md and implementation.md. 
Ruthlessly enforce the 5 phases of code review.
```

## Stage 2: Truth Serum

### Purpose
Detect AI hallucinations, laziness, and skipped verification.

### Checks

| Check | Description |
|-------|-------------|
| Hallucination | Code references non-existent functions/modules |
| Laziness | TODO comments, placeholder implementations |
| Skipped Tests | Test coverage gaps vs martin-fowler-tests.md |
| Fake Execution | Claims of "tested" without actual command output |
| Bypass Attempts | Code that circumvents TDD Guard |

### Passing Criteria
- No hallucinations
- No unimplemented contracts
- All tests in martin-fowler-tests.md exist and pass
- Exit code verification for all claims

### Invocation
```
Load truth-serum skill. Audit code for AI hallucinations, laziness, 
or skipped verification steps.
```

## Review Order
1. Run Black Hat Reviewer first
2. If REJECTED → fix defects, re-run
3. If APPROVED → run Truth Serum
4. If Truth Serum finds issues → fix, re-run Black Hat
5. Both PASS → proceed to merge

## Rules
- NEVER skip review stages
- NEVER self-approve - use the skills
- ANY critical issue blocks merge
