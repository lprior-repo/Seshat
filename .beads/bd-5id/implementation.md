bead_id: bd-5id
bead_title: release-gate: enforce ci hardening checklist
phase: p2
updated_at: 2026-03-01T00:43:00Z

# Implementation: release-gate

## Verification

### ci-hardening Task Definition
```yaml
ci-hardening:
  script: |
    moon run :check
    moon run :test
    moon run :clippy
    moon run :e2e-smoke
    moon run :e2e-full
```

✅ Sequence: check -> test -> clippy -> e2e-smoke -> e2e-full

### No Bypass Commands
All tasks use `moon run :<task>` pattern:
- ✅ No direct cargo in ci-hardening
- ✅ No direct dx in ci-hardening  
- ✅ No direct npm in ci-hardening

### Documentation References
- AGENTS.md: Allowed commands include `:ci-hardening --force`
- docs/02_MOON_BUILD.md: Documents ci-hardening as canonical command
- CLAUDE.md: Documents ci-hardening --force as mandatory

## Acceptance Criteria Coverage
| Criteria | Status |
|----------|--------|
| ci-hardening runs in correct order | ✅ |
| No bypass commands | ✅ |
| All tasks use moon run prefix | ✅ |
