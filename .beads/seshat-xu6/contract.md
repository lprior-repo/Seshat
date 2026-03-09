# Contract Specification

## Context
- **Feature**: MetadataBackend trait for dual Git/JJ branch metadata support
- **Bead**: seshat-xu6
- **Domain terms**:
  - `BranchName`: Identifier for a branch (non-empty string, max 256 chars)
  - `BranchMetadata`: Key-value pairs associated with a branch (JSON-serializable)
  - `Trunk`: The primary/main branch in the repository
  - `MetadataBackend`: Trait abstracting branch metadata storage operations
- **Assumptions**:
  - Git backend uses `refs/branch-metadata/<branch_name>` namespace
  - JJ backend uses bookmarks with `branch-metadata:` prefix or custom metadata storage
  - Both backends provide equivalent semantics for metadata operations
- **Open questions**:
  - What is the maximum size for metadata values?
  - Should metadata support nested/structured values or only flat key-value?
  - Is there a specific serialization format required (JSON, MessagePack, etc.)?

## Preconditions

| ID | Precondition | Enforcement Level | Type/Pattern |
|----|--------------|-------------------|---------------|
| P1 | Branch name must be non-empty | Compile-time | `NonEmptyString` wrapper type |
| P2 | Branch name must not exceed 256 characters | Compile-time | `new()` returns `Result<BranchName, Error>` |
| P3 | Branch name must not contain null bytes | Compile-time | Validated in `BranchName::new()` |
| P4 | Metadata key must be non-empty | Compile-time | `NonEmptyString` wrapper type |
| P5 | Metadata value must be serializable | Runtime-checked | `serde::Serialize` bound in trait |
| P6 | Backend must be initialized before use | Runtime-checked | `Result` error if not initialized |

## Postconditions

| ID | Postcondition | Enforcement Level |
|----|---------------|-------------------|
| Q1 | After `write_branch_metadata`, subsequent `read_branch_metadata` returns the written value | Result verification |
| Q2 | After `delete_branch_metadata`, subsequent `read_branch_metadata` returns `Err(MetadataNotFound)` | Result verification |
| Q3 | After `set_trunk`, `get_trunk` returns the set branch name | Result verification |
| Q4 | `list_tracked_branches` returns all branches with metadata, including trunk if set | Result verification |
| Q5 | All mutations preserve backend invariants (no partial writes) | Atomic operations |

## Invariants

| ID | Invariant | Enforcement |
|----|-----------|-------------|
| I1 | Backend state is consistent: either all operations succeed or fail atomically | Backend implementation |
| I2 | No metadata exists for non-existent branches | Read returns `MetadataNotFound` |
| I3 | Trunk is always a valid branch name if set | Validated in `set_trunk` |
| I4 | List operations reflect current state, not cached stale data | Backend implementation |

## Error Taxonomy

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Branch name is invalid (empty, too long, or contains invalid characters)
    InvalidBranchName { reason: String },
    
    /// Metadata for the specified branch does not exist
    MetadataNotFound { branch: BranchName },
    
    /// Attempted to delete non-existent metadata
    MetadataNotFound { branch: BranchName },
    
    /// Trunk branch has not been set
    TrunkNotSet,
    
    /// Storage backend encountered an error (IO, network, etc.)
    StorageError { message: String },
    
    /// Backend is not initialized
    NotInitialized,
    
    /// Concurrent modification conflict detected
    Conflict { message: String },
    
    /// Serialization/deserialization error
    SerializationError { message: String },
}
```

## Contract Signatures

```rust
/// Metadata associated with a branch
pub type BranchMetadata = HashMap<String, String>;

/// Branch name wrapper ensuring validity
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BranchName(String);

impl BranchName {
    /// Creates a new BranchName, returns Error if invalid
    pub fn new(name: impl Into<String>) -> Result<Self, Error>;
    
    /// Returns the underlying string
    pub fn as_str(&self) -> &str;
}

/// Result type alias for this module
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Trait for abstracting branch metadata storage
pub trait MetadataBackend: Send + Sync {
    /// Read metadata for a specific branch
    fn read_branch_metadata(&self, branch: &BranchName) -> Result<Option<BranchMetadata>>;
    
    /// Write metadata for a specific branch (replaces existing)
    fn write_branch_metadata(&self, branch: &BranchName, metadata: BranchMetadata) -> Result<()>;
    
