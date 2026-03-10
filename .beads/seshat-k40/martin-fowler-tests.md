# Martin Fowler Test Plan: seshat-k40

## Overview
Test plan for refactoring `store_async.rs` to use Scott Wlaschin DDD types (`ValidEvent`, `BoundedBatch`, `Revision`) instead of primitive inputs, and updating downstream call sites to parse at boundaries.

## DSL Layer (Test Abstraction)

To insulate tests from implementation details and provide a behavior-driven DSL:

```rust
/// Test DSL for store operations
mod store_test_dsl {
    use crate::store_async::*;
    use crate::store::types::*;
    
    /// Parse raw inputs into ValidEvent at boundary
    pub fn valid_event(
        op_id: impl Into<String>,
        timestamp: u64,
        payload: impl Into<String>,
    ) -> Result<ValidEvent, AsyncStoreError> {
        let op_id = ValidOperationId::new(op_id.into())?;
        let timestamp = ValidTimestamp::new(timestamp)?;
        let payload = ValidPayload::new(payload.into())?;
        Ok(ValidEvent { op_id, timestamp, payload })
    }
    
    /// Parse events into bounded batch at boundary  
    pub fn bounded_batch<MIN, MAX>(
        events: Vec<ValidEvent>
    ) -> Result<BoundedBatch<MIN, MAX>, AsyncStoreError>
    where MIN: ArrayLength<ValidEvent>, MAX: ArrayLength<ValidEvent> {
        BoundedBatch::try_from(events)
    }
    
    /// Parse revision at boundary
    pub fn revision(rev: i64) -> Result<Revision, AsyncStoreError> {
        Revision::new(rev)
    }
    
    /// Append single event to store
    pub async fn append_event(
        pool: &SqlitePool,
        event: ValidEvent,
        expected: Option<Revision>,
    ) -> Result<AsyncAppendResult, AsyncStoreError> {
        append_event_async(pool, event, expected).await
    }
    
    /// Append batch to store
    pub async fn append_batch<MIN, MAX>(
        pool: &SqlitePool,
        batch: BoundedBatch<MIN, MAX>,
        expected: Option<Revision>,
    ) -> Result<AsyncBatchAppendResult, AsyncStoreError>
    where MIN: ArrayLength<ValidEvent>, MAX: ArrayLength<ValidEvent> {
        append_batch_async(pool, batch, expected).await
    }
}
```

## Test Organization

### Test Categories (North BDD Style)
1. **Behavior Tests** - Behavioral descriptions following GWT format
2. **Happy Path Tests** - Valid inputs produce expected results  
3. **Error Path Tests** - Invalid inputs produce appropriate errors
4. **Edge Case Tests** - Boundary conditions and extremes
5. **Contract Verification Tests** - Preconditions, postconditions, invariants
6. **Integration Tests** - End-to-end scenarios with downstream callers

---

## Behavior Tests (Dan North BDD Format)

### Feature: Event Append Operations

#### Scenario: Successfully appending a valid event
**Given** a store with valid connection pool  
**And** a valid event with non-empty operation ID, non-zero timestamp, and bounded payload  
**When** the event is appended to the store  
**Then** the operation succeeds with incremented revision  
**And** the returned op_id matches the input  
**And** the returned timestamp matches the input  

#### Scenario: Failing to append when revision mismatch
**Given** a store at revision 5  
**And** expected revision is set to Some(10)  
**When** attempting to append an event  
**Then** the operation fails with RevisionMismatch error  

#### Scenario: Successfully appending a bounded batch
**Given** a store with valid connection pool  
**And** a bounded batch containing 3 valid events  
**When** the batch is appended to the store  
**Then** the operation succeeds with count matching batch size  
**And** revisions are continuous from start to end  

#### Scenario: Rejecting empty batch at boundary
**Given** an empty vector of events  
**When** attempting to construct a BoundedBatch  
**Then** the construction fails with EmptyBatch error  

#### Scenario: Rejecting oversized batch at boundary
**Given** a vector of events exceeding MAX size  
**When** attempting to construct a BoundedBatch  
**Then** the construction fails with BatchTooLarge error  

---

## Happy Path Tests (Behavior-Driven)

