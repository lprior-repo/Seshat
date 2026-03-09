# Martin Fowler Test Plan

## Happy Path Tests

### test_write_and_read_branch_metadata_returns_written_value
**Given**: A valid branch name and metadata key-value pairs  
**When**: Writing metadata to a branch, then reading it back  
**Then**: 
- Returns `Ok(Some(metadata))` with the exact key-value pairs written
- All key-value pairs are preserved

### test_write_metadata_replaces_existing_metadata
**Given**: Branch with existing metadata  
**When**: Writing new metadata to the same branch  
**Then**: 
- Returns `Ok(())`
- Subsequent read returns only the new metadata (old is replaced)

### test_delete_removes_metadata
**Given**: Branch with existing metadata  
**When**: Deleting the branch metadata  
**Then**: 
- Returns `Ok(())`
- Subsequent read returns `Ok(None)`

### test_list_tracked_branches_returns_all_branches_with_metadata
**Given**: Multiple branches with metadata  
**When**: Listing all tracked branches  
**Then**: 
- Returns `Ok(Vec<BranchName>)` containing all branches that have metadata
- Does not include branches without metadata

### test_set_and_get_trunk_returns_set_branch
**Given**: No trunk set  
**When**: Setting trunk to a branch, then getting trunk  
**Then**: 
- `set_trunk` returns `Ok(())`
- `get_trunk` returns `Ok(Some(branch_name))` with the set branch

### test_get_trunk_returns_none_when_not_set
**Given**: No trunk has been set  
**When**: Getting trunk  
**Then**: Returns `Ok(None)`

## Error Path Tests

### test_read_returns_metadata_not_found_for_nonexistent_branch
**Given**: Backend with no metadata for branch "nonexistent"  
**When**: Reading metadata for branch "nonexistent"  
**Then**: Returns `Ok(None)` (not an error - metadata absence is a valid state)

### test_delete_returns_metadata_not_found_for_nonexistent_branch
**Given**: Backend with no metadata for branch "nonexistent"  
**When**: Deleting metadata for branch "nonexistent"  
**Then**: Returns `Err(Error::MetadataNotFound { branch: "nonexistent" })`

### test_write_returns_error_for_empty_branch_name
**Given**: An empty string as branch name  
**When**: Attempting to write metadata  
**Then**: Returns `Err(Error::InvalidBranchName { reason: "branch name cannot be empty" })`

### test_write_returns_error_for_too_long_branch_name
**Given**: A branch name exceeding 256 characters  
**When**: Attempting to write metadata  
**Then**: Returns `Err(Error::InvalidBranchName { reason: "branch name exceeds 256 characters" })`

### test_write_returns_error_for_branch_name_with_null_bytes
**Given**: A branch name containing null bytes  
**When**: Attempting to write metadata  
**Then**: Returns `Err(Error::InvalidBranchName { reason: "branch name cannot contain null bytes" })`

### test_get_trunk_returns_trunk_not_set_when_not_configured
**Given**: Backend where trunk has never been set  
**When**: Getting trunk  
**Then**: Returns `Err(Error::TrunkNotSet)`

### test_backend_returns_storage_error_on_io_failure
**Given**: Backend configured with invalid path  
**When**: Any operation is attempted  
**Then**: Returns `Err(Error::StorageError { message: "..." })`

### test_uninitialized_backend_returns_not_initialized_error
**Given**: Backend instance that has not been initialized  
**When**: Any operation is attempted  
**Then**: Returns `Err(Error::NotInitialized)`

## Edge Case Tests

### test_handles_empty_metadata_gracefully
**Given**: A valid branch name  
**When**: Writing empty metadata `HashMap::new()`  
**Then**: 
- Returns `Ok(())`
- Read returns `Ok(Some(HashMap::new()))`

### test_handles_special_characters_in_branch_name
**Given**: Branch names with special characters (hyphens, underscores, slashes)  
**When**: Writing and reading metadata  
**Then**: Returns correct metadata for each branch

### test_handles_unicode_branch_names
**Given**: Branch names with unicode characters  
**When**: Writing and reading metadata  
**Then**: Returns correct metadata (assuming UTF-8 support)

### test_list_returns_empty_vec_when_no_branches_tracked
**Given**: Backend with no tracked branches  
**When**: Listing tracked branches  
**Then**: Returns `Ok(Vec::new())`

### test_consecutive_operations_on_same_branch
**Given**: A branch  
**When**: Writing, reading, deleting, reading in sequence  
**Then**: 
- First read returns written metadata
- Second read after delete returns None

### test_set_trunk_to_different_branch_updates_trunk
**Given**: Trunk is set to "main"  
**When**: Setting trunk to "develop"  
**Then**: 
- Returns `Ok(())`
- Get trunk returns "develop"

## Contract Verification Tests

### test_precondition_branch_name_non_empty_enforced_at_compile_time
**Given**: Using `BranchName::new()` constructor  
**When**: Passing empty string  
**Then**: Returns `Err(Error::InvalidBranchName{..})` (compile-time pattern would use Result-returning constructor)

### test_precondition_branch_name_length_enforced
**Given**: `BranchName::new()` constructor  
**When**: Passing string longer than 256 characters  
**Then**: Returns `Err(Error::InvalidBranchName{..})`

