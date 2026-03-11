# Architecture Drift Report

## Summary

Analysis of source files in `diagram_tool/src` reveals significant architectural drift with **6 files exceeding 300 lines** and numerous primitive obsession violations.

## Files Exceeding 300 Lines (Critical)

| File | Lines | Status |
|------|-------|--------|
| `src/ui/commands.rs` | 3852 | NEEDS SPLIT |
| `src/ui/canvas.rs` | 3193 | NEEDS SPLIT |
| `src/ui/canvas/interaction_reducer.rs` | 3135 | NEEDS SPLIT |
| `src/history.rs` | 1929 | NEEDS SPLIT |
| `src/models/envelope.rs` | 1883 | NEEDS SPLIT |
| `src/models/document.rs` | 1325 | REVIEW |
| **Total** | **15,317** | |

## Primitive Obsession Violations

Found **69 instances** of `id: String` where NewTypes (`NodeId`, `EdgeId`) should be used:

### Critical Locations
- `models/envelope.rs` - DomainOp uses `String` for IDs (should be NodeId/EdgeId)
- `ui/dispatch.rs` - Dispatch functions accept `String` IDs
- `store/types.rs` - Operation types use `String` for IDs
- `models/export.rs` - Export operations use `String` for IDs

## Refactoring Plan

### Phase 1: Split commands.rs (3852 → ~600 lines)
Split into logical modules:
- `ui/commands/clipboard.rs` - Clipboard operations
- `ui/commands/alignment.rs` - Alignment & distribution
- `ui/commands/mod.rs` - Re-exports

### Phase 2: Split canvas components
- `canvas.rs` (3193 lines) - Extract canvas event handlers
- `interaction_reducer.rs` (3135 lines) - Already has domain/ subdirectory

### Phase 3: Split envelope.rs
- `models/envelope/domain_ops.rs` - DomainOp enum
- `models/envelope/parsing.rs` - Parsing logic
- `models/envelope/mod.rs` - Re-exports

### Phase 4: Split history.rs
- `history/state.rs` - History state management
- `history/stack.rs` - Undo/redo stack

### Phase 5: Fix Primitive Obsession
- Replace `String` with `NodeId`/`EdgeId` in DomainOp
- Update dispatch functions to use NewTypes

## DDD Compliance Check

### ✅ Good (NewTypes Present)
- `NodeId`, `EdgeId`, `Revision`, `OrderedFloat` in document.rs
- Proper error types with thiserror
- Parse don't validate pattern in OrderedFloat

### ❌ Issues
- DomainOp uses String for entity IDs
- Many functions accept String instead of typed IDs
- Large enum in envelope.rs mixes domain concepts with parsing

## Status: REFACTORED

This report identifies the architectural drift. Refactoring required.