    /// Delete metadata for a specific branch
    fn delete_branch_metadata(&self, branch: &BranchName) -> Result<()>;
    
    /// List all branches that have metadata stored
    fn list_tracked_branches(&self) -> Result<Vec<BranchName>>;
    
    /// Get the current trunk branch
    fn get_trunk(&self) -> Result<Option<BranchName>>;
    
    /// Set the trunk branch
    fn set_trunk(&self, branch: &BranchName) -> Result<()>;
}
```

## Ownership Contracts

| Parameter | Type | Ownership | Mutation Contract |
|-----------|------|-----------|-------------------|
| `branch` | `&BranchName` | Shared borrow (read-only) | No mutation - function reads only |
| `metadata` | `BranchMetadata` | Ownership transfer | Caller transfers ownership; backend decides storage (copy or reference) |
| `self` | `&dyn MetadataBackend` | Shared borrow | No mutation to backend state; operations are idempotent queries |
| Return `Vec<BranchName>` | `Vec<BranchName>` | Ownership transfer | Caller receives ownership of new allocation |

### Clone Policy
- `BranchName`: Does NOT clone internally; uses `String` internally with potential clone on return
- `BranchMetadata`: Does NOT clone; caller transfers ownership in, backend transfers ownership out
- Trait object: No cloning; uses dynamic dispatch

## Violation Examples (REQUIRED)

### Precondition Violations

- **VIOLATES P1 (empty branch name)**: 
  ```rust
  backend.write_branch_metadata(&BranchName::new("").unwrap(), HashMap::new())
  ```
  Should produce: `Err(Error::InvalidBranchName { reason: ... })`

- **VIOLATES P2 (branch name too long)**:
  ```rust
  let long_name = "a".repeat(257);
  backend.write_branch_metadata(&BranchName::new(long_name).unwrap(), HashMap::new())
  ```
  Should produce: `Err(Error::InvalidBranchName { reason: ... })`

- **VIOLATES P3 (null bytes in name)**:
  ```rust
  backend.write_branch_metadata(&BranchName::new("branch\0name").unwrap(), HashMap::new())
  ```
  Should produce: `Err(Error::InvalidBranchName { reason: ... })`

- **VIOLATES P6 (backend not initialized)**:
  ```rust
  let uninit_backend = GitMetadataBackend::new(); // without calling init()
  uninit_backend.read_branch_metadata(&BranchName::new("main").unwrap())
  ```
  Should produce: `Err(Error::NotInitialized)`

### Postcondition Violations

- **VIOLATES Q1 (metadata not readable after write)**:
  ```rust
  let branch = BranchName::new("feature").unwrap();
  backend.write_branch_metadata(&branch, hashmap!["key".to_string() => "value".to_string()]).unwrap();
  // Assume backend has a bug where write doesn't persist
  let result = backend.read_branch_metadata(&branch);
  assert!(result.unwrap().is_none()); // Bug: should be Some
  ```
  Should produce: `Err(Error::MetadataNotFound)` (if bug exists) or return `Some(...)` (correct)

- **VIOLATES Q2 (metadata still exists after delete)**:
  ```rust
  let branch = BranchName::new("feature").unwrap();
  backend.write_branch_metadata(&branch, hashmap!["key".to_string() => "value".to_string()]).unwrap();
  backend.delete_branch_metadata(&branch).unwrap();
  let result = backend.read_branch_metadata(&branch);
  assert!(result.unwrap().is_some()); // Bug: should be None
  ```
  Should produce: `Err(Error::MetadataNotFound)` (correct) or return `Some(...)` (bug)

- **VIOLATES Q3 (trunk not retrievable after set)**:
  ```rust
  let trunk = BranchName::new("main").unwrap();
  backend.set_trunk(&trunk).unwrap();
  // Assume backend has a bug
  let result = backend.get_trunk();
  assert!(result.unwrap().is_none()); // Bug: should be Some(main)
  ```
  Should produce: `Err(Error::TrunkNotSet)` (if never set) or return `Some(BranchName("main"))` (correct)

## Non-goals
- [ ] Implementing actual Git or JJ backend (only trait specification)
- [ ] Supporting multiple trunks (single trunk per repository)
- [ ] Providing merge semantics for concurrent metadata updates
- [ ] Supporting metadata history/auditing
- [ ] Implementing authentication/authorization for metadata access