### test_precondition_no_null_bytes_enforced
**Given**: `BranchName::new()` constructor  
**When**: Passing string with null bytes  
**Then**: Returns `Err(Error::InvalidBranchName{..})`

### test_postcondition_write_then_read_returns_written_value
**Given**: Valid branch and metadata  
**When**: Writing metadata, then reading without any intervening operations  
**Then**: Read returns `Ok(Some(written_metadata))`

### test_postcondition_delete_then_read_returns_none
**Given**: Branch with metadata  
**When**: Deleting metadata, then reading  
**Then**: Read returns `Ok(None)`

### test_postcondition_set_trunk_then_get_returns_set_value
**Given**: Valid branch name  
**When**: Setting trunk, then getting trunk  
**Then**: Get returns `Ok(Some(set_branch))`

### test_invariant_no_metadata_for_nonexistent_branches
**Given**: Never written to branch "ghost"  
**When**: Reading from "ghost"  
**Then**: Returns `Ok(None)` (not an error)

### test_invariant_trunk_is_valid_branch
**Given**: Setting trunk to a branch  
**When**: Getting trunk  
**Then**: Returns valid `BranchName` (non-empty, no null bytes)

## Contract Violation Tests

(One test per violation example in contract.md - verifying Err variants are returned, NOT panics)

### test_violates_p1_empty_branch_name_returns_invalid_branch_name_error
**Given**: Empty string as branch name  
**When**: `BranchName::new("")`  
**Then**: Returns `Err(Error::InvalidBranchName { reason: .. })` - NOT a panic

### test_violates_p2_branch_name_too_long_returns_invalid_branch_name_error
**Given**: String with 257+ characters  
**When**: `BranchName::new("a".repeat(257))`  
**Then**: Returns `Err(Error::InvalidBranchName { reason: .. })` - NOT a panic

### test_violates_p3_null_bytes_in_name_returns_invalid_branch_name_error
**Given**: String with null bytes  
**When**: `BranchName::new("branch\0name")`  
**Then**: Returns `Err(Error::InvalidBranchName { reason: .. })` - NOT a panic

### test_violates_p6_uninitialized_backend_returns_not_initialized_error
**Given**: Uninitialized backend  
**When**: Any operation (`read_branch_metadata`, `write_branch_metadata`, etc.)  
**Then**: Returns `Err(Error::NotInitialized)` - NOT a panic

### test_violates_q1_metadata_not_readable_after_write_is_contract_violation
**Given**: Backend with potential write persistence bug  
**When**: Write then immediate read  
**Then**: Read MUST return `Some(metadata)` - if returns `None`, this is a contract violation

### test_violates_q2_metadata_exists_after_delete_is_contract_violation  
**Given**: Backend with potential delete bug  
**When**: Write then delete then read  
**Then**: Read MUST return `None` - if returns `Some`, this is a contract violation

### test_violates_q3_trunk_not_gettable_after_set_is_contract_violation
**Given**: Backend with potential trunk persistence bug  
**When**: Set trunk then get trunk  
**Then**: Get MUST return `Some(branch)` - if returns `None`, this is a contract violation

## Given-When-Then Scenarios

### Scenario 1: Complete Branch Metadata Lifecycle
**Given**: A new branch "feature-login" in a repository  
**When**: 
1. Writing metadata `{"owner": "alice", "ticket": "123", "status": "in-progress"}`
2. Reading metadata
3. Updating status to "merged"
4. Reading again
5. Deleting metadata
6. Reading final state
**Then**:
- Step 2 returns the original metadata
- Step 4 returns updated metadata with new status
- Step 6 returns None

### Scenario 2: Setting Up New Repository Trunk
**Given**: A fresh repository with no trunk configured  
**When**:
1. Attempting to get trunk
2. Setting trunk to "main"
3. Getting trunk again
4. Changing trunk to "develop"
**Then**:
- Step 1 returns `Err(Error::TrunkNotSet)`
- Step 2 returns `Ok(())`
- Step 3 returns `Ok(Some("main"))`
- Step 4 returns `Ok(())`

### Scenario 3: Git Backend Namespace Verification
**Given**: A GitMetadataBackend configured with repository path  
**When**: Writing metadata to branch "my-feature"  
**Then**: 
- Internal storage uses `refs/branch-metadata/my-feature` namespace
- Metadata is stored as ref content (not in regular refs/heads)

### Scenario 4: JJ Backend Bookmark Verification
**Given**: A JujutsuMetadataBackend configured with repository path  
**When**: Writing metadata to branch "my-feature"  
**Then**:
- Internal storage uses JJ bookmarks with metadata prefix
- Or uses custom JJ metadata storage mechanism

### Scenario 5: Concurrent Metadata Modification (Conflict Detection)
**Given**: Two concurrent writes to the same branch  
**When**: 
1. Backend A reads metadata
2. Backend B reads metadata  
3. Both write different metadata
**Then**:
- At least one write should fail with `Err(Error::Conflict { .. })` OR
- Last-write-wins semantics with no error (documented behavior)

## Test Naming Convention
All tests follow the pattern: `test_<what_is_being_tested>`

Tests are organized into categories:
- `test_*_returns_*` - for error path tests
- `test_*_handles_*` - for edge case tests  
- `test_precondition_*` - for contract verification
- `test_postcondition_*` - for contract verification
- `test_invariant_*` - for invariant verification
- `test_violates_*` - for violation example tests
