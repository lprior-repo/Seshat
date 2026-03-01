# Contract: bd-1ik - cli-errors: standardize structured rejection codes and output

## Metadata
- bead_id: bd-1ik
- bead_title: cli-errors: standardize structured rejection codes and output
- phase: p0
- updated_at: 2026-03-01T16:13:00Z

## Preconditions
- Rust Contract Signature: `fn map_error_code(err: &StoreError) -> CliErrorCode`
- Rust Error Contract: `enum StoreError { Sqlite, RevisionMismatch, HumanPriorityBlock, ValidationFailed }`
- Legacy code path for this slice is identified and removable in one commit

## Postconditions
- Rust Contract Signature: `fn render_error_json(code: CliErrorCode, message: &str) -> String`
- Legacy path is deleted or unreachable by compile-time guarantees
- Replacement path passes focused tests with no fallback to removed code

## Invariants
- No migration path is introduced
- No dual-write compatibility path exists
- All fallible operations use typed Result errors

## Implementation Tasks
1. Create error code enum and serializer
2. Wire mapping tests for all public error variants
