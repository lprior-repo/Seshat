# Contract: bd-mtu - recovery-export

bead_id: bd-mtu
bead_title: recovery-export: support json export while in recovery-only mode
phase: p0
updated_at: 2026-03-01T19:20:00Z

## Overview

Support JSON export functionality while in recovery-only mode.

## Preconditions

- System is in recovery-only mode with read-only access
- Rust Contract Signature: `fn export_while_recovering(conn: &Connection) -> Result<String, ExportError>`

## Postconditions

- Export works in recovery-only mode
- Returns valid JSON even when write operations are blocked

## Implementation Tasks

### Phase 2: Implementation
- Enable export functions to work in recovery mode
- Ensure read-only transactions work correctly

### Phase 4: Verification
- Run moon run :ci
