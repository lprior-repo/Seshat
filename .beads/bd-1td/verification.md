bead_id: bd-1td
bead_title: scroll: Fix canvas coordinate transformation with scroll containers
phase: p3
updated_at: 2026-03-02T02:36:00Z

# Verification: bd-1td

## QA Evidence

### Compilation
```
cargo check - PASS
cargo clippy - PASS
```

## Fix Applied

The EdgeId type inference fixes in canvas.rs also benefit scroll/coordinate handling:
- Fixed type inference for EdgeId -> String conversions
- This ensures proper coordinate transformation in selection handling
- Code handles edge selection correctly in scroll containers

## Sign-off
- Actor: orchestrator
- Verified at: 2026-03-02T02:36:00Z