### behavior: appending single event with valid types

**Given** a ValidEvent with valid op_id "op-1", timestamp 1700000000, payload "test data"  
**When** calling `append_event_async` with the ValidEvent  
**Then** returns `Ok(AsyncAppendResult)`  
**And** revision equals previous + 1  
**And** op_id matches input "op-1"  
**And** timestamp matches input 1700000000  

### behavior: appending event with matching expected revision

**Given** store at revision 5, expected_revision = Some(Revision(5))  
**When** calling `append_event_async` with matching expected revision  
**Then** returns success with revision 6  

### behavior: appending bounded batch successfully

**Given** BoundedBatch<1, 1000> with 3 valid events  
**When** calling `append_batch_async` with the batch  
**Then** returns `Ok(AsyncBatchAppendResult)`  
**And** count equals 3  
**And** start_revision = previous + 1  
**And** end_revision = start + 2  

### behavior: appending batch at minimum size boundary

**Given** BoundedBatch<1, 1000> with exactly MIN (1) events  
**When** calling `append_batch_async`  
**Then** returns success with count = 1  

### behavior: appending batch at maximum size boundary  

**Given** BoundedBatch<1, 1000> with exactly MAX (1000) events  
**When** calling `append_batch_async`  
**Then** returns success with count = 1000  

### behavior: parsing valid event at boundary

**Given** valid primitive inputs (op_id: "op-1", timestamp: 1700000000, payload: "data")  
**When** calling `parse_valid_event` (or DSL equivalent)  
**Then** returns `Ok(ValidEvent)` with matching fields  

### behavior: parsing bounded batch at boundary

