# Contract: schema: Two-way JSON schema sync for AI integration

bead_id: bd-30o
bead_title: schema: Two-way JSON schema sync for AI integration
phase: p0
updated_at: 2026-02-28T19:45:00Z

## EARS Requirements

### Ubiquitous
- THE SYSTEM SHALL export current diagram state as JSON schema
- THE SYSTEM SHALL import JSON schema to update diagram state
- THE SYSTEM SHALL auto-save diagram schema to localStorage on changes

### Event-Driven
- WHEN diagram state changes, THE SYSTEM SHALL serialize to JSON and save to localStorage
- WHEN user imports JSON schema, THE SYSTEM SHALL parse schema and replace current diagram state

### Unwanted
- IF imported JSON has invalid schema structure, THE SYSTEM SHALL NOT corrupt current diagram state, because: Invalid imports must not destroy user work

## Preconditions
- auth_required: false
- required_inputs: []
- system_state:
  - Diagram state is serializable to JSON
  - localStorage is available in browser

## Postconditions
- state_changes:
  - Exported JSON is valid and parseable
  - Imported schema produces valid diagram state

## Invariants
- Schema version is included for forward compatibility
- All node and edge data is preserved in schema

## Implementation Status: PARTIAL
1. Export JSON - ✅ Implemented (export_actions.rs:83)
2. Import JSON - ✅ Implemented (persistence.rs:20-35)
3. Auto-save to localStorage - ❌ NOT implemented (manual save only)
4. Schema versioning - ❌ NOT implemented
5. Invalid import protection - ✅ Implemented (ImportTransitionError)

## Contract Compliance
- Export diagram as JSON: ✅
- Import JSON to update diagram: ✅
- Auto-save to localStorage on changes: ❌ NOT IMPLEMENTED
- Invalid import protection: ✅
