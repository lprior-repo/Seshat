bead_id: bd-1fc
bead_title: docs-sync: align hardening runbooks with normalized moon tasks
phase: p3
updated_at: 2026-02-28T22:04:35Z

# Verification Report

## Documentation Alignment Check

### AGENTS.md
- ✅ Allowed tasks include: `:serve`, `:check`, `:test`, `:clippy`, `:fmt`, `:build-web`, `:e2e-smoke`, `:e2e-full`, `:ci --force`, `:ci-hardening --force`

### CLAUDE.md  
- ✅ Allowed tasks include: `:e2e-full`
- ✅ References `moon run :ci-hardening --force`

### docs/02_MOON_BUILD.md
- ✅ References `moon run :ci-hardening --force`
- ✅ References individual tasks: check, test, clippy, e2e-smoke, e2e-full

### docs/03_WORKFLOW.md
- ✅ References `moon run :ci-hardening --force`

## Assessment
All acceptance criteria satisfied:
1. ✅ Build/runbook docs reference moon run :ci-hardening --force
2. ✅ Agent instruction allowlists include e2e-smoke and e2e-full where applicable
