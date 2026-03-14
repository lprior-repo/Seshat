# Contract Specification: EDG-032 to EDG-035 Arrowhead Styles

## Context

- **Feature**: Edge terminal shapes (none/arrow/diamond) - new user-facing enum
- **Domain terms**:
  - `TerminalShape` - NEW enum representing user-facing terminal at edge endpoints (`None`, `Arrow`, `Diamond`)
  - `ArrowType` - EXISTING canonical enum (`Default`, `Sharp`, `Curved`, `Step`, `Straight`)
  - Edge - a directed connection between two nodes with optional terminal shapes
- **Assumptions**:
  - Terminal shapes apply to both source and target endpoints independently
  - Default behavior is to show arrow terminal on target end
  - Serialization must handle both legacy string values and new canonical enum
- **Open questions**: None - domain fully specified by persistence_compat.rs and existing ArrowType mapping

## Preconditions

- [P1] `TerminalShape::None` requires `directed: true` to have visual effect (otherwise no terminal shown)
- [P2] `TerminalShape::Arrow` must map to `ArrowType::Default` in canonical form
- [P3] `TerminalShape::Diamond` must map to `ArrowType::Step` in canonical form
- [P4] Valid string representations: "none", "arrow", "diamond" (case-insensitive)

## Postconditions

- [Q1] Edge with `TerminalShape::None` serializes with `arrow_type: "sharp"` (canonical)
- [Q2] Edge with `TerminalShape::Arrow` serializes with `arrow_type: "default"` (canonical)
- [Q3] Edge with `TerminalShape::Diamond` serializes with `arrow_type: "step"` (canonical)
- [Q4] Round-trip: deserialize legacy "arrowhead" key, serialize back, preserves visual appearance

## Invariants

- [I1] `ArrowType` enum remains the canonical storage format in `DiagramDocument`
- [I2] Terminal shape mapping is bijective: legacy string → canonical → legacy string (lossless)
  - Mapping table:
    - "none" ↔ `ArrowType::Sharp`
    - "arrow" ↔ `ArrowType::Default`
    - "diamond" ↔ `ArrowType::Step`
    - "open" ↔ `ArrowType::Straight`
    - "circle" ↔ `ArrowType::Curved`
- [I3] Edge bounds calculation accounts for terminal shape size differences (none=0, arrow=standard, diamond=standard)

## Error Taxonomy

- `Error::InvalidTerminalShape` - when string value does not match any known terminal shape variant
- `Error::InvalidArrowType` - when ArrowType enum parsing fails
- `Error::PreconditionViolation` - when P1-P4 are violated (returned as Result, not panic)

## Contract Signatures

```rust
/// Normalize legacy arrowhead string to canonical ArrowType
fn normalize_terminal_shape(value: &str) -> Result<ArrowType, Error>;

/// Serialize TerminalShape to legacy string for UI compatibility
fn terminal_shape_to_legacy(shape: TerminalShape) -> &'static str;

/// Parse either legacy "arrowhead" or canonical "arrowType" string
fn parse_terminal_input(value: &str) -> Result<TerminalShape, Error>;

/// Validate terminal shape is appropriate for edge directionality
fn validate_terminal_for_direction(shape: TerminalShape, directed: bool) -> Result<(), Error>;

/// Bijective: convert ArrowType (canonical) to TerminalShape (user-facing)
fn arrow_type_to_terminal_shape(arrow_type: ArrowType) -> TerminalShape;
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: TerminalShape::None with directed=false | Runtime validation | `Result<T, Error::PreconditionViolation>` |
| P2: Arrow terminal maps to Default | Compile-time | `TerminalShape::Arrow` → const mapping |
| P3: Diamond terminal maps to Step | Compile-time | `TerminalShape::Diamond` → const mapping |
| P4: Valid string values | Runtime parser | `parse_terminal_input()` Result error |

## Violation Examples (REQUIRED)

- VIOLATES P1: `validate_terminal_for_direction(TerminalShape::None, false)` → should produce `Err(Error::PreconditionViolation("TerminalShape::None has no effect when directed=false"))`
- VIOLATES P2: `normalize_terminal_shape("arrow")` returning anything other than `ArrowType::Default` → should produce `Err(Error::InvalidTerminalShape)`
- VIOLATES P3: `normalize_terminal_shape("diamond")` returning anything other than `ArrowType::Step` → should produce `Err(Error::InvalidTerminalShape)`
- VIOLATES P4: `parse_terminal_input("invalid")` → should produce `Err(Error::InvalidTerminalShape("invalid".into()))`
- VIOLATES Q1: Deserialize `{arrow_type: "sharp"}` and check canonical output is NOT "sharp" → should fail test
- VIOLATES Q2: Deserialize edge with legacy "arrowhead": "arrow" → serialize → should have `arrow_type: "default"`
- VIOLATES Q3: Deserialize edge with legacy "arrowhead": "diamond" → serialize → should have `arrow_type: "step"`
- VIOLATES Q4: Full round-trip `{arrowhead: "diamond"}` → deserialize → serialize → `{arrow_type: "step"}` (not preserved)
- VIOLATES I2: "none" → ArrowType::Sharp → terminal_shape_to_legacy() returns anything other than "none"

## Ownership Contracts

- `terminal_shape_to_legacy()` takes `&self` (shared borrow) - read-only, no mutation
- All parsing functions take `&str` (shared reference) - no ownership transfer
- No `&mut` parameters in this contract - mutation is not part of terminal shape handling

## Non-goals

- [ ] Changing ArrowType to store TerminalShape directly (keeps ArrowType as canonical)
- [ ] Adding new terminal shapes beyond none/arrow/diamond (future work)
- [ ] Rendering/visual implementation (UI layer concern)
- [ ] Bounds calculation changes (covered by geo_033 tests)

---

## Key Clarifications from Test-Defects.md

### Q1 Ambiguity Resolution
**Q: Why does TerminalShape::None serialize as "sharp"?**

A: This is a **legacy compatibility** decision:
- `ArrowType::Sharp` visually renders as no arrow (just a vertex)
- The legacy format used "none" for this visual state
- When deserializing old files with `arrow_type: "sharp"`, we preserve that value
- The bijective mapping ensures: "none" ↔ Sharp ↔ "none" (lossless)

**Mapping Table (I2):**
| Legacy String | ArrowType (Canonical) | TerminalShape (User-Facing) |
|---|---|---|
| "none" | Sharp | None |
| "arrow" | Default | Arrow |
| "diamond" | Step | Diamond |
| "open" | Straight | (not exposed) |
| "circle" | Curved | (not exposed) |

### Bijective Invariant (I2) Clarification
The invariant "lossless" means:
1. Any legacy string parses to an ArrowType
2. That ArrowType serializes back to a string
3. That string, when reparsed, produces the same TerminalShape
4. The visual appearance is preserved (this is what matters to users)
