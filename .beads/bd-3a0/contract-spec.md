# Contract Specification: bd-3a0 - Multi-Diagram Session Support

## Metadata
- **Bead ID**: bd-3a0
- **Title**: multi-diagram: Support for multiple diagrams/tabs in a single session
- **Priority**: P1
- **Type**: feature
- **Created**: 2026-03-03
- **Dependencies**: bd-2qs (selection), bd-139 (clipboard), bd-2kt (history), bd-2cy (multi-select)

## Overview

This contract specifies the requirements for implementing multi-diagram session support, allowing users to work with multiple diagrams simultaneously within a single application session. Each diagram operates in its own tab with isolated state while sharing common session resources.

## Core Requirements

### 1. Session Management

**Requirement**: The application must support multiple concurrent diagrams within a single session.

**Implementation**:
- `SessionManager` coordinates multiple diagram sessions
- Each diagram has its own `DiagramSession` containing document, history, and UI state
- Session-level resources (clipboard, theme) are shared across diagrams

### 2. Tab Management

**Requirement**: Users can create, switch, close, and reorder diagram tabs.

**Implementation**:
- `TabManager` handles tab lifecycle operations
- Tabs display diagram name with dirty indicator (*)
- Tab close prompts for unsaved changes
- Tab reordering via drag-and-drop

### 3. Diagram Isolation

**Requirement**: Each diagram maintains completely isolated state.

**Implementation**:
- Separate `DiagramDocument` per tab
- Separate `History` stack per tab
- Separate `EditorState` (viewport, selection, tool mode) per tab
- Cross-diagram operations (clipboard) work correctly

## Test Case Specifications

### TAB-001: Create New Diagram
**Given**: Application running with at least one diagram
**When**: User clicks "New Diagram" button or uses Ctrl/Cmd+N
**Then**: New diagram tab is created with default name
**Contract**:
- Precondition: Session is initialized
- Postcondition: `sessions.len() == previous + 1`
- Invariant: Active tab switches to new diagram

### TAB-002: Switch Between Diagrams
**Given**: Multiple diagram tabs open
**When**: User clicks on a different tab
**Then**: Active diagram switches, all state preserved
**Contract**:
- Precondition: Target tab exists
- Postcondition: `active_session_id == target_id`
- Invariant: Previous diagram state unchanged

### TAB-003: Close Diagram Tab
**Given**: Multiple diagram tabs open
**When**: User clicks close button on tab
**Then**: Tab closes, switches to adjacent tab
**Contract**:
- Precondition: At least 2 tabs open
- Postcondition: `sessions.len() == previous - 1`
- Invariant: If dirty, prompts user to save/discard/cancel

### TAB-004: Close Last Tab
**Given**: Only one diagram tab open
**When**: User clicks close button
**Then**: Creates new empty diagram (never zero tabs)
**Contract**:
- Postcondition: `sessions.len() == 1`
- Invariant: New diagram created automatically

### TAB-005: Reorder Tabs
**Given**: Multiple diagram tabs
**When**: User drags tab to new position
**Then**: Tab order updated
**Contract**:
- Postcondition: Tab order matches drag result
- Invariant: Active tab unchanged unless dragged

### TAB-006: Tab Dirty State
**Given**: Diagram with unsaved changes
**Then**: Tab shows dirty indicator (*)
**Contract**:
- Postcondition: `tab.title == "Diagram Name*"` when dirty
- Invariant: Dirty state clears on save

### TAB-007: Diagram Name in Tab
**Given**: Diagram with custom name
**When**: User renames diagram
**Then**: Tab title updates immediately
**Contract**:
- Postcondition: `tab.title == new_name + (dirty ? "*" : "")`
- Invariant: Name persists across sessions

### TAB-008: Keyboard Navigation
**Given**: Multiple diagram tabs
**When**: User presses Ctrl/Cmd+Tab or Ctrl/Cmd+Shift+Tab
**Then**: Switches to next/previous tab
**Contract**:
- Postcondition: Active tab changes in order
- Invariant: Wraps around at boundaries

### TAB-009: Tab Context Menu
**Given**: Right-click on tab
**When**: Context menu appears
**Then**: Options for Close, Close Others, Close to Right, Rename
**Contract**:
- Postcondition: Selected action executes
- Invariant: Cancel does nothing

### TAB-010: Tab Middle-Click Close
**Given**: Middle mouse button click on tab
**When**: User middle-clicks tab
**Then**: Tab closes (same as close button)
**Contract**:
- Postcondition: Same as TAB-003
- Invariant: Works even if tab not active

### SES-001: Session Initialization
**Given**: Application starts
**When**: Session manager initializes
**Then**: One default diagram created
**Contract**:
- Postcondition: `sessions.len() == 1`
- Invariant: Default diagram has unique ID

### SES-002: Clipboard Cross-Diagram
**Given**: Content copied in Diagram A
**When**: User switches to Diagram B and pastes
**Then**: Content pastes with new IDs
**Contract**:
- Precondition: Clipboard has content
- Postcondition: Pasted content in Diagram B
- Invariant: Clipboard content preserved across tab switches

### SES-003: History Per-Diagram
**Given**: Operations performed in Diagram A
**When**: User switches to Diagram B
**Then**: Diagram B has its own history stack
**Contract**:
- Postcondition: Undo in B affects only B
- Invariant: A's history preserved but inactive

