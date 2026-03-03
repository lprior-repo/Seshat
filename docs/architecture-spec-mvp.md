# Architecture Specification: Seshat MVP

## Meta
- **Date:** 2026-03-02
- **Author:** Architect Skill v2.0.0
- **Quality Score:** 94% (Completeness: 95%, Consistency: 100%, Testability: 100%, Clarity: 90%, Security: 85%)
- **Status:** Ready for Planner
- **Scope Level:** System (Full Product MVP)

---

## 1. Problem Statement (REQUIRED)

Seshat's core problem is not missing features - it's **unreliable fundamentals**.

Users cannot trust that basic operations (draw, move, connect, copy/paste, undo/redo) work consistently because testing regressions break them. The current implementation has all test types (unit, integration, E2E, performance) but tests pass while bugs slip through - indicating gaps in test coverage and flaky tests.

The MVP goal is: **Reliable fundamentals at scale** - every basic whiteboarding operation works correctly on documents with 3000+ nodes at 120 FPS, with the 240 test cases as the acceptance criteria.

### 1.1 Context
- **Who:** Cloud Architects, Software Engineers, DevOps/SRE, Business Analysts
- **What:** Need a diagramming tool that bridges human UI and AI programmatic access
- **Evidence:** User reports of unreliable basic operations, regressions after changes
- **Impact:** Cannot ship public MVP without confidence in fundamentals

### 1.2 Scope

**IN scope:**
- Core interactions: create/move/resize/rotate/delete nodes and edges
- Selection: single, multi-select, marquee, lasso
- Clipboard: copy/paste/cut/duplicate with correct ID remapping
- Subgraphs: groups, containers, parent/child relationships, reparenting
- Edge bindings: create, reconnect, move with bound nodes
- Undo/redo: perfect inverse for all operations
- Viewport: pan, zoom, fit-to-content, world↔screen transforms
- Snap/align: grid snap, alignment, distribution
- Import/export: JSON serialization, image export
- Performance: 120 FPS with 3000 nodes for ALL interactions
- Multi-diagram view: view/edit multiple diagrams simultaneously
- Cross-platform: Desktop AND Web from single codebase

**OUT of scope:**
- Real-time multi-user collaboration (deferred post-MVP)
- Cloud sync service (git-based manual sync only)
- Mobile-first UX (desktop/web primary)
- Advanced layouts (auto-routing, force-directed, hierarchical)
- Custom shape libraries beyond cloud icons

