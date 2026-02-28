# Contract: bd-3vt

**bead_id**: bd-3vt  
**bead_title**: subgraph: add save-reload stability regressions  
**phase**: contract  
**updated_at**: 2026-03-01T06:39:00Z

## Overview
Bug fix to ensure subgraph and child geometry are preserved across save and reload operations, preventing recursive auto-resize distortions after reload.

## EARS Requirements

### Ubiquitous
- THE SYSTEM SHALL preserve subgraph and child geometry across save and reload.

### Event-Driven
- WHEN workspace with subgraphs is loaded and a child moves, THE SYSTEM SHALL keep container and child sizes stable per current rules.

### Unwanted
- IF child is moved after reload, THE SYSTEM SHALL NOT trigger recursive auto-resize distortions, because container chain reactions corrupt layouts.

## Contracts

### Preconditions
- auth_required: false
- required_inputs: []
- system_state:
  - Subgraph creation and persistence flows exist

### Postconditions
- state_changes:
  - Subgraph persistence regressions pass under baseline
- return_guarantees: []

### Invariants
- No parent cycles
- World-space child positions remain coherent after reload

## Research Requirements
- Files to read: diagram_tool/e2e/diagram.subgraph-resize.spec.ts, diagram_tool/src/ui/canvas.rs, diagram_tool/src/ui/subgraph.rs
- What to extract: Existing patterns for save/reload stability
