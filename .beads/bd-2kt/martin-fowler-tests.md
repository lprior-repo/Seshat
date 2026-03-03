# Martin Fowler-Style BDD Tests: bd-2kt History

## Overview

These tests follow Martin Fowler's BDD specification style, emphasizing:
- Given-When-Then structure
- Business-readable test names
- Clear separation of context, action, and outcome

---

## Feature: Undo Operations

### Scenario: Undo node position change
**Given** a diagram with a node at position (100, 100)
**And** the node has been moved to position (200, 200)
**And** the previous state was saved to history
**When** the user triggers an undo operation
**Then** the node should be at position (100, 100)
**And** the undo stack should have one less entry
**And** the redo stack should have one more entry

### Scenario: Undo node resize
**Given** a diagram with a node of size (80, 40)
**And** the node has been resized to (160, 80)
**And** the previous state was saved to history
**When** the user triggers an undo operation
**Then** the node should have size (80, 40)
**And** the exact dimensions should be restored without floating-point drift

### Scenario: Undo node rotation
**Given** a diagram with a node at rotation 0 degrees
**And** the node has been rotated to 45 degrees
**And** the rotation is stored in metadata
**And** the previous state was saved to history
**When** the user triggers an undo operation
**Then** the node rotation should be 0 degrees

### Scenario: Undo group creation
**Given** a diagram with nodes A and B
**And** the nodes have been grouped into subgraph G
**When** the user triggers an undo operation
**Then** subgraph G should not exist
**And** nodes A and B should have no parent

### Scenario: Undo node reparenting
**Given** a diagram with node C as child of parent P1
**And** node C has been reparented to parent P2
**And** the previous state was saved to history
**When** the user triggers an undo operation
**Then** node C should be a child of parent P1

### Scenario: Undo edge creation
**Given** a diagram with nodes A and B
**And** an edge has been created connecting A to B
**And** the previous state was saved to history
**When** the user triggers an undo operation
**Then** no edges should exist in the document

### Scenario: Undo style change
**Given** a diagram with a node having style "Box"
**And** the node style has been changed to "Dashed"
**And** the previous state was saved to history
**When** the user triggers an undo operation
**Then** the node should have style "Box"

---

## Feature: Redo Operations

### Scenario: Redo after undo
**Given** a node has been moved from (100, 100) to (200, 200)
**And** an undo has been performed
**When** the user triggers a redo operation
**Then** the node should be at position (200, 200)

### Scenario: Redo chain integrity
**Given** history has states A, B, C, D
**And** three undos have been performed (now at A)
**When** the user triggers redo operations
**Then** states should be restored in order: B, C, D
**And** the redo chain should maintain correct order

---

## Feature: Transaction Boundaries

### Scenario: Text edit creates single history entry
**Given** a diagram with a node labeled "Original"
**And** the initial state is saved to history
**When** the user changes the label to "Modified"
**And** the new state is pushed to history
**Then** the undo stack should have exactly 2 entries
**And** a single undo should restore "Original"

### Scenario: Drag gesture creates single history entry
**Given** a diagram with a node at position (100, 100)
**And** the initial state is saved to history
**When** the user completes a drag gesture moving to (150, 150)
**And** the final state is pushed to history
**Then** the undo stack should have exactly 2 entries
**And** a single undo should restore position (100, 100)

---

## Feature: History Stack Management

### Scenario: Push clears redo stack
**Given** history has undo entries and redo entries
**When** a new state is pushed
**Then** the redo stack should be empty
**And** the new state should be at the top of the undo stack

### Scenario: History bounded at 100 entries
**Given** 105 states have been pushed to history
**When** the history is accessed
**Then** the undo stack should contain exactly 100 entries
**And** the most recent 100 states should be preserved

### Scenario: Multiple undos walk back correctly
**Given** history has states at positions 100, 200, 300
**When** the user performs two undos
**Then** the document should show position 100
**And** the redo stack should have two entries

---

## Feature: Edge Cases

### Scenario: Undo on empty history
**Given** a fresh history with no entries
**When** the user triggers an undo operation
**Then** the operation should return None
**And** no error should occur

### Scenario: Redo without prior undo
**Given** a history with only push operations
**When** the user triggers a redo operation
**Then** the operation should return None
**And** no error should occur

### Scenario: Camera state in document
**Given** a document with camera at (50, 75), zoom 1.5
**And** a state is pushed to history
**When** undo is performed
**Then** the camera state should be from the pushed state

---

## Feature: Inverse Properties

### Scenario: Exact position restoration
**Given** a node at position (123.45, 678.90)
**And** moved to (999.99, 111.11)
**When** undo is performed
**Then** position should be exactly (123.45, 678.90)
**And** floating-point drift should be < 1e-10

### Scenario: Exact dimension restoration
**Given** a node with dimensions (150.75, 200.25)
**And** resized to (50.5, 75.5)
**When** undo is performed
**Then** dimensions should be exactly (150.75, 200.25)
**And** floating-point drift should be < 1e-10
