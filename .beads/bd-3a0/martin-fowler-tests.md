# Martin Fowler Test Patterns: bd-3a0 - Multi-Diagram Session Support

## Overview

This document outlines the test patterns and methodology for verifying multi-diagram session support, following Martin Fowler's testing principles and the Given-When-Then BDD format.

## Test Philosophy

### 1. Isolation
Each test must be independent and not rely on state from other tests. Use fresh session state for each test.

### 2. Determinism
All tests must be deterministic - no flaky tests. Use controlled inputs and verify exact outputs.

### 3. Readability
Tests serve as documentation. Use descriptive names following `given_X_when_Y_then_Z` pattern.

## Test Categories

### Category A: Tab Lifecycle Tests

```rust
// Pattern: State Machine Testing
// Tabs follow a lifecycle: Created -> Active/Inactive -> Closed

#[test]
fn given_fresh_session_when_create_diagram_then_one_session_exists() {
    // Arrange
    let manager = SessionManager::new();

    // Act
    let result = manager.create_diagram("New Diagram");

    // Assert
    assert!(result.is_ok());
    assert_eq!(manager.session_count(), 1);
}

#[test]
fn given_multiple_sessions_when_close_middle_then_order_preserved() {
    // Arrange
    let mut manager = SessionManager::new();
    let id_a = manager.create_diagram("A").unwrap();
    let id_b = manager.create_diagram("B").unwrap();
    let id_c = manager.create_diagram("C").unwrap();

    // Act
    let result = manager.close_session(&id_b);

    // Assert
    assert!(result.is_ok());
    assert_eq!(manager.session_count(), 2);
    assert_eq!(manager.tab_order(), vec![id_a, id_c]);
}
```

### Category B: State Isolation Tests

```rust
// Pattern: Cross-Boundary State Verification
// Verify state doesn't leak between diagram sessions

#[test]
fn given_selection_in_diagram_a_when_switch_to_b_then_selection_isolated() {
    // Arrange
    let mut manager = SessionManager::new();
    let id_a = manager.create_diagram("A").unwrap();
    let id_b = manager.create_diagram("B").unwrap();

    // Add node to A and select it
    manager.with_session(&id_a, |session| {
        session.add_node(Node::default());
        session.select_all();
    });

    // Act
    manager.set_active(&id_b);

    // Assert - B should have empty selection
    manager.with_session(&id_b, |session| {
        assert!(session.selection().is_empty());
    });

    // Assert - A should still have selection
    manager.with_session(&id_a, |session| {
        assert!(!session.selection().is_empty());
    });
}

#[test]
fn given_history_in_diagram_a_when_switch_to_b_then_history_isolated() {
    // Arrange
    let mut manager = SessionManager::new();
    let id_a = manager.create_diagram("A").unwrap();
    let id_b = manager.create_diagram("B").unwrap();

    // Make changes in A
    manager.with_session(&id_a, |session| {
        session.add_node(Node::default());
        session.push_history();
    });

    // Act
    manager.set_active(&id_b);
    let undo_result = manager.with_session(&id_b, |session| session.undo());

    // Assert - B has nothing to undo
    assert!(undo_result.is_none());
}
```

### Category C: Clipboard Integration Tests

```rust
// Pattern: Shared Resource Testing
// Clipboard is shared across diagrams but operations are isolated

#[test]
fn given_copy_in_diagram_a_when_paste_in_b_then_new_ids_generated() {
    // Arrange
    let mut manager = SessionManager::new();
    let id_a = manager.create_diagram("A").unwrap();
    let id_b = manager.create_diagram("B").unwrap();

    let original_node_id = {
        manager.with_session(&id_a, |session| {
            let node = Node::new("Original");
            let id = node.id.clone();
            session.add_node(node);
            session.select_node(&id);
            session.copy_selection();
            id
        })
    };

    // Act
    manager.set_active(&id_b);
    manager.with_active_session(|session| {
        session.paste();
    });

    // Assert - B has node with different ID
    manager.with_session(&id_b, |session| {
        let node_ids: Vec<_> = session.document().nodes.keys().collect();
        assert_eq!(node_ids.len(), 1);
        assert_ne!(node_ids[0], &original_node_id);
    });
}
```

### Category D: Edge Case Tests

