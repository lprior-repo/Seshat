# Contract: bd-ohy - Per-diagram Queue and File Lock Discipline

## Overview

This contract implements a locking system for diagram mutations that:
1. Serializes mutations per-diagram to prevent concurrent modification conflicts
2. Uses OS-level file locking for cross-process safety
3. Allows parallel work across different diagrams
4. Integrates with the atomic persistence from cli_persistence.rs

## Core Requirements

### 1. Per-diagram Mutation Serialization

**Requirement**: Each diagram must have its own queue to serialize mutations and prevent trampling.

**Implementation**:
- `DiagramLockManager` manages multiple `DiagramQueue` instances
- Each `DiagramQueue` holds a FIFO queue of pending mutations
- Mutations for the same diagram are executed sequentially
- Different diagrams can be processed in parallel

### 2. Per-file Lock Handling

**Requirement**: File-level locking to prevent concurrent access to diagram files.

**Implementation**:
- Use OS-level file locking via `flock`-style operations
- `FileLock` struct provides acquire/release semantics
- Lock acquisition must have configurable timeouts to prevent deadlocks
- Proper cleanup on drop to avoid stale locks

### 3. Parallel Work Across Diagrams

**Requirement**: Allow parallel work on different diagrams.

**Implementation**:
- HashMap of diagram_id -> DiagramQueue
- Independent queues allow parallel processing
- Use tokio for async coordination if needed

### 4. Integration with Existing Storage

**Requirement**: Use the atomic persistence from cli_persistence.rs.

**Implementation**:
- `DiagramLockManager` uses `save_workspace_atomic` for persistence
- `DiagramLockManager` uses `load_workspace_with_lkg` for loading
- Lock must be held during the entire read-modify-write cycle

## Technical Design

### Data Types

```rust
// Newtype for Diagram Identifier
struct DiagramId(String);

// Error types for locking operations
#[derive(Debug, Error)]
pub enum LockError {
    #[error("Lock acquisition timeout for diagram: {0}")]
    Timeout(String),
    #[error("Lock release failed: {0}")]
    ReleaseError(String),
    #[error("IO error during lock operation: {0}")]
    IoError(#[from] std::io::Error),
}

// File lock with timeout
pub struct FileLock {
    path: PathBuf,
    file: File,
}

// Per-diagram queue
struct DiagramQueue {
    id: DiagramId,
    mutations: Vec<Mutation>,
}

// Lock manager for all diagrams
pub struct DiagramLockManager {
    queues: HashMap<DiagramId, DiagramQueue>,
    lock_timeout: Duration,
}
```

### Public API

```rust
impl DiagramLockManager {
    /// Create a new lock manager
    pub fn new(lock_timeout: Duration) -> Self;
    
    /// Execute a mutation on a diagram with locking
    pub fn with_lock<T>(
        &mut self,
        diagram_id: DiagramId,
        operation: impl FnOnce(&mut DiagramDocument) -> Result<T, MutationError>,
    ) -> Result<T, LockError>;
    
    /// Check if a diagram is currently locked
    pub fn is_locked(&self, diagram_id: &DiagramId) -> bool;
    
    /// Get the number of pending mutations for a diagram
    pub fn queue_length(&self, diagram_id: &DiagramId) -> usize;
}
```

## Error Handling

- All fallible operations return `Result<T, Error>`
- Zero `unwrap` or `expect` calls in source code
- Proper error codes for lock failures:
  - `LockError::Timeout` - Lock could not be acquired within timeout
  - `LockError::ReleaseError` - Lock could not be released
  - `LockError::IoError` - Underlying I/O error

## Acceptance Criteria

1. **Concurrent mutations to same diagram serialized**: Multiple mutations to the same diagram must be executed sequentially, not concurrently.

2. **File lock acquisition and release**: File locks must be properly acquired before mutation and released after.

3. **Parallel work on different diagrams**: Mutations to different diagrams can execute in parallel.

4. **Integration with storage**: Uses `save_workspace_atomic` and `load_workspace_with_lkg` from cli_persistence.

5. **Timeout handling**: Lock acquisition respects timeout and returns error on timeout.

6. **Clean resource management**: Locks are released even on errors (RAII pattern).

## Test Scenarios

1. `given_two_mutations_same_diagram_when_executed_sequentially_then_succeeds` - Serialization
2. `given_lock_timeout_when_acquire_then_returns_error` - Timeout handling
3. `given_different_diagrams_when_mutated_in_parallel_then_succeeds` - Parallel execution
4. `given_locked_diagram_when_check_locked_then_returns_true` - State query
5. `given_file_lock_when_dropped_then_releases` - RAII cleanup
