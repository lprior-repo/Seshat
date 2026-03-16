# ADR-011: TDD Guard Setup

## Status
Accepted

## Date
2026-03-15

## Context
TDD Guard enforces test-first development by blocking code modifications without failing tests.

## Decision
We We will use **tdd-guard** with **tdd-guard-rust** reporter for all Rust development.
## Installation
```bash
# Install TDD Guard
npm install -g tdd-guard
# Install Rust reporter
cargo install tdd-guard-rust
```
## Test Command Format
```bash
cargo nextest run 2>&1 | tdd-guard-rust --project-root . --passthrough
```
## Hook Configuration
TDD Guard requires hooks in Claude Code settings:
```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Write|Edit|MultiEdit|TodoWrite",
        "hooks": [{ "type": "command", "command": "tdd-guard" }]
      }
    ],
    "UserPromptSubmit": [
      { "hooks": [{ "type": "command", "command": "tdd-guard" }] }
    ],
    "SessionStart": [
      {
        "matcher": "startup|resume|clear",
        "hooks": [{ "type": "command", "command": "tdd-guard" }]
      }
    ]
  }
}
```
## Workflow
1. **Write a failing test** that defines the expected behavior
2. **Run test through tdd-guard-rust** to capture failure
3. **Write minimum implementation** to make test pass
4. **Run test again** to verify success
5. **Refactor** if needed (keeping tests green)
## Rules
- **RED phase**: No implementation without failing test
- **GREEN phase**: Write only minimum code to pass
- **REFACTOR phase**: Improve code while keeping tests green
- **No bypass**: Never use `#[allow(dead_code)]` to skip tests
## Integration with GO Skill
| State | TDD Guard Role |
|-------|---------------|
| State 3 (Implementation) | Enforced - no code without failing test |
| State 4 (Moon Gate) | Tests run through reporter |
| State 6 (Repair Loop) | All fixes must test validation |
