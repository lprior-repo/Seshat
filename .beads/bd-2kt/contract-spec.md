# Contract Specification: bd-2kt - History Undo/Redo Reliability

## Overview

**Bead ID**: bd-2kt
**Title**: history: Fix undo/redo reliability (HIS-001 to HIS-013)
**Priority**: P1
**Type**: feature

## Contract

### Precondition
- The `History` struct exists at `diagram_tool/src/history.rs`
- The `DiagramDocument` model is defined with all required fields
- Persistent data structures (rpds) are available for immutable history stacks

### Postcondition
- All 13 HIS test cases (HIS-001 to HIS-013) pass
- Undo/redo operations are reliable and deterministic
- History stack is bounded at 100 entries
- Zero unwrap/panic in production code paths

## Test Cases (HIS-001 to HIS-013)

### HIS-001: Undo Move
| Aspect | Description |
|--------|-------------|
| Given | A node at position (100, 100) |
| When | Node moved to (200, 200) and undo is called |
| Then | Node position is restored to (100, 100) |

### HIS-002: Redo Move
| Aspect | Description |
|--------|-------------|
| Given | An undone move operation |
| When | Redo is called |
| Then | Node position is restored to (200, 200) |

### HIS-003: Drag Creates One Entry
| Aspect | Description |
|--------|-------------|
| Given | A node being dragged |
| When | Drag gesture completes |
| Then | Single history entry created (not per-frame) |

### HIS-004: Group Undo
| Aspect | Description |
|--------|-------------|
| Given | Nodes grouped into a subgraph |
| When | Undo is called |
| Then | Group removed and original parent relationships restored |

### HIS-005: Reparent Undo
| Aspect | Description |
|--------|-------------|
| Given | A node reparented from parent A to parent B |
| When | Undo is called |
| Then | Original parent relationship restored |

### HIS-006: Connector Create Undo
| Aspect | Description |
|--------|-------------|
| Given | Two nodes connected by an edge |
| When | Undo is called |
| Then | Edge is removed |

### HIS-007: Style Change Undo
| Aspect | Description |
|--------|-------------|
| Given | Node style changed from Box to Dashed |
| When | Undo is called |
| Then | Original style (Box) is restored |

### HIS-008: Text Edit Single Entry
| Aspect | Description |
|--------|-------------|
| Given | Node label being edited |
| When | Text edit is committed |
| Then | Single history entry created |

### HIS-009: Drag Gesture Single Entry
| Aspect | Description |
|--------|-------------|
| Given | A drag gesture in progress |
| When | Drag completes |
| Then | Single history entry for entire gesture |

### HIS-010: Camera State Unchanged
| Aspect | Description |
|--------|-------------|
| Given | Document with camera position (50, 75) |
| When | Undo/redo is called |
| Then | Camera state restored from pushed state |

### HIS-011: Push Clears Redo Stack
| Aspect | Description |
|--------|-------------|
| Given | History with redo entries |
| When | New state is pushed |
| Then | Redo stack is cleared |

### HIS-012: Multiple Undos
| Aspect | Description |
|--------|-------------|
| Given | History with states A, B, C |
| When | Undo is called twice |
| Then | Walks back through B to A |

### HIS-013: Redo After Multiple Undos
| Aspect | Description |
|--------|-------------|
| Given | History after multiple undos |
| When | Redo is called |
| Then | Walks forward correctly through states |

## Invariants

1. **I1**: History stack is bounded at 100 entries
2. **I2**: Push clears redo stack
3. **I3**: Undo returns most recent state first (LIFO)
4. **I4**: Redo returns states in correct order
5. **I5**: No panics on empty stack operations

## Implementation Requirements

### Code Quality
- Zero `unwrap()` in production code
- Zero `expect()` in production code
- Zero `panic!()` in production code
- All public functions return `Option` or `Result`

### Performance
- Undo/redo latency < 50ms (per architecture spec)
- Memory bounded by 100 entries

## Verification

All tests must pass:
```bash
cargo test --package diagram_tool history::
```

Expected: 51 tests pass (including HIS-001 to HIS-013)
