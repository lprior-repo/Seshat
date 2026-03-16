# ADR-010: Verification & Review Tools

## Status
Accepted

## Date
2026-03-15

## Context
Code quality requires multiple layers of automated verification: formal proofs, adversarial testing, and code review.

## Decision
We will use a layered verification approach with **5 distinct tools**.

---

## 1. Kani (Formal Verification)

**Purpose:** Mathematical proofs for state machines and geometry calculations.

**When Required:**
- State machine transitions
- Geometry calculations (transforms, collision detection)
- Invariant verification
- Panic prevention (division by zero, overflow)

**When NOT Required:**
- Simple CRUD operations
- UI rendering code
- Database I/O
- Tests already covered by proptest

**Command:**
```bash
cargo kani
```

**Proof Template:**
```rust
#[kani::proof]
fn verify_<function>_<property>() {
    let input: f64 = kani::any();
    kani::assume(input.is_finite());
    kani::assume(input.abs() < MAX_VALUE);
    let result = my_function(input);
    assert!(result.is_valid());
}
```

**Existing Harnesses:**
- `diagram_tool/src/geometry/transforms_kani.rs`
- `diagram_tool/src/geometry/operations_kani.rs`

---

## 2. TDD Guard (Test-First Enforcement)

**Purpose:** Blocks implementation without failing tests.

**Installation:**
```bash
npm install -g tdd-guard
cargo install tdd-guard-rust
```

**Test Command:**
```bash
cargo nextest run 2>&1 | tdd-guard-rust --project-root . --passthrough
```

**Rules:**
- No implementation code without a failing test
- Write minimum code to make tests pass
- NEVER bypass tdd-guard

**Claude Code Hooks Required:**
- PreToolUse: `Write|Edit|MultiEdit|TodoWrite` → `tdd-guard`
- UserPromptSubmit → `tdd-guard`
- SessionStart: `startup|resume|clear` → `tdd-guard`

---

## 3. Black Hat Reviewer (Code Quality Gate)
**Purpose:** Ruthless enforcement of architectural constraints.

**When:** After implementation, before merge (GO Skill State 5.5).

**Checks:**
| Check | Description |
|-------|-------------|
| Contract Parity | Implementation matches contract.md |
| Functional Rust Big 6 | itertools, tap, rpds, im, thiserror, anyhow |
| Strict DDD | Scott Wlaschin type-driven design |
| Zero Panics | No unwrap, expect, panic! in source |
| File Size | <300 lines per file |

**Pass Criteria:** 100% compliance required.

**Command:** Invoke `black-hat-reviewer` skill.

---

## 4. Truth Serum (AI Audit)
**Purpose:** Detect AI hallucinations, laziness, and skipped verification steps.

**When:** After implementation, before merge (paired with Black Hat).

**Checks:**
| Check | Description |
|-------|-------------|
| Hallucination | Code that doesn't exist or is fabricated |
| Laziness | Skipped tests, incomplete implementations |
| Contract Violation | Deviations from contract.md |
| Evidence Gaps | Claims without command output |

**Pass Criteria:** 100% compliance required.

**Command:** Invoke `truth-serum` skill.

---

## 5. Red Queen (Adversarial Testing)
**Purpose:** Evolutionary test generation - each generation must defeat all previous generations.

**When:** After QA passes, before Black Hat (GO Skill State 5).

**Process:**
```
Generation N:
  1. Analyze previous test failures
  2. Generate adversarial test cases
  3. Run against implementation
  4. Record any new failures
  5. Output: red-queen-report.md
```

**Test Categories:**
- Boundary values (f64::MAX, MIN, NAN)
- Malformed inputs (empty strings, null pointers)
- Concurrency (race conditions)
- State (invalid transitions)
- Resource (memory limits)

**Command:** Invoke `red-queen` skill.

---

## Verification Pipeline (GO Skill)

```
State 4: Moon Gate (cargo test)
    ↓
State 4.5: QA Enforcer (actual CLI execution)
    ↓
State 5: Red Queen (adversarial tests)
    ↓
State 5.5: Black Hat (code review)
    ↓
State 5.7: Kani (formal proofs)
    ↓
State 6: Repair Loop (if any failures)
```

## Consequences

### Positive
- **Layered defense**: Multiple verification stages catch different bug classes
- **Automated**: No manual review required
- **Deterministic**: Same inputs produce same outputs

### Negative
- **Time**: Full pipeline takes 10-30 minutes per feature
- **Complexity**: Multiple tools to learn and configure
