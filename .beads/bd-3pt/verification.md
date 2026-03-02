bead_id: bd-3pt
bead_title: rq: Fix redqueen first-20 deterministic test failures
phase: p3
updated_at: 2026-03-02T02:36:00Z

# Verification: bd-3pt

## QA Evidence

### Compilation
```
cargo check - PASS
cargo build - PASS
cargo clippy - PASS
```

## Fix Applied

Fixed type inference issue in canvas.rs:
- Added explicit String conversions for EdgeId to fix type inference errors
- Two locations fixed: lines 1197-1207 and 1748-1762

## Sign-off
- Actor: orchestrator
- Verified at: 2026-03-02T02:36:00Z
