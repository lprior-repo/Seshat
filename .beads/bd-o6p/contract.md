# Contract: watcher-sync: ingest canonical file edits safely

bead_id: bd-o6p
bead_title: watcher-sync: ingest canonical file edits safely
phase: p0
updated_at: 2026-02-28T19:13:05Z

## EARS Requirements

### Ubiquitous
- THE SYSTEM SHALL emit JSONL events for every CLI command stage
- THE SYSTEM SHALL preserve a valid last-known-good diagram state

### Event-Driven
- WHEN a mutation request is accepted, THE SYSTEM SHALL run the full validation pipeline before persistence and broadcast
- WHEN validation fails, THE SYSTEM SHALL reject the mutation and return a machine-readable error code

### Unwanted
- IF concurrent updates target the same diagram revision, THE SYSTEM SHALL NOT silently overwrite one update with another, because: Silent overwrites lose user or AI intent and corrupt trust

## Preconditions
- auth_required: false
- required_inputs: []
- system_state:
  - Input JSON parses into the expected command payload
  - Diagram identifier resolves to a known or creatable single-diagram slot

## Postconditions
- state_changes:
  - Command exits with deterministic exit code and JSONL finish event
  - Persisted diagram remains schema-valid and DAG-valid on success

## Invariants
- Revision is monotonic and only server-owned
- Invalid mutations never reach UI broadcast

## Research Requirements
- Read diagram_tool/src/cli.rs for existing patterns
- Read diagram_tool/src/backend.rs for existing patterns
- Read diagram_tool/src/patch.rs for existing patterns
- Read diagram_tool/src/models/document.rs for existing patterns

## Implementation Tasks
1. Read current CLI and backend flow to identify insertion points
2. Map existing validation and layout functions to target pipeline stages
3. Write failing integration test for command JSONL format and exit code map
4. Write failing test for rejection path preserving last-known-good state
5. Add file watcher ingest with debounce, stable-write detection, and invalid-change rejection that preserves last-known-good server state
6. Add structured error-code mapping and JSONL serializer
