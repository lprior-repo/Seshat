bead_id: bd-tet
bead_title: keyboard: Add cleanup to global keyboard hook
phase: p0
updated_at: 2026-03-01T00:43:26Z

# Contract: keyboard cleanup

## Preconditions
- Component is mounted
- eval() JavaScript bridge is available

## Postconditions
- Event listeners are registered
- Cleanup function is stored in window.__seshat_global_keyboard_cleanup
- Listeners are removed on unmount

## Invariants
- Exactly one keydown listener active at any time
- No listener accumulation across re-mounts
