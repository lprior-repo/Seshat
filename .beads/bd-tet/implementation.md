bead_id: bd-tet
bead_title: keyboard: Add cleanup to global keyboard hook
phase: p2
updated_at: 2026-03-01T00:43:30Z

# Implementation: keyboard cleanup

## Verification

The cleanup is already implemented in `diagram_tool/src/hooks/keyboard.rs`:

### 1. Cleanup on Effect Re-run (lines 27-29)
```javascript
if (window.__seshat_global_keyboard_cleanup) {
    window.__seshat_global_keyboard_cleanup();
}
```
✅ Cleans up existing listener before adding new one

### 2. Cleanup on Drop/Unmount (lines 104-113)
```javascript
use_drop(move || {
    let _ = document::eval(
        r"
            if (window.__seshat_global_keyboard_cleanup) {
                window.__seshat_global_keyboard_cleanup();
                window.__seshat_global_keyboard_cleanup = null;
            }
        ",
    );
});
```
✅ Removes listener when component unmounts

### 3. Cleanup Function Stored (line 58)
```javascript
window.__seshat_global_keyboard_cleanup = () => {
    window.removeEventListener('keydown', onKeyDown);
};
```

## Acceptance Criteria Coverage
| Criteria | Status |
|----------|--------|
| Event listeners are registered | ✅ |
| Cleanup function is stored | ✅ |
| Listeners are removed on unmount | ✅ |
| Exactly one keydown listener at any time | ✅ |
| No listener accumulation | ✅ |
