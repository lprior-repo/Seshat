# Martin Fowler-Style Behavioral Tests: bd-2qs - Selection Reliability

## Overview

This document specifies behavioral tests for selection functionality following Martin Fowler's testing principles:
- Test behavior, not implementation
- Use domain language
- One assertion per test concept
- Clear Given-When-Then structure

## Selection State Machine Tests

### Feature: Selection State Transitions

```gherkin
Feature: Selection State Management
  As a user
  I want to select and deselect diagram elements
  So that I can perform operations on them

  Background:
    Given a diagram with nodes "A", "B", "C" at positions (100,100), (200,100), (300,100)
    And each node has size 80x40

  Scenario: Single selection replaces previous selection
    Given node "A" is selected
    When I click on node "B" without modifiers
    Then only node "B" should be selected
    And node "A" should not be selected

  Scenario: Shift-click adds to selection
    Given node "A" is selected
    When I shift-click on node "B"
    Then nodes "A" and "B" should be selected

  Scenario: Shift-click on selected node removes it
    Given nodes "A" and "B" are selected
    When I shift-click on node "A"
    Then only node "B" should be selected

  Scenario: Click on empty canvas clears selection
    Given nodes "A" and "B" are selected
    When I click on empty canvas at position (500, 500)
    Then no nodes should be selected
```

### Feature: Marquee Selection

```gherkin
Feature: Marquee (Rectangle) Selection
  As a user
  I want to select multiple nodes by dragging a rectangle
  So that I can quickly select groups of elements

  Background:
    Given a diagram with nodes at positions:
      | id | x   | y   | width | height |
      | A  | 100 | 100 | 80    | 40     |
      | B  | 200 | 100 | 80    | 40     |
      | C  | 300 | 200 | 80    | 40     |

  Scenario: Left-to-right marquee requires full containment
    Given no nodes are selected
    When I drag marquee from (90, 90) to (190, 150)
    Then only node "A" should be selected
    And node "B" should not be selected (not fully contained)

  Scenario: Right-to-left marquee uses intersection
    Given no nodes are selected
    When I drag marquee from (190, 150) to (90, 90)
    Then nodes "A" and "B" should be selected (both intersect)

  Scenario: Marquee across scattered nodes
    Given no nodes are selected
    When I drag marquee from (90, 90) to (400, 250)
    Then all nodes "A", "B", "C" should be selected
```

### Feature: Selection Persistence

```gherkin
Feature: Selection Persists Across Operations
  As a user
  I want my selection to persist when I pan or zoom
  So that I don't lose my work context

  Background:
    Given a diagram with node "A" at (100, 100)
    And node "A" is selected

  Scenario: Selection persists across pan
    When I pan the canvas by (50, 50)
    Then node "A" should still be selected
    And the selection bounding box should update to screen position

  Scenario: Selection persists across zoom
    When I zoom to 200%
    Then node "A" should still be selected
    And the selection bounding box should scale with zoom

  Scenario: Selection persists across undo/redo
    Given I have performed a selection change
    When I undo the selection change
    Then the previous selection should be restored
    When I redo the selection change
    Then the new selection should be restored
```

### Feature: Selection Handles

```gherkin
Feature: Selection Visual Feedback
  As a user
  I want to see visual feedback when items are selected
  So that I know what I'm working with

  Background:
    Given a diagram with node "A" at (100, 100) with size 80x40

  Scenario: Single selection shows resize handles
    When I select node "A"
    Then 8 resize handles should be visible
    And handles should be at corners and edge midpoints
    And a selection bounding box should surround the node

  Scenario: Multi-selection shows unified bounding box
    Given nodes "A" at (100, 100) and "B" at (200, 100)
    When I select both nodes
    Then a single bounding box should encompass both nodes
    And 8 resize handles should be on the unified box

  Scenario: Hover shows affordance before selection
    Given no nodes are selected
    When I hover over node "A"
    Then the node border should change visually
    And the cursor should indicate clickability
```

### Feature: Edge Selection

```gherkin
Feature: Edge (Connector) Selection
  As a user
  I want to select edges independently from nodes
  So that I can modify or delete connections

  Background:
    Given a diagram with nodes "A" and "B"
    And an edge connecting "A" to "B"

  Scenario: Click on edge selects the edge
    Given no items are selected
    When I click on the edge line
    Then the edge should be selected
    And nodes "A" and "B" should not be selected

  Scenario: Edge hit area extends beyond visible line
    When I click 5 pixels away from the edge line
    Then the edge should still be selected
    When I click 10 pixels away from the edge line
    Then the edge should not be selected
```

### Feature: Selection With Modifiers

