# Architecture Specification: JSON File I/O and Ghost Diff System

**Version:** 1.0
**Date:** 2026-03-23
**Status:** Ready for Decomposition
**Related ADRs:** adr-014, adr-015, adr-016

---

## 1. Problem Statement

Seshat needs a complete file-based persistence and AI collaboration system that enables:

1. Users to save/load diagram documents as JSON files
2. AI agents to propose changes that users can review before accepting
3. Crash recovery without data loss
4. Clear source of truth hierarchy between filesystem and SQLite cache

Currently, the system has partial implementations but lacks:
- Formal `ProposedChanges` data types for AI proposals
- Ghost diff visualization in the UI
- Proper dirty flag tracking and session management
- Auto-save and crash recovery mechanisms

---

## 2. EARS Requirements

### 2.1 Ubiquitous

- THE SYSTEM SHALL persist all documents as `.seshat.json` files with schema version 2
- THE SYSTEM SHALL validate all JSON against the schema before loading
- THE SYSTEM SHALL reject any proposal with stale base_revision
- THE SYSTEM SHALL maintain filesystem as primary source of truth

### 2.2 Event-Driven

- WHEN a user saves THE SYSTEM SHALL write to `.seshat.json` file and clear dirty flag
- WHEN a user opens a file THE SYSTEM SHALL load JSON, validate, and OVERWRITE SQLite cache
- WHEN an AI proposes changes THE SYSTEM SHALL display ghost diff preview
- WHEN a user accepts proposal THE SYSTEM SHALL apply changes, increment revision, clear ghost state
- WHEN a user rejects proposal THE SYSTEM SHALL clear ghost state without modifying document
- WHEN auto-save timer fires (30s) THE SYSTEM SHALL write to SQLite only, not file
- WHEN app restarts after crash THE SYSTEM SHALL detect if SQLite revision > file revision and offer recovery

### 2.3 State-Driven

- WHILE document revision differs from last_saved_revision THE SYSTEM SHALL show dirty indicator (●)
- WHILE ghost diff is pending THE SYSTEM SHALL show accept/reject buttons
- WHILE proposal base_revision != current revision THE SYSTEM SHALL reject as stale

### 2.4 Unwanted

- IF NaN or Infinity found in document THE SYSTEM SHALL NOT serialize (reject with error)
- IF AI tries to write directly to file THE SYSTEM SHALL NOT allow it (must go through ghost diff)
- IF ghost diff is pending on tab close THE SYSTEM SHALL NOT prompt (silent discard)
- IF edge references non-existent node THE SYSTEM SHALL NOT load document

---

## 3. Domain Model

### 3.1 Entities

| Entity | Key Fields | Description |
|--------|-----------|-------------|
| ProposedChanges | base_revision, proposer, proposed_at, changes, summary | AI proposal container |
| ProposedChange | variant-specific fields | Single atomic change |
| GhostDiffState | pending, toggled | UI state for proposal review |
| DocumentSession | doc, file_path, last_saved_revision | Session tracking |

### 3.2 ProposedChange Variants

```
ProposedChange
  +-- MoveNode { node_id, from, to }
  +-- AddNode { node }
  +-- DeleteNode { node_id, was }
  +-- UpdateNodeLabel { node_id, from, to }
  +-- UpdateNodeProperty { node_id, property, from, to }
  +-- AddEdge { edge_id, edge }
  +-- DeleteEdge { edge_id, was }
  +-- UpdateEdgeRouting { edge_id, from_bend_points, to_bend_points }
  +-- UpdateEdgeLabel { edge_id, from, to }
```

### 3.3 State Machines

**Ghost Diff State Machine:**
```
States: IDLE, REVIEWING, APPLYING, APPLIED, CLEARED

Transitions:
  IDLE -> REVIEWING: ProposalReceived
  REVIEWING -> APPLYING: AcceptAll/AcceptSelected
  REVIEWING -> CLEARED: RejectAll/DocumentChanged
  APPLYING -> APPLIED: ApplySuccess
  APPLYING -> CLEARED: ApplyFailed
  APPLIED -> IDLE: immediate
  CLEARED -> IDLE: immediate
```

**Document Session State Machine:**
```
States: CLEAN, DIRTY

Transitions:
  CLEAN -> DIRTY: any mutation
  DIRTY -> CLEAN: save succeeds
```

---

## 4. KIRK Contracts

### Component: GhostDiffManager

**Preconditions:**
| # | Condition | Enforcement | Error |
|---|-----------|-------------|-------|
| P1 | proposal.base_revision == doc.revision | Runtime check | Stale proposal rejected |
| P2 | Only one pending proposal | State check | New proposal replaces old |

**Postconditions:**
| # | Guarantee | Verification |
|---|-----------|-------------|
| Q1 | Accept increments revision | Assert: new_rev == old_rev + 1 |
| Q2 | Accept sets dirty flag | Assert: is_dirty() == true |
| Q3 | Reject leaves document unchanged | Assert: doc == old_doc |

**Invariants:**
| # | Condition | Enforcement |
|---|-----------|-------------|
| I1 | Ghost state cleared on document reload | Always |
| I2 | Ghost state cleared on tab close | Always |
| I3 | Stale proposals never applied | Revision check |

### Component: DocumentSession

**Preconditions:**
| # | Condition | Enforcement | Error |
|---|-----------|-------------|-------|
| P1 | File exists for load | Runtime check | IoError |
| P2 | Parent dir writable for save | Runtime check | IoError |
| P3 | Document has no NaN/Infinity | Validation | SerializationFailed |