```rust
// Pattern: Boundary Condition Testing
// Test limits and edge cases

#[test]
fn given_max_diagrams_when_create_another_then_returns_error() {
    // Arrange
    let mut manager = SessionManager::with_max_sessions(3);
    manager.create_diagram("1").unwrap();
    manager.create_diagram("2").unwrap();
    manager.create_diagram("3").unwrap();

    // Act
    let result = manager.create_diagram("4");

    // Assert
    assert!(matches!(result, Err(SessionError::MaxDiagramsReached)));
}

#[test]
fn given_one_diagram_when_close_it_then_new_empty_created() {
    // Arrange
    let mut manager = SessionManager::new();
    let id = manager.create_diagram("Only").unwrap();

    // Act
    let result = manager.close_session(&id);

    // Assert
    assert!(result.is_ok());
    assert_eq!(manager.session_count(), 1); // New empty created
    assert_ne!(manager.active_session_id(), &id); // Different ID
}

#[test]
fn given_dirty_diagram_when_close_then_prompts_user() {
    // Arrange
    let mut manager = SessionManager::new();
    let id = manager.create_diagram("Dirty").unwrap();
    manager.with_session(&id, |session| {
        session.add_node(Node::default());
        session.set_dirty(true);
    });

    // Act
    let close_result = manager.request_close(&id);

    // Assert
    assert!(close_result.needs_confirmation());
    assert!(close_result.is_dirty());
}
```

### Category E: Performance Tests

```rust
// Pattern: Performance Contract Testing
// Verify operations meet latency requirements

#[test]
fn given_10_diagrams_when_switch_tabs_then_under_16ms() {
    // Arrange
    let mut manager = SessionManager::new();
    let ids: Vec<_> = (0..10)
        .map(|i| manager.create_diagram(&format!("Diagram {}", i)).unwrap())
        .collect();

    // Populate with content
    for id in &ids {
        manager.with_session(id, |session| {
            for _ in 0..100 {
                session.add_node(Node::default());
            }
        });
    }

    // Act & Assert
    for id in &ids[1..] {
        let start = std::time::Instant::now();
        manager.set_active(id);
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_millis() < 16,
            "Tab switch took {}ms, expected < 16ms",
            elapsed.as_millis()
        );
    }
}
```

## Test Data Builders

```rust
/// Builder for creating test sessions
pub struct SessionBuilder {
    name: String,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    dirty: bool,
}

impl SessionBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
            dirty: false,
        }
    }

    pub fn with_nodes(mut self, count: usize) -> Self {
        self.nodes = (0..count).map(|i| Node::new(&format!("Node {}", i))).collect();
        self
    }

    pub fn with_edges(mut self, edges: Vec<(usize, usize)>) -> Self {
        self.edges = edges
            .into_iter()
            .map(|(src, tgt)| Edge::new(&self.nodes[src].id, &self.nodes[tgt].id))
            .collect();
        self
    }

    pub fn dirty(mut self) -> Self {
        self.dirty = true;
        self
    }

    pub fn build(self) -> DiagramSession {
        let mut session = DiagramSession::new(&self.name);
        for node in self.nodes {
            session.add_node(node);
        }
        for edge in self.edges {
            session.add_edge(edge);
        }
        session.set_dirty(self.dirty);
        session
    }
}
```

## Test Organization

```
tests/
  multi_diagram/
    mod.rs              # Test module entry point
    tab_lifecycle.rs    # TAB-* tests
    session_state.rs    # SES-* tests
    clipboard_cross.rs  # Cross-diagram clipboard tests
    performance.rs      # Performance contract tests
    edge_cases.rs       # Boundary conditions
```

## Mocking Strategy

For E2E tests, mock the following:
- File system operations (use in-memory storage)
- User confirmation dialogs (return deterministic responses)
- System clipboard (use test clipboard)

## Assertions Checklist

Each test should verify:
- [ ] Return value/result is correct
- [ ] State changes are as expected
- [ ] No panics or errors (unless testing error cases)
- [ ] Performance within bounds (if applicable)
- [ ] Invariants maintained

## Anti-Patterns to Avoid

1. **Sleep in tests**: Use deterministic synchronization
2. **Shared mutable state**: Each test gets fresh fixtures
3. **Brittle selectors**: Use stable identifiers
4. **Implicit ordering**: Tests must pass in any order
5. **Magic numbers**: All values should be named constants

## Continuous Integration

```yaml
# CI Configuration for multi-diagram tests
test_multi_diagram:
  script:
    - cargo test --package diagram_tool multi_diagram::
  rules:
    - changes:
      - diagram_tool/src/session/**/*.rs
      - diagram_tool/src/session_manager.rs
  timeout: 5m
  retry: 2
```

## Summary

Following these patterns ensures:
- Comprehensive coverage of multi-diagram functionality
- Maintainable test suite
- Clear documentation through tests
- Confidence in refactoring
- Fast feedback in CI
