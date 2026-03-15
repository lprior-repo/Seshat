# ADR-008: AI Development Lifecycle (GO Skill State Machine)

## Status
Accepted

## Date
2026-03-15

## Context
AI agents require a deterministic, fail-closed workflow to produce high-quality code.

## Decision
We will use the **GO Skill Lifecycle** - an 8-state deterministic state machine with strict gates.

## State Machine

| State | Name | Purpose |
|-------|------|---------|
| 0 | Isolation & Calibration | Claim bead, create jj workspace |
| 1 | Contract Synthesis | rust-contract sub-agent → contract.md |
| 2 | Test Plan Review | test-reviewer validates BDD/ATDD |
| 3 | Implementation | functional-rust sub-agent |
| 4 | Moon Gate | moon run :quick, :test, :ci, :e2e |
| 4.5 | QA Execution | qa-enforcer runs actual CLI commands |
| 4.6 | QA Review | PASS → State 5, FAIL → State 3 |
| 5 | Red Queen | Adversarial testing |
| 5.5 | Black Hat Review | Contract Parity, Big 6, DDD |
| 5.7 | Kani | Model checking (or formal argument) |
| 6 | Repair Loop | Fix defects, return to State 4 |
| 7 | Architectural Drift | <300 line files, Scott Wlaschin DDD |
| 8 | Landing | jj git push, bd close, cleanup |

## Functional Rust Big 6

| Crate | Purpose |
|-------|---------|
| itertools | Iterator pipelines |
| tap | Suffix pipelines |
| rpds | Persistent data structures |
| im | Thread-share persistent data |
| thiserror | Domain errors (core) |
| anyhow | Boundary errors (shell) |

## Rules
- ANY claim without command output is INVALID
- Looping > 5 times in State 6 = ABORT
- Orchestrator NEVER implements code directly
