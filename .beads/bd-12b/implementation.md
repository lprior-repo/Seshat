bead_id: bd-12b
bead_title: moon-config: normalize hardening task graph and aliases
phase: p2
updated_at: 2026-02-28T21:59:30Z

# Implementation: moon-config normalization

## Files Changed
- `moon.yml` - Added e2e-full task, updated ci-hardening task order

## Changes Made

### 1. Added e2e-full task
```yaml
e2e-full:
  script: |
    npm exec -- playwright test
  options:
    cache: false
```

### 2. Updated ci-hardening task order
From:
```yaml
ci-hardening:
  script: |
    moon run :ci
    moon run :e2e-hardening
```

To:
```yaml
ci-hardening:
  script: |
    moon run :check
    moon run :test
    moon run :clippy
    moon run :e2e-smoke
    moon run :e2e-full
```

## Acceptance Criteria Coverage
| Criteria | Status |
|----------|--------|
| moon.yml defines explicit e2e-smoke task | ✅ Already existed |
| moon.yml defines explicit e2e-full task | ✅ Added |
| ci-hardening task exists and runs full sequence | ✅ Updated |
| Task aliases match documented hardening pipeline | ✅ check -> test -> clippy -> e2e-smoke -> e2e-full |
