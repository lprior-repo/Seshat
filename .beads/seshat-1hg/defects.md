# Defects Found - seshat-1hg

## CRITICAL DEFECT

### Duplicate Clipboard Context Providers

**Location:** `diagram_tool/src/app.rs` lines 62, 67, 72, 75

**Issue:** Four identical `use_context_provider(|| Signal::new(Option::<ClipboardData>::None));` calls create 4 separate Signal instances. Only the last one is used by Dioxus context system.

**Code:**
```rust
// line 62
use_context_provider(|| Signal::new(Option::<ClipboardData>::None));
// line 67
use_context_provider(|| Signal::new(Option::<ClipboardData>::None));
// line 72
use_context_provider(|| Signal::new(Option::<ClipboardData>::None));
// line 75
use_context_provider(|| Signal::new(Option::<ClipboardData>::None));
```

**Impact:**
- 3 orphaned signal instances consuming memory
- Copy-paste error indicates code quality issues
- Could cause subtle reactivity bugs if earlier providers are accessed elsewhere
- Violates DRY (Don't Repeat Yourself)

**Fix Required:**
- Remove duplicate providers, keep only ONE `use_context_provider` for clipboard
- Lines 67, 72, 75 should be deleted

## VERIFIED CORRECT

### 1. Signal/Context Usage (Correct)
- `commands.rs` line 235-298: All clipboard functions use `Signal<Option<ClipboardData>>`
- Proper use of Dioxus signals for reactivity

### 2. Pure Functions (Correct)
- `copy_selection` (line 104): Takes `&DiagramDocument`, returns `Option<ClipboardData>`
- `paste_contents` (line 180): Pure transformation, no side effects
- `copy_selection_for_duplicate` (line 142): Pure function
- `clipboard_has_content` (line 95): Pure function

### 3. No thread_local/RefCell (Correct)
- No actual `thread_local` or `RefCell` usage in clipboard code
- Only referenced in documentation/comments

### 4. Operations Return bool (Correct)
- `apply_copy_selection` returns `bool` (line 244)
- `apply_paste_selection` returns `bool` (line 270)
- `apply_duplicate_selection` returns `bool` (line 298)

### 5. No unwrap/panic (Correct)
- File has `#![deny(clippy::unwrap_used)]` at line 1
- No unwrap/expect/panic in clipboard functions