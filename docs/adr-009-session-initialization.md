# ADR-009: Session Initialization Protocol

## Status
Accepted

## Date
2026-03-15

## Context
AI sessions begin without context. Inconsistent initialization leads to forgotten rules.

## Decision
Every AI session MUST begin with the Session Initialization Protocol.

## Step 1: Invoke Functional Rust Skill
ALWAYS invoke the functional-rust skill at session start.

## Step 2: Read Critical Documentation

| Priority | File |
|----------|------|
| 1 | CLAUDE.md |
| 2 | docs/13_LESSONS_LEARNED.md |
| 3 | docs/04_DATA_CALC_ACTIONS.md |
| 4 | docs/adr-008-ai-development-lifecycle.md |

## Step 3: Verify Tool Availability
```bash
moon --version
bd ready --json
jj --version
```

## Step 4: Check for Ready Work
```bash
bd ready --json
```

## Forbidden Actions
- Writing code before reading docs
- Using Task tool or subagents for dev work
- Skipping functional-rust skill invocation
- Starting work without claiming a bead
