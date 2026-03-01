# Contract: bd-7rt - recovery-integrity

bead_id: bd-7rt
bead_title: recovery-integrity: gate startup with integrity check and recovery-only mode
phase: p0
updated_at: 2026-03-01T20:47:10Z

## Overview

Gate startup with integrity check and recovery-only mode.

## Preconditions

- Rust Contract Signature: `fn integrity_check(db_path: &Path) -> Result<IntegrityStatus, RecoveryError>`
- Rust Error Contract: `enum RecoveryError { CorruptDatabase, Sqlite, Io, BackupUnavailable }`

## Postconditions

- Rust Postcondition Signature: `fn open_recovery_only(db_path: &Path) -> Result<RecoverySession, RecoveryError>`
- Legacy path is deleted or unreachable by compile-time guarantees
- Replacement path passes focused tests with no fallback to removed code

## Invariants

- No migration path is introduced
- No dual-write compatibility path exists
- All fallible operations use typed Result errors

## Implementation Tasks

### Phase 2: Implementation
- Run integrity check before writable open
- Expose recovery-only status to CLI and UI

### Phase 4: Verification
- Run moon run :ci
