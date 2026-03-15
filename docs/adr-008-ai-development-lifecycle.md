# ADR-008: AI Development Lifecycle (GO Skill State Machine)

## Status
Accepted

## Date
2026-03-15

## Context
AI agents require a deterministic, fail-closed workflow to produce high-quality code. Ad-hoc processes lead to inconsistent quality, missed validation steps, and unmaintainable code.

## Decision
We will use the **GO Skill Lifecycle** - an 8-state deterministic state machine with strict gates.

## State Machine Definition

```
┌─────────────────────────────────────────────────────────────────┐
│                     STATE 0: ISOLATION & CALIBRATION            │
│  - Claim bead via bd                                            │
│  - Create isolated jj workspace                                 │
│  - Initialize .beads/<id>/STATE.md                              │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                     STATE 1: CONTRACT SYNTHESIS                 │
│  - Launch rust-contract sub-agent                               │
│  - Output: contract.md, martin-fowler-tests.md                  │
│  - GATE: Files must exist or ABORT                              │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                     STATE 2: TEST PLAN REVIEW                   │
│  - Launch test-reviewer sub-agent                               │
│  - Validates against Testing Trophy, BDD, ATDD                  │
│  - GATE: STATUS: APPROVED or loop back to STATE 1               │
│  - Max retries: 3                                               │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                     STATE 3: IMPLEMENTATION                     │
│  - Launch functional-rust sub-agent                             │
│  - Enforce Data→Calc→Actions, zero panics/unwrap/mut            │
│  - Output: implementation.md                                    │
│  - GATE: File must exist or ABORT                               │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                     STATE 4: MOON GATE                          │
│  - Run: moon run :quick, :test, :ci, :e2e                       │
│  - GATE: All GREEN or loop to STATE 3 with error log            │
│  - Max retries: 2                                               │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                   STATE 4.5: QA EXECUTION                       │
│  - Launch qa-enforcer sub-agent                                 │
│  - Execute actual CLI commands, verify behavior                 │
│  - Output: qa-report.md                                         │
│  - Max retries: 5                                               │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                     STATE 4.6: QA REVIEW                        │
│  - Review qa-report.md                                          │
│  - GATE: PASS → STATE 5, FAIL → STATE 3                         │
│  - Max retries: 5                                               │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                STATE 5: ADVERSARIAL REVIEW (RED QUEEN)          │
│  - Launch red-queen sub-agent                                   │
│  - Generate adversarial test cases                              │
│  - Output: red-queen-report.md                                  │
│  - Max retries: 5                                               │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                STATE 5.5: BLACK HAT CODE REVIEW                 │
│  - Launch black-hat-reviewer sub-agent                          │
│  - Enforce: Contract Parity, Functional Rust Big 6, Strict DDD  │
│  - Output: defects.md (if any)                                  │
│  - GATE: STATUS: APPROVED → STATE 5.7, REJECTED → STATE 6       │
│  - Max retries: 5                                               │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                STATE 5.7: KANI MODEL CHECKING                   │
│  - Option A: Run cargo kani on critical state machines          │
│  - Option B: Formal argument to skip (must be justified)        │
│  - Output: kani-report.md or kani-justification.md              │
│  - Max retries: 5                                               │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                     STATE 6: THE REPAIR LOOP                    │
│  - Launch functional-rust sub-agent with defects.md             │
│  - Edit source files to fix ALL defects                         │
│  - GATE: Return to STATE 4 (re-run all gates)                   │
│  - HARD LIMIT: > 5 loops = ABORT                                │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                STATE 7: ARCHITECTURAL DRIFT & POLISH            │
│  - Launch architectural-drift sub-agent                         │
│  - Enforce <300 line file limit, Scott Wlaschin DDD             │
│  - GATE: STATUS: PERFECT → STATE 8, REFACTORED → STATE 4        │
│  - Max drift loops: 5                                           │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                STATE 8: LANDING AND CLEANUP                     │
│  - jj git fetch, rebase onto main@origin                        │
│  - jj git push --bookmark main                                  │
│  - bd close <bead-id>                                           │
│  - bd sync                                                       │
│  - jj workspace forget, rm -rf workspace                        │
│  - VERIFY: workspace deleted, directory gone                    │
└─────────────────────────────────────────────────────────────────┘
```

## Required Skills (Sub-Agents)

| State | Skill | Purpose |
|-------|-------|---------|
| 1 | `rust-contract` | Design-by-contract specification |
| 2 | `test-reviewer` | Validate test plan against BDD/ATDD |
| 3, 6 | `functional-rust` | Implementation with zero panics/unwrap/mut |
| 4.5 | `qa-enforcer` | Execute actual CLI commands |
| 5 | `red-queen` | Adversarial testing |
| 5.5 | `black-hat-reviewer` | Contract Parity, Big 6, DDD enforcement |
| 7 | `architectural-drift` | File size limits, DDD refactoring |

## Functional Rust Big 6 (Core Libraries)

| Crate | Purpose | Tier |
|-------|---------|------|
| `itertools` | Iterator pipelines | core+shell |
| `tap` | Suffix pipelines (pipe/tap/conv) | core+shell |
| `rpds` | Persistent data structures (default) | core |
| `im` | Persistent data structures (thread-share) | core (Arc) |
| `thiserror` | Domain errors | core |
| `anyhow` | Boundary errors | shell |

## Consequences

### Positive
- **Deterministic quality** - Every feature passes the same gates
- **Fail-closed** - Silent errors are impossible
- **Auditable** - Every state produces artifacts
- **Recoverable** - STATE.md enables crash recovery

### Negative
- **Overhead** - Simple changes require full pipeline
- **Complexity** - Multiple sub-agents to coordinate

### Rules
- ANY claim without command output and exit code is INVALID
- If looping > 5 times in STATE 6, ABORT the bead
- The orchestrator NEVER implements code directly
