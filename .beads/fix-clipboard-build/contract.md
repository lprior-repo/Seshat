# Contract: Fix Critical Build-Breaking Issues in Seshat

## bead_id: fix-clipboard-build
## bead_title: Fix Critical Build-Breaking Issues
## phase: p0
## updated_at: 2026-03-04T12:00:00Z

## Problem Statement

The codebase is currently non-buildable due to several critical issues:

1. **Clipboard refactor half-integrated** - `apply_copy_selection`/`apply_paste_selection` require new `Signal<Option<Clipboard>>` parameter but callers still use old signatures
2. **const fn uses unstable const behavior** - `clipboard_has_content` uses `is_some_and` which is not const-stable
3. **Test layer broken** - Tests still call removed symbols (`clear_clipboard`, `copy_selection_to_clipboard`, etc.)
4. **Event decode errors silently dropped** - `filter_map(Result::ok)` loses data without surfacing errors
5. **OCC not safe under concurrent writers** - No DB uniqueness on revision, locking not integrated
6. **Import treats OCC mismatch as success** - Silent acceptance of conflicts

## Acceptance Criteria

### Build Criteria
- [ ] `cargo check --workspace` passes with no errors
- [ ] `cargo test --workspace` compiles and runs
- [ ] All clippy warnings resolved (except documented allowances)

### Functional Criteria
- [ ] Clipboard operations work via keyboard shortcuts (Ctrl+C/V/D)
- [ ] Clipboard operations work via toolbar buttons
- [ ] Event log import/export maintains data integrity
- [ ] Concurrent writes to SQLite fail gracefully with proper error messages

### Test Criteria
- [ ] Legacy clipboard tests migrated to new API or removed
- [ ] Property-based tests still run and pass
- [ ] E2E tests for clipboard work (if applicable)

## Technical Approach

### Fix 1: Complete Clipboard Integration

1. Add `Signal<Option<Clipboard>>` context provider in `app.rs`
2. Pass clipboard signal through keyboard hook and toolbar actions
3. Fix `clipboard_has_content` to be non-const or use const-compatible pattern

### Fix 2: Fix Test Layer

1. Remove calls to removed functions (`clear_clipboard`, etc.)
2. Use new pure `Clipboard` API in tests
3. Fix `is_finite` method calls (add `()`)

### Fix 3: Fix Silent Event Loss

1. Replace `filter_map(Result::ok)` with explicit error handling
2. Return structured decode errors with revision/op context
3. Log warnings for skipped rows

### Fix 4: Strengthen OCC

1. Add unique constraint on revision column (or use alternative)
2. Use immediate transactions with retry on conflict
3. Integrate DiagramLockManager on write paths

## Preconditions

- Repository is in detached HEAD at 6a0c2ed
- Working copy has modifications from previous review work

## Postconditions

- Build passes
- Tests pass
- No silent data loss in event processing

## Implementation Notes

The functional-rust agent should:
1. First run `cargo check` to see current errors
2. Fix each issue in order of criticality
3. Run `cargo check` after each fix to verify progress
4. Run `cargo test` at the end to verify all tests pass
