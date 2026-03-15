# ADR-009: Session Initialization Protocol

## Status
Accepted

## Date
2026-03-15

## Context
AI sessions begin without context. Inconsistent initialization leads to forgotten rules, missed constraints, and inconsistent code quality.

## Decision
Every AI session MUST begin with the **Session Initialization Protocol**.

## Protocol Definition

### Step 1: Invoke Functional Rust Skill
```
ALWAYS invoke the functional-rust skill at session start.
```

This loads:
- Data → Calculations → Actions hierarchy
- Zero panics/unwrap/mut rules
- Functional Rust Big 6 libraries
- Clippy enforcement rules
- Quality gates

### Step 2: Read Critical Documentation
The AI MUST read these files before writing ANY code:

| Priority | File | Purpose |
|----------|------|---------|
| 1 | `CLAUDE.md` | Root constraints and mandates |
| 2 | `docs/13_LESSONS_LEARNED.md` | Dioxus WASM constraints, TDD rules |
| 3 | `docs/04_DATA_CALC_ACTIONS.md` | Functional Rust pattern |
| 4 | `docs/adr-008-ai-development-lifecycle.md` | GO Skill state machine |

### Step 3: Verify Tool Availability
```bash
moon --version      # Build system
bd ready --json     # Issue tracking
jj --version        # Version control
codanna serve       # Code intelligence (background)
```

### Step 4: Check for Ready Work
```bash
bd ready --json
```

## Forbidden Actions During Initialization

- ❌ Writing implementation code before reading docs
- ❌ Using `Task` tool or subagents for development work
- ❌ Skipping the functional-rust skill invocation
- ❌ Starting work without claiming a bead

## Consequences

### Positive
- **Consistent context** - Every session starts with same knowledge
- **Constraint awareness** - Dioxus WASM rules never forgotten
- **Process adherence** - GO Skill lifecycle always followed

### Negative
- **Startup overhead** - 2-3 minutes before first code
- **Token cost** - Reading docs consumes context window
