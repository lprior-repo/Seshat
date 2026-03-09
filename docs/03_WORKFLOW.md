# Workflow: Verify -> Merge

1. **Discover**: Find beads via `bv` or issue tracker.
2. **Verify**: `moon run :ci-hardening --force`.
3. **Merge**: Commit and push.

## Development Vibe

This entire app is vibe-coded but grounded in serious engineering discipline. 
We test the shit out of everything (E2E tests, mutation testing, property-based testing).
We follow principles laid out by Martin Fowler, Kent Beck, and David Farley:
- Make code as testable as possible.
- Adhere to clear Domain-Driven Design boundaries.
- Treat code quality, code design, and architecture as first-class citizens.
