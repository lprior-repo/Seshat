# Defects Audit: seshat-hly (RE-REVIEW - FAILED)

## ❌ REJECTED - DEFECTS REMAIN

### Remaining Issues

#### 1. Function Length Violation (PHASE 2: Farley Rigor)
| Function | Lines | Limit | Status |
|----------|-------|-------|--------|
| `group_selection` | 48 | 25 | ❌ EXCEEDS |

**Location**: `diagram_tool/src/core/grouping.rs:124-171`

The `group_selection` function is **48 lines** - nearly double the 25-line limit.

**Required fix**: Extract validation logic into helper functions to reduce to ≤25 lines.

---

#### 2. Mutable Variables in Functional Code (PHASE 4: Scott Wlaschin DDD)
| Location | Line | Code |
|----------|------|------|
| `remove_subgraphs_and_reparent` | 212 | `let mut orphaned = BTreeSet::new();` |
| `remove_subgraphs_and_reparent` | 222 | `let mut next = node.clone();` |

**Location**: `diagram_tool/src/core/grouping.rs:212, 222`

These `let mut` declarations violate the functional style requirement. The defects.md claimed these were "FIXED" but they are clearly present.

**Required fix**: 
- Line 212: Use iterator pattern with `.filter_map()` + `.fold()` or collect directly
- Line 222: Use functional `.map()` instead of clone + mutate

---

### What Was Actually Fixed ✅

1. **Compilation** - Code compiles ✅
2. **Error Variants** - All required variants present ✅
3. **Constants** - SUBGRAPH_PADDING, MAX_SUBGRAPH_NESTING_DEPTH extracted ✅
4. **BoundingBox** - Newtype struct added ✅
5. **Safe Constructors** - OrderedFloat::new() used (not new_unchecked) ✅

---

### Verification Commands

```bash
cd /home/lewis/src/seshat/diagram_tool
cargo check  # Compiles
```

### Files Requiring Fix

- `src/core/grouping.rs` - Reduce `group_selection` to ≤25 lines, remove `let mut`
