# Contract: bd-104 - recovery-mode: add integrity check and read-only recovery workflow

## Metadata
- bead_id: bd-104
- bead_title: recovery-mode: add integrity check and read-only recovery workflow
- phase: p0
- updated_at: 2026-03-01T16:02:00Z

## Preconditions
- Rust Contract Signature: `fn startup_integrity_check(db_path: &Path) -> Result<IntegrityStatus, RecoveryError>`
- Rust Error Contract: `enum RecoveryError { CorruptDatabase, BackupUnavailable, Io, Sqlite }`
- Legacy code path for this slice is identified and removable in one commit

## Postconditions
- Rust Contract Signature: `fn open_recovery_mode(db_path: &Path) -> Result<RecoveryHandle, RecoveryError>`
- Legacy path is deleted or unreachable by compile-time guarantees
- Replacement path passes focused tests with no fallback to removed code

## Invariants
- No migration path is introduced
- No dual-write compatibility path exists
- All fallible operations use typed Result errors

## Implementation Tasks
1. Run integrity check before accepting write operations
2. Expose read-only mode capabilities including JSON export and diagnostics