### SES-004: Viewport Per-Diagram
**Given**: Diagram A zoomed to 200%, Diagram B at 100%
**When**: User switches between them
**Then**: Each maintains its own viewport
**Contract**:
- Postcondition: Zoom/pan preserved per diagram
- Invariant: No viewport state leakage

### SES-005: Selection Per-Diagram
**Given**: Items selected in Diagram A
**When**: User switches to Diagram B
**Then**: Diagram B has its own selection state
**Contract**:
- Postcondition: Selection cleared/preserved per diagram
- Invariant: Selection doesn't leak across diagrams

### SES-006: Tool Mode Per-Diagram
**Given**: Diagram A in Select mode, Diagram B in Edge mode
**When**: User switches between them
**Then**: Each maintains its tool mode
**Contract**:
- Postcondition: Tool mode per-diagram
- Invariant: Mode doesn't change on switch

### SES-007: Session Persistence
**Given**: Multiple diagrams with changes
**When**: User saves session / auto-save triggers
**Then**: All diagrams saved with state
**Contract**:
- Postcondition: All dirty flags cleared
- Invariant: Session can be restored

### SES-008: Session Restoration
**Given**: Previously saved session
**When**: Application starts with session file
**Then**: All diagrams restored with state
**Contract**:
- Postcondition: Same tabs, viewports, selections
- Invariant: History stacks restored

### SES-009: Max Diagrams Limit
**Given**: Session with diagrams
**When**: Creating new diagram would exceed limit
**Then**: Shows warning or prevents creation
**Contract**:
- Limit: 50 diagrams maximum
- Postcondition: User informed of limit
- Invariant: No memory exhaustion

### SES-010: Memory Management
**Given**: Many large diagrams
**When**: Memory pressure detected
**Then**: Inactive diagrams can be unloaded
**Contract**:
- Postcondition: Active diagram always responsive
- Invariant: Can reload from disk

## Data Types

```rust
/// Unique identifier for a diagram session
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SessionId(String);

/// Manages all diagram sessions in the application
pub struct SessionManager {
    sessions: HashMap<SessionId, DiagramSession>,
    active_session_id: SessionId,
    tab_order: Vec<SessionId>,
    clipboard: ClipboardState,
    max_sessions: usize,
}

/// A single diagram session with all its state
pub struct DiagramSession {
    id: SessionId,
    document: DiagramDocument,
    history: History,
    viewport: ViewportState,
    selection: SelectionState,
    tool_mode: ToolMode,
    dirty: bool,
    file_path: Option<PathBuf>,
    name: String,
}

/// Tab representation for UI
pub struct TabInfo {
    id: SessionId,
    name: String,
    dirty: bool,
    is_active: bool,
}

/// Viewport state per diagram
pub struct ViewportState {
    camera_x: f64,
    camera_y: f64,
    zoom: f64,
}

/// Selection state per diagram
pub struct SelectionState {
    selected_items: HashSet<String>,
}
```

## Error Handling

| Error | Condition | Response |
|-------|-----------|----------|
| `SessionError::MaxDiagramsReached` | Creating diagram would exceed 50 | Show error dialog |
| `SessionError::SessionNotFound` | Referencing non-existent session | Log error, no-op |
| `SessionError::SaveFailed` | Diagram save fails | Show error, keep dirty flag |
| `SessionError::LoadFailed` | Diagram load fails | Show error, create empty |
| `SessionError::TabCloseCancelled` | User cancels close prompt | No action taken |

## Performance Requirements

| Operation | Max Latency |
|-----------|-------------|
| Tab switch | 16ms |
| New diagram creation | 50ms |
| Close diagram | 50ms |
| Session save (10 diagrams) | 500ms |
| Session restore (10 diagrams) | 1s |
| Tab reorder | 16ms |

## Invariants

1. **I1**: At least one diagram session exists at all times
2. **I2**: Active session ID always references an existing session
3. **I3**: Tab order matches sessions HashMap keys
4. **I4**: Clipboard persists across tab switches
5. **I5**: Each diagram has unique SessionId
6. **I6**: History stacks are independent per diagram
7. **I7**: Dirty state accurately reflects unsaved changes

## Verification Criteria

All test cases must pass:
- [ ] TAB-001: Create new diagram
- [ ] TAB-002: Switch between diagrams
- [ ] TAB-003: Close diagram tab
- [ ] TAB-004: Close last tab
- [ ] TAB-005: Reorder tabs
- [ ] TAB-006: Tab dirty state
- [ ] TAB-007: Diagram name in tab
- [ ] TAB-008: Keyboard navigation
- [ ] TAB-009: Tab context menu
- [ ] TAB-010: Tab middle-click close
- [ ] SES-001: Session initialization
- [ ] SES-002: Clipboard cross-diagram
- [ ] SES-003: History per-diagram
- [ ] SES-004: Viewport per-diagram
- [ ] SES-005: Selection per-diagram
- [ ] SES-006: Tool mode per-diagram
- [ ] SES-007: Session persistence
- [ ] SES-008: Session restoration
- [ ] SES-009: Max diagrams limit
- [ ] SES-010: Memory management

## Acceptance Criteria

- [ ] All 20 test cases implemented and passing
- [ ] Zero unwrap/panic in production code
- [ ] Zero unsafe code
- [ ] #![deny(clippy::unwrap_used)] present
- [ ] #![forbid(unsafe_code)] present
- [ ] Performance requirements met
- [ ] All invariants verified in tests
