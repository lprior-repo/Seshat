# seshat-fir Code Review Defects

## Summary
REJECTED - Code violates PHASE 2 (Farley Rigor) hard constraints and PHASE 3/4 (DDD newtypes).

---

## CRITICAL (Must Fix)

### 1. Function Exceeds 25-Line Limit
**Location:** `diagram_tool/src/models/envelope.rs:319-368`  
**Issue:** `parse_node_resize` is 50 lines - exceeds the 25-line hard constraint.  
**Fix Required:** Split into smaller helper functions:
- Extract `validate_node_id(id: &str) -> Result<String, ContractError>` (lines 326-331)
- Extract `validate_dimension(value: f64, name: &str) -> Result<f64, ContractError>` (lines 338-365)

### 2. Primitive Obsession - Missing NodeId Newtype
**Location:** `diagram_tool/src/models/envelope.rs:121-125`  
**Issue:** `NodeResize { id: String, width: f64, height: f64 }` uses raw primitives.  
**Violation:** PHASE 3 "Newtypes" rule - `id` should be `NodeId` newtype.  
**Fix Required:** Create `NodeId` newtype wrapper around `String` with validation in constructor.

---

## RECOMMENDED (Should Fix)

### 3. Consider Dimension Newtypes
**Location:** `diagram_tool/src/models/envelope.rs:121-125`  
**Issue:** `width: f64` and `height: f64` are primitives.  
**Consider:** Create `Width(f64)` and `Height(f64)` newtypes for stronger typing.

---

## Test Coverage Status
✅ All contract tests present and passing (24 tests)  
✅ Violation examples from contract.md covered  
✅ Exhaustive match updated  

## Files Modified
1. `diagram_tool/src/models/envelope.rs` - Main implementation
2. `diagram_tool/src/models/conflict/resolution.rs` - Entity extraction
3. `diagram_tool/src/models/sync.rs` - Entity extraction