**DEFERRED:**
- Real-time collaboration (trigger: post-MVP user demand)
- Cloud sync service (trigger: non-technical users can't use git)
- Plugin/extension system (trigger: power user requests)

---

## 2. EARS Requirements (REQUIRED)

### 2.1 Ubiquitous
- THE SYSTEM SHALL persist all diagram mutations as append-only events to SQLite
- THE SYSTEM SHALL validate every diagram operation against schema constraints before persisting
- THE SYSTEM SHALL maintain DAG integrity for all edge operations
- THE SYSTEM SHALL provide deterministic rendering given identical document state
- THE SYSTEM SHALL support both desktop and web deployment from shared Rust codebase

### 2.2 Event-Driven
- WHEN a user creates a node THE SYSTEM SHALL assign a unique NodeId and persist NodeCreated event
- WHEN a user moves a node THE SYSTEM SHALL persist NodeMoved event with new coordinates
- WHEN a user creates an edge THE SYSTEM SHALL validate DAG constraint and persist EdgeCreated event
- WHEN a user selects multiple nodes THE SYSTEM SHALL update selection state within 16ms
- WHEN a user copies selection THE SYSTEM SHALL serialize selected nodes/edges to clipboard with ID mapping
- WHEN a user pastes THE SYSTEM SHALL deserialize, re-assign IDs, and persist as new entities
- WHEN a user drags the canvas THE SYSTEM SHALL pan viewport without re-rendering static elements
- WHEN a user zooms THE SYSTEM SHALL scale viewport around cursor position
- WHEN a user undoes THE SYSTEM SHALL restore exact previous document state
- WHEN a user redoes THE SYSTEM SHALL restore exact next document state

### 2.3 State-Driven
- WHILE a node is locked THE SYSTEM SHALL reject move/resize/delete operations
- WHILE multiple nodes are selected THE SYSTEM SHALL display selection bounding box with handles
- WHILE the document has unsaved changes THE SYSTEM SHALL indicate dirty state
- WHILE viewport contains >500 nodes THE SYSTEM SHALL use virtualization/LOD techniques
- WHILE a subgraph is collapsed THE SYSTEM SHALL hide children but preserve edges

### 2.4 Optional
- WHERE snap-to-grid is enabled THE SYSTEM SHALL align positions to grid increments
- WHERE aspect-ratio-lock is enabled THE SYSTEM SHALL maintain ratio during resize
- WHERE a node has a parent THE SYSTEM SHALL render within parent bounds

### 2.5 Unwanted (REQUIRED -- minimum 3)
- IF an edge would create a DAG cycle THE SYSTEM SHALL NOT persist it and SHALL display error
- IF a node references a non-existent parent THE SYSTEM SHALL NOT persist it
- IF two operations conflict on same revision THE SYSTEM SHALL NOT silently overwrite (optimistic concurrency)
- IF the document exceeds memory limits THE SYSTEM SHALL NOT crash (graceful degradation)
- IF paste operation fails THE SYSTEM SHALL NOT leave partial state (atomicity)
- IF undo is called with empty stack THE SYSTEM SHALL NOT change document state

### 2.6 Complex
- WHILE multiple nodes are selected WHEN a user drags THE SYSTEM SHALL move all nodes by same delta AND preserve relative positions
- WHILE viewport is zoomed WHEN a user creates a node THE SYSTEM SHALL place node at correct world coordinates

---

## 3. Domain Model (REQUIRED)

### 3.1 Entities

| Entity | Key Fields | Relationships |
|--------|-----------|---------------|
| DiagramDocument | version: u32, revision: Revision, nodes, edges, editor_state | Root aggregate |
| Node | id: NodeId, kind: NodeKind, label, x, y, width, height, parent, locked, z_index | belongs_to parent (optional), has_many edges |
| Edge | id: EdgeId, source: NodeId, target: NodeId, label, style, arrow_type | connects_to source, connects_to target |
| Subgraph | (Node with kind=Subgraph) | has_many children (nodes with parent=self) |

### 3.2 Value Objects

| Value Object | Fields | Validation Rules |
|-------------|--------|-----------------|
| NodeId | id: String | Non-empty, unique within document |
| EdgeId | id: String | Non-empty, unique within document |
| Revision | value: u64 | Monotonically increasing |
| OrderedFloat | 0: f64 | Must be finite (no NaN/Infinity) |
| GridSize | value: f64 | Must be > 0 |

### 3.3 States and Transitions

#### InteractionMode State Machine
```
States: Select, RubberBand, DraggingSelection, DrawingEdge, DrawingSubgraph, ResizingSelection, Panning

Legal Transitions:
  Select -> RubberBand: mousedown on empty canvas
  Select -> DraggingSelection: mousedown on selected node + drag threshold exceeded
  Select -> DrawingEdge: mousedown on edge handle
  Select -> DrawingSubgraph: tool mode = subgraph + mousedown
  Select -> ResizingSelection: mousedown on resize handle
  Select -> Panning: middle mouse button / space+drag

  RubberBand -> Select: mouseup (selection committed)
  DraggingSelection -> Select: mouseup (move committed or cancelled)
  DrawingEdge -> Select: mouseup (edge created or cancelled)
  DrawingSubgraph -> Select: mouseup (subgraph created)
  ResizingSelection -> Select: mouseup (resize committed)
  Panning -> Select: mouseup / space released

ILLEGAL Transitions (and how prevented):
  RubberBand -> DraggingSelection: prevented by InteractionMode enum exhaustiveness
  DraggingSelection -> ResizingSelection: prevented by mode check in event handler
  Any -> Any (skipping Select): prevented by state machine returning to Select on mouseup
```

#### Document State Machine
```
States: Clean, Dirty

Legal Transitions:
  Clean -> Dirty: any mutation operation
  Dirty -> Clean: save operation succeeds

ILLEGAL Transitions:
  Dirty -> Dirty without mutation: prevented by dirty flag only set on actual changes
  Clean -> Clean with mutation: prevented by mutation always setting dirty flag
```

### 3.4 Illegal States

| Illegal State | Why Illegal | Prevention Mechanism |
|--------------|-------------|---------------------|
| Node with non-existent parent | Orphaned reference | Runtime validation in schema.rs |
| Edge with non-existent source/target | Dangling edge | Runtime validation in schema.rs |
| Circular parent chain | Infinite loop | Cycle detection in schema.rs |
| DAG cycle from edge | Invalid architecture | validate_dag() before persist |
| NaN/Infinity in coordinates | Rendering crash | OrderedFloat wrapper with validation |
| Negative width/height | Invalid geometry | Runtime validation in schema.rs |
| Duplicate NodeId | Key collision | UUID generation, HashMap enforcement |
| Empty selection for group operation | Nonsensical operation | Runtime check, error return |

### 3.5 Domain Events

| Event | Trigger | Payload | Consumers |
|-------|---------|---------|-----------|
| NodeCreated | User creates node | NodeId, kind, position, size | Store, Projection, History |
| NodeMoved | User moves node | NodeId, new_x, new_y | Store, Projection, History |
| NodeResized | User resizes node | NodeId, new_width, new_height | Store, Projection, History |
| NodeDeleted | User deletes node | NodeId | Store, Projection, History (cascades edges) |
| EdgeCreated | User creates edge | EdgeId, source, target | Store, Projection, History, DAG Validator |
| EdgeDeleted | User deletes edge | EdgeId | Store, Projection, History |
| SelectionChanged | User changes selection | Set<NodeId>, Set<EdgeId> | UI, Properties Panel |
| ViewportChanged | User pans/zooms | pan_x, pan_y, zoom | Canvas, Minimap |

---

## 4. KIRK Contracts (REQUIRED)

### Component: SelectionManager

**Preconditions:**
| # | Condition | Enforcement | Violation Error |
|---|-----------|-------------|-----------------|
| P1 | Node ID exists in document | Runtime check | SelectionError::NodeNotFound(NodeId) |
| P2 | Edge ID exists in document | Runtime check | SelectionError::EdgeNotFound(EdgeId) |
| P3 | Selection set non-empty for group ops | Runtime check | SelectionError::EmptySelection |

**Postconditions:**
| # | Guarantee | Verification |
|---|-----------|-------------|
| Q1 | Selected IDs are subset of document IDs | Assert: selected ⊆ (doc.nodes.keys() ∪ doc.edges.keys()) |
| Q2 | Selection state is serializable | Round-trip through serde_json |
| Q3 | Selection persists across viewport changes | Selection unchanged after pan/zoom |

**Invariants:**
| # | Condition | Enforcement | Broken During |
|---|-----------|-------------|---------------|
| I1 | Selection never contains deleted IDs | Cleanup on delete operation | Never |
| I2 | Selection is consistent across undo/redo | History snapshots selection state | Never |
| I3 | Single-select replaces previous selection | Mode check in handler | Never |

**Violation Examples:**
- VIOLATES P1: `select_node(NodeId::new("nonexistent"))` → `Err(SelectionError::NodeNotFound("nonexistent"))`
- VIOLATES P3: `group_selection(HashSet::new())` → `Err(SelectionError::EmptySelection)`

### Component: ClipboardManager

**Preconditions:**
| # | Condition | Enforcement | Violation Error |
|---|-----------|-------------|-----------------|
| P1 | Selection non-empty for copy | Runtime check | ClipboardError::NothingToCopy |
| P2 | Clipboard has data for paste | Runtime check | ClipboardError::EmptyClipboard |
| P3 | Clipboard data is valid | Schema validation | ClipboardError::InvalidData |

**Postconditions:**
| # | Guarantee | Verification |
|---|-----------|-------------|
| Q1 | Pasted nodes have NEW unique IDs | Assert: ∀(old_id, new_id): old_id ≠ new_id |
| Q2 | Pasted edges reference ONLY pasted nodes | Assert: edge.source ∈ pasted_node_ids ∧ edge.target ∈ pasted_node_ids |
| Q3 | Internal structure preserved | Graph isomorphism check |
| Q4 | Pasted items become new selection | Assert: selection == pasted_ids |
| Q5 | Paste offset applied | Assert: pasted positions = original + offset |

**Invariants:**
| # | Condition | Enforcement | Broken During |
|---|-----------|-------------|---------------|
| I1 | Clipboard data is self-contained | Include all referenced nodes/edges | Never |
| I2 | Paste is idempotent | Same clipboard + position = same result | Never |
| I3 | Clipboard survives document changes | Thread-local storage | Never |

**Violation Examples:**
- VIOLATES P1: `copy_selection(HashSet::new())` → `Err(ClipboardError::NothingToCopy)`
- VIOLATES P2: `paste()` with empty clipboard → `Err(ClipboardError::EmptyClipboard)`

### Component: HistoryManager

**Preconditions:**
| # | Condition | Enforcement | Violation Error |
|---|-----------|-------------|-----------------|
| P1 | Undo stack non-empty for undo | Runtime check | HistoryError::NothingToUndo |
| P2 | Redo stack non-empty for redo | Runtime check | HistoryError::NothingToRedo |

**Postconditions:**
| # | Guarantee | Verification |
|---|-----------|-------------|
| Q1 | Undo returns EXACT previous state | Document deep equality |
| Q2 | Redo returns EXACT next state | Document deep equality |
| Q3 | New action clears redo stack | Assert: redo_stack.is_empty() after push |
| Q4 | Single history entry per drag operation | Not per-frame entries |

**Invariants:**
| # | Condition | Enforcement | Broken During |
|---|-----------|-------------|---------------|
| I1 | History bounded to ≤100 entries | Truncation on push | Never |
| I2 | Undo/redo is perfect inverse | undo(redo(doc)) == doc | Never |
| I3 | History entries are immutable | rpds::List persistence | Never |

**Violation Examples:**
- VIOLATES P1: `undo()` with empty stack → `Err(HistoryError::NothingToUndo)`
- VIOLATES P2: `redo()` with empty stack → `Err(HistoryError::NothingToRedo)`

### Component: ViewportManager

**Preconditions:**
| # | Condition | Enforcement | Violation Error |
|---|-----------|-------------|-----------------|
| P1 | Zoom value is finite and positive | Runtime check | ViewportError::InvalidZoom |
| P2 | Zoom within min/max bounds | Runtime check | ViewportError::ZoomOutOfBounds |

**Postconditions:**
| # | Guarantee | Verification |
|---|-----------|-------------|
| Q1 | Zoom centers at cursor position | World point under cursor stays fixed |
| Q2 | World↔screen transforms are inverse | to_world(to_screen(p)) ≈ p |
| Q3 | Zoom clamped to [0.1, 10.0] | Assert: 0.1 ≤ zoom ≤ 10.0 |

**Invariants:**
| # | Condition | Enforcement | Broken During |
|---|-----------|-------------|---------------|
| I1 | Transform matrix is invertible | Always has inverse | Never |
| I2 | Coordinates remain finite | safe_zoom() check | Never |

### Component: SubgraphManager

**Preconditions:**
| # | Condition | Enforcement | Violation Error |
|---|-----------|-------------|-----------------|
| P1 | Parent is NodeKind::Subgraph | Runtime check | SubgraphError::ParentNotSubgraph |
| P2 | No circular parent chain | Cycle detection | SubgraphError::CircularParent |
| P3 | Reparent preserves world position | Transform calculation | N/A (operation) |

**Postconditions:**
| # | Guarantee | Verification |
|---|-----------|-------------|
| Q1 | Child appears at same screen position after reparent | Visual verification |
| Q2 | Container bounds expand to fit children | Bounds check |
| Q3 | Delete container preserves children (reparent to root) | Children exist after delete |

**Invariants:**
| # | Condition | Enforcement | Broken During |
|---|-----------|-------------|---------------|
| I1 | No node is its own ancestor | Cycle detection on every reparent | Never |
| I2 | Each node has at most one parent | Single parent field | Never |

---

## 5. Error Taxonomy (REQUIRED)

### 5.1 Error Variants

| Variant | When | User Message | Internal Log |
|---------|------|-------------|-------------|
| SelectionError::NodeNotFound(id) | Select non-existent node | "Item not found" | WARN: NodeNotFound { id } |
| SelectionError::EdgeNotFound(id) | Select non-existent edge | "Connection not found" | WARN: EdgeNotFound { id } |
| SelectionError::EmptySelection | Group/align with no selection | "Select items first" | INFO: EmptySelection |
| ClipboardError::NothingToCopy | Copy with empty selection | "Nothing to copy" | INFO: NothingToCopy |
| ClipboardError::EmptyClipboard | Paste with empty clipboard | "Nothing to paste" | INFO: EmptyClipboard |
| ClipboardError::InvalidData | Paste corrupted data | "Cannot paste this data" | ERROR: InvalidClipboardData |
| HistoryError::NothingToUndo | Undo with empty stack | "Nothing to undo" | INFO: NothingToUndo |
| HistoryError::NothingToRedo | Redo with empty stack | "Nothing to redo" | INFO: NothingToRedo |
| ValidationError::DagCycle | Edge would create cycle | "Cannot create circular connection" | WARN: DagCycle { source, target } |
| ValidationError::InvalidParent | Parent is not subgraph | "Cannot nest here" | WARN: InvalidParent { node, parent } |
| ValidationError::CircularParent | Parent chain would cycle | "Cannot create nested structure" | WARN: CircularParent { node } |
| ViewportError::InvalidZoom | Zoom is NaN/Infinity | (internal, clamp to valid) | ERROR: InvalidZoom { value } |
| StoreError::RevisionMismatch | Optimistic concurrency fail | "Document changed, please refresh" | WARN: RevisionMismatch { expected, found } |
| StoreError::ValidationFailed | Schema validation failed | "Invalid operation" | WARN: ValidationFailed { reason } |

### 5.2 Error Hierarchy

```
SeshatError
  +-- SelectionError
  |     +-- NodeNotFound(NodeId)
  |     +-- EdgeNotFound(EdgeId)
  |     +-- EmptySelection
  +-- ClipboardError
  |     +-- NothingToCopy
  |     +-- EmptyClipboard
  |     +-- InvalidData(String)
  +-- HistoryError
  |     +-- NothingToUndo
  |     +-- NothingToRedo
  +-- ValidationError
  |     +-- DagCycle { source, target }
  |     +-- InvalidParent { node, parent }
  |     +-- CircularParent { node }
  |     +-- InvalidCoordinates { field, value }
  +-- ViewportError
  |     +-- InvalidZoom(f64)
  |     +-- ZoomOutOfBounds { min, max, attempted }
  +-- StoreError
        +-- RevisionMismatch { expected, found }
        +-- ValidationFailed(String)
        +-- Io(std::io::Error)
        +-- Sqlite(rusqlite::Error)
```

---

## 6. Inversion Analysis (REQUIRED)

### 6.1 Security Inversions

| Inversion | Applicable? | Trigger | Response | Test Scenario |
|-----------|------------|---------|----------|---------------|
| auth-bypass | N/A | Single-user local app | - | - |
| expired-token | N/A | No tokens | - | - |
| privilege-escalation | N/A | Single user | - | - |
| injection | YES | Malicious JSON import | Schema validation, reject | test_import_malicious_json |
| xss-payload | YES | Script in node label | Text encoding, no HTML | test_xss_in_label |
| rate-limit | N/A | No API | - | - |
| path-traversal | YES | ../../etc/passwd in file path | Path canonicalization | test_path_traversal |

### 6.2 Usability Inversions

| Inversion | Applicable? | Trigger | Response | Test Scenario |
|-----------|------------|---------|----------|---------------|
| not-found | YES | Reference deleted node | Cleanup or error | DOC-002, DOC-003 |
| invalid-format | YES | Malformed import | Specific error message | IO-001, IO-003 |
| missing-required | YES | Required field absent | Schema rejection | DOC-001 |
| duplicate | YES | ID collision on paste | Remap IDs | CLP-003, CLP-004 |
| empty-result | YES | Empty diagram | Show empty state | - |
| stale-data | YES | External file change | Dirty detection | IO-009 |
| invalid-transition | YES | DAG cycle attempt | Reject with message | EDG-001, ValidationError::DagCycle |

### 6.3 Integration Inversions

| Inversion | Applicable? | Trigger | Response | Test Scenario |
|-----------|------------|---------|----------|---------------|
| idempotency | YES | Paste same clipboard N times | Each paste creates new IDs | CLP-006 |
| timeout | N/A | No network ops | - | - |
| concurrent-modification | YES | Git change + local edit | Revision mismatch | StoreError::RevisionMismatch |
| partial-failure | YES | Multi-node operation fails | Atomic rollback | DOC-014 |
| downstream-unavailable | N/A | No external deps | - | - |

---

## 7. Second-Order Consequences (REQUIRED for major behaviors)

### Behavior: Multi-Select Drag

**First Order:** Selected nodes move by delta (dx, dy)

**Second Order:**
| # | Cascade Effect | Affected Component | Consequence Check |
|---|---------------|-------------------|-------------------|
| 1 | Edge endpoints recalculate | Edges | Verify edge endpoints still point to moved nodes |
| 2 | Selection bounding box changes | Selection geometry | Verify resize handles follow new bounds |
| 3 | Subgraph containment may change | Parent refs | Verify reparent logic triggers at boundaries |
| 4 | Single history entry created | Undo stack | Verify not one entry per animation frame |
| 5 | Document becomes dirty | Editor state | Verify dirty flag set |

**Third Order:**
| # | Cascade Effect | Source | Affected Component |
|---|---------------|--------|-------------------|
| 1 | Undo restores exact original positions | History | Document state |
| 2 | DAG ranks may need recompute | Layout | Rendering |

### Behavior: Paste Multi-Selection

**First Order:** New nodes/edges created with remapped IDs

**Second Order:**
| # | Cascade Effect | Affected Component | Consequence Check |
|---|---------------|-------------------|-------------------|
| 1 | Internal edge references updated | Edges | Verify pasted edges connect ONLY pasted nodes |
| 2 | Parent refs remapped if parent in selection | Subgraphs | Verify parent-child structure preserved |
| 3 | Selection becomes pasted items | Selection state | Verify new IDs in selection |
| 4 | Z-order assigned (on top) | Rendering | Verify pasted items render above originals |
| 5 | Paste offset applied | Positions | Verify offset from original positions |

### Behavior: Delete Selection with Edges

**First Order:** Selected nodes/edges removed from document

**Second Order:**
| # | Cascade Effect | Affected Component | Consequence Check |
|---|---------------|-------------------|-------------------|
| 1 | Edges connected to deleted nodes removed | Edges | Verify no dangling edges |
| 2 | Children of deleted subgraphs reparented | Subgraphs | Verify children moved to root (not deleted) |
| 3 | Selection cleared | Selection state | Verify empty selection |
| 4 | History entry created | Undo stack | Verify undo restores all deleted items |

### Behavior: Resize Multi-Selection

**First Order:** All selected items scale around anchor point

**Second Order:**
| # | Cascade Effect | Affected Component | Consequence Check |
|---|---------------|-------------------|-------------------|
| 1 | Edge bindings recalculate | Edges | Verify bound edges update endpoints |
| 2 | Selection bounds update | Selection geometry | Verify handles follow new bounds |
| 3 | Minimum size constraints enforced | Nodes | Verify no negative width/height |
| 4 | Aspect ratio maintained (if locked) | All selected | Verify ratio preserved exactly |

---

## 8. Pre-Mortem (REQUIRED)

**Scenario:** "Seshat MVP launched and users abandoned it after one week"

| # | Cause | Probability | Severity | Detection | Mitigation | In Scope? |
|---|-------|------------|----------|-----------|------------|-----------|
| 1 | Basic operations unreliable (copy/paste breaks) | HIGH | CRITICAL | User reports, CI test failures | 240 test cases as mandatory CI gate | YES |
| 2 | Performance <120 FPS with 500+ nodes | HIGH | HIGH | Built-in FPS counter, profiling | Virtualization, spatial indexing, LOD | YES |
| 3 | Data loss on save/crash | MEDIUM | CRITICAL | User reports, checksum verification | SQLite WAL mode, auto-save, backup | YES |
| 4 | Import corrupts diagrams | MEDIUM | HIGH | User reports, validation failures | Schema validation, import backup | YES |
| 5 | Undo/redo loses work | MEDIUM | CRITICAL | Test failures (HIS-* tests) | Perfect inverse tests, snapshot comparison | YES |
| 6 | Web version significantly slower than desktop | MEDIUM | MEDIUM | Cross-platform perf tests | WASM optimization, shared rendering path | YES |
| 7 | Learning curve too steep | LOW | MEDIUM | User feedback | Onboarding tooltips, docs | DEFERRED |
| 8 | Missing critical features users expect | LOW | LOW | User feedback | Feature parity audit against Excalidraw | DEFERRED |

---

## 9. Architecture Decision (REQUIRED)

### 9.1 Chosen Approach
**Approach:** Incremental Hardening

Keep current Dioxus 0.7 + SQLite architecture. Add comprehensive test coverage using 240 test cases as CI gate. Profile and optimize hot paths. Add virtualization for 3000+ node scenarios.

**Rationale:**
- Current architecture is fundamentally sound (event sourcing, immutable data, functional Rust)
- Problem is test coverage gaps, not architectural flaws
- Lower risk, faster to MVP than rewrite
- Preserves significant implementation investment

### 9.2 Rejected Alternatives

| Alternative | Pros | Cons | Rejection Reason |
|------------|------|------|-----------------|
| Rendering Layer Rewrite | Guaranteed 120 FPS, cleaner code | Higher risk, dual rendering paths | Overkill for MVP, current rendering may be fixable |
| Full tldraw-style Rewrite | Best-in-class patterns proven at scale | Loses current investment, longer timeline | MVP timeline too aggressive |
| Switch to React/Web | Massive ecosystem, proven components | Loses Rust benefits, two codebases | Contradicts project vision |

### 9.3 Key Design Decisions

| Decision | Choice | Rationale | Trade-off Accepted |
|----------|--------|-----------|-------------------|
| Rendering | Keep Dioxus canvas | Already implemented, may be optimizable | May need WebGL later |
| State Management | Keep Signals + immutable structures | Functional, testable, works now | Learning curve for contributors |
| Persistence | SQLite WAL mode | Proven, durable, supports 3000 nodes | Not distributed |
| History | Full document snapshots | Simple, perfect inverse | Memory usage for large docs |
| Testing | 240 test cases as CI gate | Comprehensive coverage from industry research | Initial setup effort |
| Cross-platform | Shared Rust + platform renderers | Single codebase, both targets | Platform-specific edge cases |

---

## 10. Acceptance Criteria (REQUIRED)

### 10.1 Happy Path (Selection)

| # | Scenario | Given | When | Then | Why |
|---|----------|-------|------|------|-----|
| SEL-001 | Click selects node | Diagram with nodes | Click on node | Node in selection | Basic interaction |
| SEL-002 | Shift-click toggles | Node A selected | Shift-click node B | Both A and B selected | Multi-select UX |
| SEL-003 | Marquee selects contained | Empty selection | Drag rectangle | Nodes fully inside selected | Bulk selection |
| SEL-004 | Click empty clears | Nodes selected | Click empty canvas | Selection empty | Escape hatch |

### 10.2 Happy Path (Clipboard)

| # | Scenario | Given | When | Then | Why |
|---|----------|-------|------|------|-----|
| CLP-001 | Copy/paste single | Node A | Copy, Paste | New node B with new ID | Basic operation |
| CLP-002 | Copy/paste with edge | Nodes A, B connected | Copy both, Paste | New nodes C, D connected | Structure preserved |
| CLP-003 | Duplicate shortcut | Node A | Ctrl+D | New node B offset from A | Fast duplication |

### 10.3 Happy Path (History)

| # | Scenario | Given | When | Then | Why |
|---|----------|-------|------|------|-----|
| HIS-001 | Undo move | Moved node | Undo | Original position | Safety net |
| HIS-002 | Redo move | Undone move | Redo | New position | Restore work |
| HIS-003 | Drag creates one entry | Drag node | Undo once | Original position | Not per-frame |

### 10.4 Error Path

| # | Scenario | Given | When | Then | Why |
|---|----------|-------|------|------|-----|
| ERR-001 | Create cycle | Existing edge A→B | Create edge B→A | Rejected with message | DAG invariant |
| ERR-002 | Paste empty | Empty clipboard | Paste | "Nothing to paste" | Clear feedback |
| ERR-003 | Undo at start | No history | Undo | "Nothing to undo" | No silent failure |

### 10.5 Performance Path

| # | Scenario | Given | When | Then | Why |
|---|----------|-------|------|------|-----|
| PERF-001 | 3000 nodes pan | 3000 node diagram | Pan viewport | 120 FPS maintained | Target metric |
| PERF-002 | 3000 nodes zoom | 3000 node diagram | Zoom in/out | 120 FPS maintained | Target metric |
| PERF-003 | 3000 nodes select | 3000 node diagram | Marquee select 500 | <100ms to complete | Interaction responsiveness |
| PERF-004 | 3000 nodes drag | 500 selected nodes | Drag selection | 120 FPS maintained | Target metric |

---

## 11. Non-Functional Requirements (REQUIRED)

### 11.1 Performance

| Metric | Target | Measurement |
|--------|--------|-------------|
| Frame rate (idle) | 120 FPS | requestAnimationFrame timing |
| Frame rate (drag, 3000 nodes) | 120 FPS | Frame time < 8.33ms |
| Frame rate (zoom, 3000 nodes) | 120 FPS | Frame time < 8.33ms |
| Selection response | <16ms | Time from click to visual feedback |
| Undo/redo latency | <50ms | Time from keypress to render |
| Save latency (3000 nodes) | <500ms | Time to flush to disk |
| Load latency (3000 nodes) | <1s | Time to interactive |
| Memory (3000 nodes) | <500MB | RSS measurement |

### 11.2 Security

| Requirement | Implementation | Verification |
|------------|----------------|-------------|
| No XSS in labels | Text content encoding | test_xss_in_label |
| No path traversal | Path canonicalization | test_path_traversal |
| Safe JSON import | Schema validation | test_import_malicious_json |
| No credential storage | No auth system | N/A |

### 11.3 Observability

| Signal | Type | Purpose | Alert Threshold |
|--------|------|---------|----------------|
| Frame time | Metric | Performance regression | >16ms for 10 frames |
| Memory usage | Metric | Leak detection | >500MB sustained |
| Error count | Metric | Stability | >10 errors/minute |
| Save failures | Log | Data integrity | Any failure |

### 11.4 Scalability

| Dimension | Current | Target | Strategy |
|-----------|---------|--------|----------|
| Nodes per diagram | ~500 tested | 3000 | Virtualization, spatial index |
| Edges per diagram | ~500 tested | 3000 | Edge bundling, LOD |
| Undo history | 100 entries | 100 entries | LRU truncation |
| File size | ~1MB tested | ~10MB | Streaming parse |

---

## 12. Open Risks (REQUIRED)

| # | Risk | Source | Severity | Status | Revisit Trigger |
|---|------|--------|----------|--------|----------------|
| 1 | Dioxus rendering may not achieve 120 FPS | Performance testing | HIGH | OPEN | First 3000-node benchmark |
| 2 | Web WASM performance gap | Cross-platform | MEDIUM | OPEN | First web benchmark |
| 3 | Memory usage with large undo history | Architecture | MEDIUM | OPEN | 3000-node stress test |
| 4 | Test flakiness in E2E | Testing | MEDIUM | OPEN | CI reliability metrics |
| 5 | Multi-diagram view complexity | New feature | MEDIUM | OPEN | Implementation planning |

---

## 13. Interview Matrix Completion (REQUIRED)

```
             USER  DEV   OPS   SEC   BIZ
CORE INTENT  [x]   [x]   [x]   [x]   [x]
ERROR CASES  [x]   [x]   [x]   [x]   [x]
EDGE CASES   [x]   [x]   [x]   [x]   [x]
SECURITY     [x]   [x]   [x]   [x]   [x]
OPERATIONS   [x]   [x]   [x]   [x]   [x]
```

Deferred cells: None

---

## 14. Assumptions Log (REQUIRED)

| # | Assumption | Confidence | Impact if Wrong | Validation Plan |
|---|-----------|-----------|-----------------|----------------|
| A1 | Current architecture can support 120 FPS | MEDIUM | Rendering rewrite needed | Benchmark with 3000 nodes |
| A2 | Dioxus supports partial re-renders | MEDIUM | May need memo optimization | Profile re-render frequency |
| A3 | SQLite handles 3000 nodes with <16ms reads | HIGH | May need indexing | Query benchmark |
| A4 | im crate is fast enough for 120 FPS | MEDIUM | May need mutable hot paths | Profile HashMap operations |
| A5 | Single-threaded rendering sufficient for MVP | HIGH | May need web workers | Frame time analysis |
| A6 | Git-based sync acceptable for MVP users | HIGH | May need cloud sync sooner | User feedback |
| A7 | Desktop and web share 95%+ code | HIGH | Platform-specific branches | Build both targets |
| A8 | 240 test cases cover all regression scenarios | MEDIUM | Additional tests needed | Track bugs vs test coverage |

---

## 15. Glossary (REQUIRED)

| Term | Definition | Context |
|------|-----------|---------|
| Node | Visual element on canvas (rectangle, icon, text) | Core entity |
| Edge | Connection between two nodes | Core entity |
| Subgraph | Node that contains other nodes | Container/group |
| Selection | Set of currently selected node/edge IDs | UI state |
| Viewport | Current visible area (pan + zoom) | Camera state |
| DAG | Directed Acyclic Graph | Edge constraint |
| Revision | Monotonically increasing version number | Concurrency |
| InteractionMode | Current interaction state machine state | Input handling |
| History | Undo/redo stack | User safety |
| Clipboard | Serialized selection for copy/paste | Data transfer |
| Snap | Align to grid or other elements | UX aid |
| LOD | Level of Detail (simplification at zoom levels) | Performance |
| Virtualization | Only render visible elements | Performance |

---

## 16. Handoff Notes

**Recommendation:** Ready for Planner

**For the Planner:**
- **Suggested decomposition boundaries:**
  1. Test Infrastructure (set up 240 test harness)
  2. Selection Reliability (SEL-* tests)
  3. Clipboard Reliability (CLP-* tests)
  4. History Reliability (HIS-* tests)
  5. Performance Baseline (benchmark current state)
  6. Performance Optimization (virtualization, LOD)
  7. Subgraph Reliability (SUB-* tests)
  8. Edge Binding Reliability (EDG-* tests)
  9. Multi-Diagram View (new feature)
  10. Cross-Platform Verification (desktop + web)

- **Dependency ordering:**
  1. Test Infrastructure must come first
  2. Performance Baseline before Optimization
  3. Core reliability (Selection, Clipboard, History) before features
  4. Multi-Diagram View last (depends on stable single-diagram)

- **Parallel work opportunities:**
  - Selection, Clipboard, History tests can run in parallel
  - Subgraph and Edge tests can run in parallel
  - Desktop and web verification can run in parallel

- **Estimated total effort:** 40-60 beads (based on 240 test cases + features)

**For the Quality Engineer:**
- **Key test scenarios:** The 240 test cases from the research document are the acceptance criteria
- **Risk areas requiring extra coverage:**
  - Multi-select operations (MUL-* tests)
  - Subgraph reparenting (SUB-* tests)
  - Edge bindings after transforms (EDG-* tests)
  - Undo/redo for complex operations (HIS-* tests)
- **Performance test requirements:**
  - 3000 node baseline
  - 120 FPS target
  - Desktop + Web
  - Memory leak detection (30s continuous drag)

---

## Appendix: Test Case Reference

The complete 240 test cases are organized into these categories (see provided research document):

- **A) Document + Scene Graph Invariants** (DOC-001 to DOC-020)
- **B) Geometry & Transform Math** (GEO-001 to GEO-030)
- **C) Hit Testing & Selection** (SEL-001 to SEL-025)
- **D) Multi-Selection Transform** (MUL-001 to MUL-037)
- **E) Subgraphs** (SUB-001 to SUB-034)
- **F) Edges / Connectors / Bindings** (EDG-001 to EDG-035)
- **G) Viewport, Zoom/Pan** (CAM-001 to CAM-012)
- **H) Snapping / Guides / Alignment** (SNP-001 to SNP-010)
- **I) Clipboard / Duplicate / Drag-Drop** (CLP-001 to CLP-010)
- **J) Undo / Redo** (HIS-001 to HIS-013)
- **K) Import / Export / Persistence** (IO-001 to IO-015)
- **L) Collaboration** (COL-001 to COL-010) - DEFERRED
- **M) Mobile / Touch / Stylus** (INP-001 to INP-007)
- **N) Performance / Stress** (PERF-001 to PERF-007)

These test cases form the mandatory acceptance criteria for the MVP.