```gherkin
Feature: Modifier Keys Affect Selection
  As a user
  I want modifier keys to change selection behavior
  So that I have fine control over what's selected

  Background:
    Given a diagram with nodes "A", "B", "C" in a row

  Scenario: Ctrl/Cmd+A selects all
    Given no nodes are selected
    When I press Ctrl+A (or Cmd+A on Mac)
    Then all nodes should be selected

  Scenario: Alt-click selects parent container
    Given node "B" is inside subgraph "Group1"
    When I alt-click on node "B"
    Then "Group1" should be selected
    And node "B" should not be selected

  Scenario: Right-click on unselected selects first
    Given node "A" is selected
    When I right-click on node "B"
    Then node "B" should be selected
    And node "A" should not be selected
    And a context menu should appear
```

### Feature: Selection Constraints

```gherkin
Feature: Selection Constraints and Edge Cases
  As a user
  I want selection to respect element states
  So that locked or hidden elements behave correctly

  Scenario: Locked nodes cannot be selected
    Given node "A" has locked=true
    When I click on node "A"
    Then node "A" should not be selected

  Scenario: Hidden nodes are not hit-testable
    Given node "A" is hidden
    And node "B" is visible behind "A"
    When I click on "A"'s position
    Then node "B" should be selected
    And node "A" should not be selected

  Scenario: Drag threshold prevents accidental moves
    Given node "A" is selected
    When I drag node "A" by 2 pixels
    Then the node should not move
    And the selection should remain
```

### Feature: Selection Geometry

```gherkin
Feature: Selection Geometry Accuracy
  As a system
  I want selection bounds to match element geometry
  So that visual feedback is accurate

  Scenario: Selection bounds match node bounds
    Given node "A" at (100, 100) with size 80x40
    When I select node "A"
    Then selection bounds should be (100, 100, 80, 40)

  Scenario: Selection handles positioned at corners
    Given node "A" at (100, 100) with size 80x40 is selected
    Then the SE handle should be near (180, 140)
    And the NW handle should be near (100, 100)

  Scenario: Negative coordinates handled correctly
    Given node "A" at (-100, -50) with size 80x40
    When I select node "A"
    Then selection bounds should be (-100, -50, 80, 40)
    And handles should be positioned correctly
```

### Feature: Selection Timing

```gherkin
Feature: Selection Timing and Thresholds
  As a system
  I want selection to respect timing thresholds
  So that different interaction patterns are distinguished

  Scenario: Long press selects without drag
    Given node "A" exists
    When I press on node "A" and hold for 100ms without moving
    Then node "A" should be selected
    And no drag operation should start

  Scenario: Double-click enters edit mode
    Given node "A" with label "My Node"
    When I double-click on node "A" within 500ms
    Then node "A" should be in edit mode
    And the label text should be editable

  Scenario: Drag threshold distinguishes click from drag
    Given node "A" is at (100, 100)
    When I press on node "A" and move 2 pixels
    Then node "A" should not move (below threshold)
    When I press on node "A" and move 5 pixels
    Then node "A" should move (above threshold)
```

## Property-Based Tests

### Property: Selection Is Always Valid

```rust
// For any document state and any sequence of selection operations:
// 1. selected_items is always a subset of document node/edge IDs
// 2. Selection state is serializable
// 3. Selection never contains duplicates
```

### Property: Selection Is Deterministic

```rust
// Given the same document state and same user input:
// Selection result is always identical
```

### Property: Selection Bounded by Document

```rust
// For any selection operation:
// |selected_items| <= |document.nodes| + |document.edges|
```

## Performance Tests

### Scenario: Large document selection performance
```
Given a document with 3000 nodes
When I perform marquee selection of 500 nodes
Then the operation should complete within 100ms
And the UI should remain responsive
```

### Scenario: Selection visual update performance
```
Given 500 nodes are selected
When I drag the selection
Then visual feedback should update at 120 FPS
And frame time should not exceed 8.33ms
```

## Test Coverage Matrix

| Test ID | Rust Unit | E2E Playwright | Category |
|---------|-----------|----------------|----------|
| SEL-001 | Yes | Yes | Basic |
| SEL-002 | Yes | Yes | Basic |
| SEL-003 | Yes | Yes | Marquee |
| SEL-004 | Yes | Yes | Basic |
| SEL-005 | Yes | Yes | Marquee |
| SEL-006 | No | Yes | Visual |
| SEL-007 | No | Yes | Handles |
| SEL-008 | No | Yes | Touch |
| SEL-009 | No | Yes | Threshold |
| SEL-010 | No | Yes | Context |
| SEL-011 | No | Yes | Modifier |
| SEL-012 | No | Yes | Constraint |
| SEL-013 | No | Yes | Constraint |
| SEL-014 | No | Yes | Context |
| SEL-015 | No | TBD | Edge |
| SEL-016 | No | TBD | Keyboard |
| SEL-017 | No | TBD | Z-Order |
| SEL-018 | No | TBD | Overlap |
| SEL-019 | No | TBD | Group |
| SEL-020 | No | TBD | Visual |
| SEL-021 | No | Yes | Geometry |
| SEL-022 | No | Yes | Timing |
| SEL-023 | No | Yes | Timing |
| SEL-024 | No | Yes | Persistence |
| SEL-025 | No | Yes | Marquee |