**Postconditions:**
| # | Guarantee | Verification |
|---|-----------|-------------|
| Q1 | Load overwrites SQLite | SQLite rev == file rev |
| Q2 | Save clears dirty flag | is_dirty() == false |
| Q3 | Auto-save does NOT clear dirty | is_dirty() unchanged |

**Invariants:**
| # | Condition |
|---|-----------|
| I1 | File = source of truth |
| I2 | SQLite = cache only |
| I3 | Dirty flag accurate |

---

## 5. Error Taxonomy

| Error | When | Message | Recovery |
|-------|------|---------|----------|
| IoError | File read/write fails | "Cannot read/write file" | Retry / Save As |
| ParseError | Invalid JSON | "Invalid file format" | Reject load |
| UnsupportedVersion | Version < 0.9 or > 2 | "Unsupported file version" | Reject load |
| StaleRevision | proposal.base_revision != doc.revision | "Proposal is outdated" | AI must regenerate |
| SerializationFailed | NaN/Infinity in doc | "Cannot save: invalid data" | Fix document |
| DiskFull | No space | "Disk full" | Save As elsewhere |

---

## 6. Module Structure

### New Files to Create

```
diagram_models/src/
  proposed_changes.rs           # ProposedChanges, ProposedChange enum
  proposed_changes_tests.rs     # Unit tests

diagram_tool/src/
  document_session.rs           # DocumentSession, dirty tracking
  autosave.rs                   # Auto-save timer, SQLite writes
  ui/ghost_diff/
    mod.rs                      # Exports
    state.rs                    # GhostDiffState, PendingProposal
    apply.rs                    # apply_proposal(), ApplyResult
    render.rs                   # Dioxus ghost visualization
    toast.rs                    # Notifications
    tests.rs                    # Integration tests
  ui/
    tab_title.rs                # Dirty indicator (●)
    recovery_modal.rs           # Crash recovery prompt
```

### Files to Modify

```
diagram_models/src/lib.rs       # Add pub mod proposed_changes
diagram_models/src/physical_io.rs  # Ensure load/save correct
diagram_tool/src/lib.rs         # Add new modules
```

---

## 7. Implementation Epics

### Epic 1: ProposedChanges Domain Types (Priority: HIGH)
- Create `proposed_changes.rs` with `ProposedChanges` struct
- Implement `ProposedChange` enum with all 9 variants
- Add serde derives and roundtrip tests
- Add validation for `base_revision` matching

### Epic 2: Ghost Diff State Management (Priority: HIGH)
- Create `ui/ghost_diff/` module structure
- Implement `GhostDiffState` with pending proposal tracking
- Implement state machine transitions
- Wire into Dioxus signal state

### Epic 3: Ghost Diff Rendering (Priority: MEDIUM)
- Add CSS classes for ghost node/edge styling
- Implement badge rendering (+, ~, ×)
- Implement ghost preview visualization
- Add accept/reject UI buttons

### Epic 4: Apply Proposal Logic (Priority: HIGH)
- Implement `apply_proposal()` function
- Implement revision matching check
- Implement each `ProposedChange` variant application
- Handle partial conflicts with skip reasons

### Epic 5: Document Session Management (Priority: HIGH)
- Create `DocumentSession` struct
- Implement dirty flag tracking
- Wire up New/Open/Save/Save As operations
- Add dirty indicator to tab title

### Epic 6: Auto-Save System (Priority: MEDIUM)
- Create auto-save timer (30s interval)
- Implement SQLite auto-save writes
- Handle auto-save failures with telemetry

### Epic 7: Crash Recovery (Priority: MEDIUM)
- Detect stale cache on app start
- Implement recovery modal UI
- Wire up accept/reject recovery actions

### Epic 8: CLI Integration (Priority: HIGH)
- Add `seshat show --json` command
- Add `seshat validate` command
- Add `seshat propose` command
- Return proper JSON responses

### Epic 9: Testing (Priority: HIGH)
- Unit tests for ProposedChanges
- Integration tests for ghost diff flow
- E2E tests for file save/load
- Tests for crash recovery

---

## 8. Acceptance Criteria

| # | Scenario | Given | When | Then |
|---|----------|-------|------|------|
| GHOST-001 | Accept proposal | Ghost diff pending | Click Accept All | Changes applied, revision incremented, ghost cleared |
| GHOST-002 | Reject proposal | Ghost diff pending | Click Reject All | No changes, ghost cleared |
| GHOST-003 | Stale proposal | Doc at rev 5 | Proposal with rev 4 | Rejected with "stale" error |
| GHOST-004 | Partial accept | 5 changes proposed | Accept 2, reject 3 | Only 2 applied |
| PERSIST-001 | Save file | Dirty document | Cmd+S | File written, dirty cleared |
| PERSIST-002 | Load file | Existing file | Open | Document loaded, SQLite overwritten |
| PERSIST-003 | Auto-save | Document modified | Wait 30s | SQLite updated, file unchanged |
| PERSIST-004 | Crash recovery | SQLite rev > file rev | App restart | Recovery modal shown |
| PERSIST-005 | Dirty indicator | Unsaved changes | View tab | Shows ● in title |

---

## 9. Dependencies

**Must complete first:**
- None (self-contained feature)

**Blocks:**
- AI CLI full integration
- Production deployment

---

## 10. Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Ghost diff UI complexity | MEDIUM | Incremental rollout, start with Accept All |
| Large proposal overwhelm | LOW | Group by type, summary view |
| Auto-save failures | MEDIUM | Telemetry, retry logic |
| Cache drift | LOW | Strict rules, overwrite on load |