**Given** Vec<ValidEvent> with 5 events  
**When** calling `parse_bounded_batch::<1, 100>  
**Then** returns `Ok(BoundedBatch<1, 100>)` with len() == 5  

### behavior: parsing valid revision at boundary

**Given** non-negative i64 value (42)  
**When** calling `parse_revision(42)` (or DSL equivalent)  
**Then** returns `Ok(Revision(42))`  

---

## Error Path Tests

### behavior: rejecting zero timestamp

**Given** ValidEvent with timestamp = 0  
**When** calling `ValidTimestamp::new(0)`  
**Then** returns `Err(AsyncStoreError::InvalidTimestamp)`  

### behavior: rejecting empty operation ID

**Given** ValidEvent with op_id = ""  
**When** calling `ValidOperationId::new("".to_string())`  
**Then** returns `Err(AsyncStoreError::InvalidOperationId)`  

### behavior: rejecting whitespace-only operation ID

**Given** ValidEvent with op_id = "   " (whitespace only)  
**When** calling `ValidOperationId::new("   ".to_string())`  
**Then** returns `Err(AsyncStoreError::InvalidOperationId)`  

### behavior: rejecting null byte in operation ID

**Given** ValidEvent with op_id containing null byte  
**When** calling `ValidOperationId::new("op\0id".to_string())`  
**Then** returns `Err(AsyncStoreError::InvalidOperationId)`  

### behavior: rejecting operation ID exceeding 255 bytes

**Given** ValidEvent with op_id > 255 bytes  
**When** calling `ValidOperationId::new(long_string)`  
**Then** returns `Err(AsyncStoreError::OperationIdTooLong)`  

### behavior: rejecting payload exceeding 100MB

**Given** ValidEvent with payload > 100MB  
**When** calling `ValidPayload::new(large_string)`  
**Then** returns `Err(AsyncStoreError::PayloadTooLarge)`  

### behavior: rejecting negative revision

**Given** Revision::new(-1)  
**When** constructing Revision with negative value  
**Then** returns `Err(AsyncStoreError::ValidationFailed("Revision cannot be negative"))`  

### behavior: rejecting batch smaller than MIN

**Given** Vec<ValidEvent> with size 0 (less than MIN=1)  
**When** constructing BoundedBatch<1, 1000>  
**Then** returns `Err(AsyncStoreError::EmptyBatch)`  

### behavior: rejecting batch larger than MAX

**Given** Vec<ValidEvent> with size 1001 (greater than MAX=1000)  
**When** constructing BoundedBatch<1, 1000>  
**Then** returns `Err(AsyncStoreError::BatchTooLarge)`  

### behavior: rejecting revision mismatch on append

**Given** store at revision 5, expected_revision = Some(10)  
**When** calling append_event_async  
**Then** returns `Err(AsyncStoreError::RevisionMismatch { expected: 10, found: 5 })`  

---

## Edge Case Tests

### behavior: initial store revision is zero

**Given** a newly created store  
**When** querying current revision  
**Then** returns 0  

### behavior: appending at revision zero with expected zero

**Given** store at revision 0, expected_revision = Some(0)  
**When** appending event  
**Then** succeeds with revision 1  

### behavior: handling concurrent appends maintains revision order

**Given** multiple concurrent append operations  
**When** all complete successfully  
**Then** each gets unique sequential revision  

### behavior: empty payload is valid

**Given** ValidEvent with empty string payload  
**When** constructing ValidPayload  
**Then** returns Ok(ValidPayload(""))  

### behavior: payload at exactly 100MB boundary

**Given** payload of exactly 100 * 1024 * 1024 bytes  
**When** constructing ValidPayload  
**Then** returns Ok (at exactly max)  

### behavior: payload exceeding 100MB by one byte fails

**Given** payload of 100 * 1024 * 1024 + 1 bytes  
**When** constructing ValidPayload  
**Then** returns Err(PayloadTooLarge)  

---

## Contract Verification Tests

### Contract P1: ValidEvent type enforcement

**Given** compile-time type checking  
**When** attempting to pass raw primitives to append_event_async  
**Then** compile error (type mismatch)  

### Contract P2: Revision non-negative enforcement

**Given** Option<Revision> with negative value  
**When** constructing Revision with negative value  
**Then** returns Err at construction time  

### Contract P3: BoundedBatch size enforcement

**Given** Vec<ValidEvent> exceeding MAX  
**When** constructing BoundedBatch  
**Then** returns Err(AsyncStoreError::BatchTooLarge)  

### Contract P4: Batch non-empty enforcement

**Given** empty Vec<ValidEvent>  
**When** constructing BoundedBatch with MIN >= 1  
**Then** returns Err(AsyncStoreError::EmptyBatch)  

### Contract Q1: AppendOutcome revision increment

**Given** store at revision N  
**When** appending valid event  
**Then** result.revision == N + 1  

### Contract Q2: Batch result count matches

**Given** batch of size K  
**When** appending batch  
**Then** result.count == K  

### Contract Q3: Revision monotonicity

**Given** multiple sequential appends  
**When** reading revisions  
**Then** revisions are strictly increasing without gaps  

### Contract I1: Store revision always non-negative

**Given** any stored event  
**When** reading revision column  
**Then** value >= 0  

### Contract I2: No duplicate operation IDs

**Given** attempting to append event with existing op_id  
**Then** returns DuplicateWithConflict error  

---

## Integration Tests (Real Database)

### behavior: end-to-end workflow with DDD types

**Given** real SQLite database with WAL enabled  
**And** valid connection pool  
**When** performing full workflow: create event, append, read, verify  
**Then** all operations succeed  
**And** data integrity maintained  

### behavior: concurrent appends maintain consistency

**Given** multiple tasks appending events concurrently  
**When** all tasks complete  
**Then** no revision gaps  
**And** no data corruption  

### behavior: downstream caller parses at boundary

**Given** store_bridge.rs calling store_async  
**When** converting EventEnvelope to ValidEvent at boundary  
**Then** parsing errors are handled appropriately  

### behavior: export module uses DDD types

**Given** export.rs calling store operations  
**When** events flow through the system  
**Then** ValidEvent types are used throughout  

### behavior: harness module tests DDD integration

**Given** test harness with real database  
**When** running integration test  
**Then** all DDD type contracts verified  

---

## Property-Based Tests (Invariant Verification)

### Invariant I1: Revision monotonicity

**Given** arbitrary sequence of appends  
**When** reading all revisions  
**Then** they form strictly increasing sequence  

### Invariant I3: BoundedBatch length always in bounds

**Given** successfully constructed BoundedBatch  
**When** checking len()  
**Then** MIN <= len() <= MAX always holds
