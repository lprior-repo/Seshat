bead_id: bd-2wp
bead_title: edge-case-bdd-tests-conflict-resolution
phase: p1
updated_at: 2026-03-02T04:46:00Z

# Contract: BDD Tests for Conflict Resolution Edge Cases

## Overview

This bead adds comprehensive BDD-style tests for conflict resolution edge cases
in the diagram_tool crate. Tests focus on the conflict detection system in
`diagram_tool/src/models/conflict.rs`.

## Scope

### In Scope

1. **Edit Window Expiry Tests**
   - Human edit window expiration after HUMAN_EDIT_WINDOW_SECS (30s)
   - AI operations allowed after edit window expires
   - Edit window refresh on subsequent human edits

2. **Concurrent Human/AI Operations Tests**
   - AI operation rejection when human has active edit on same entity
   - AI operation allowed on different entities during human edit
   - Multiple concurrent human edits on different entities
   - AI operation rejection when human edits affect related entities (edges)

3. **Author Identification Edge Cases**
   - Author ID with "human-" prefix recognized as human
   - Author name containing "human" (case-insensitive) recognized as human
   - Authors without human indicators treated as AI
   - Empty or malformed author fields

4. **Rapid Consecutive Edits Tests**
   - Rapid human edits refresh edit window correctly
   - Idempotency: duplicate operation IDs handled correctly
   - Processed operations cache behavior
   - Cleanup of expired edit windows

### Out of Scope

- BDD/Cucumber framework integration (use native Rust tests)
- Integration tests with external systems
- Performance benchmarks

## Test Specifications

### 1. Edit Window Expiry

```gherkin
Feature: Edit Window Expiry

  Scenario: AI operation allowed after edit window expires
    Given a human edit was registered on "node-1" 31 seconds ago
    And the edit window duration is 30 seconds
    When an AI operation targets "node-1"
    Then the operation should be allowed

  Scenario: Edit window refreshes on new human edit
    Given a human edit was registered on "node-1" 29 seconds ago
    When a new human edit is registered on "node-1"
    Then the edit window should be active for another 30 seconds

  Scenario: Expired edit windows are cleaned up
    Given human edits on multiple entities with varying ages
    When cleanup_expired is called
    Then only active edit windows remain
```

### 2. Concurrent Human/AI Operations

```gherkin
Feature: Concurrent Operations

  Scenario: AI operation rejected during active human edit
    Given a human has an active edit on "node-1"
    When an AI attempts to move "node-1"
    Then the operation should be rejected with HumanPriorityBlock
    And the conflicting entities should include "node:node-1"

  Scenario: AI operation allowed on unrelated entity
    Given a human has an active edit on "node-1"
    When an AI attempts to add "node-2"
    Then the operation should be allowed

  Scenario: AI edge operation rejected when source has human edit
    Given a human has an active edit on "node-source"
    When an AI attempts to connect edge from "node-source" to "node-target"
    Then the operation should be rejected
    And conflicting entities should include "node:node-source"

  Scenario: AI edge operation rejected when target has human edit
    Given a human has an active edit on "node-target"
    When an AI attempts to connect edge from "node-source" to "node-target"
    Then the operation should be rejected
```

### 3. Author Identification Edge Cases

```gherkin
Feature: Author Identification

  Scenario: Author with human- prefix is human
    Given an author with id "human-alice"
    When is_human_author is called
    Then it should return true

  Scenario: Author with Human in name is human
    Given an author with name "Human User"
    When is_human_author is called
    Then it should return true

  Scenario: Author with HUMAN in name (uppercase) is human
    Given an author with name "HUMAN OPERATOR"
    When is_human_author is called
    Then it should return true

  Scenario: AI author without human indicators
    Given an author with id "ai-assistant" and name "AI Assistant"
    When is_human_author is called
    Then it should return false

  Scenario: Author with empty id and non-human name
    Given an author with id "" and name "System"
    When is_human_author is called
    Then it should return false
```

### 4. Rapid Consecutive Edits

```gherkin
Feature: Rapid Consecutive Edits

  Scenario: Duplicate operation ID is idempotent
    Given operation "op-1" has been processed
    When operation "op-1" is evaluated again
    Then it should be allowed (idempotent)

  Scenario: Multiple rapid human edits refresh window
    Given human edit on "node-1" at time T
    When human edits occur at T+10s, T+20s, T+25s
    Then edit window should be active at T+50s

  Scenario: Multiple entities tracked independently
    Given human edits on "node-1" and "node-2"
    When an AI operation targets only "node-3"
    Then the operation should be allowed

  Scenario: Processed operations set tracks all IDs
    Given operations "op-1", "op-2", "op-3" have been processed
    Then all three should be recognized as processed
```

## Acceptance Criteria

1. All test scenarios above are implemented as Rust tests in `conflict.rs`
2. Tests follow naming convention `given_<precondition>_when_<action>_then_<outcome>`
3. All tests pass with `cargo test conflict`
4. No use of `unwrap()` or `expect()` in test assertions - use `assert!` and `assert_eq!`
5. Test coverage for conflict.rs remains at 100%

## Dependencies

- Existing `diagram_tool/src/models/conflict.rs` module
- Existing `diagram_tool/src/models/envelope.rs` for EventEnvelope
- No new external dependencies

## Files to Modify

- `diagram_tool/src/models/conflict.rs` - Add new test cases to existing `#[cfg(test)]` module

## Verification

```bash
cargo test -p diagram_tool conflict
cargo test -p diagram_tool --lib -- conflict
moon run :quick
moon run :test
```
