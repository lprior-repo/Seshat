# Workflow: Pull -> Isolate -> Verify -> Merge

1. **Pull**: `bv` discover new beads.
2. **Isolate**: `zjj spawn <bead-id>`.
3. **Verify**: `moon run :ci-hardening --force`.
4. **Merge**: `zjj done`.
