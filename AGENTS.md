# AGENTS

```jsonl
{"skill": "codanna", "description": "Code intelligence - semantic search, symbol lookups, call graphs", "commands": ["codanna init", "codanna index", "codanna serve --watch"], "mcp": "codanna --config .codanna/settings.toml serve --watch"}
{"skill": "moon", "description": "Build system - task running, caching, CI/CD", "commands": ["moon run <task>", "moon :ci-hardening --force", "moon check", "moon test", "moon run :clippy-source", "moon run :ci-source"]}
{"skill": "functional-rust", "description": "Zero panics/unwrap/mut - Data→Calc→Actions pattern", "rules": ["No panics", "No unwrap", "No mut by default", "Result<T, E> for errors", "clippy-source pipeline for flawless source"]}
{"skill": "go-skill", "description": "BRCLI-first execution - top-priority bead to main", "workflow": "1. jj new main 2. Pick bead from .beads/issues.jsonl 3. Implement 4. jj commit 5. jj git push"}
{"skill": "landing-skill", "description": "Session completion - validates quality, syncs main, closes bead", "commands": ["/land"]}
{"workflow": "jj new main → codanna serve --watch → moon run :check --force → implement → jj commit → jj git push → /land", "stack": "codanna + moon + functional-rust + go-skill"}
```

To learn more about this project, see `docs/`.

<!-- BEGIN BEADS INTEGRATION -->
## Issue Tracking with bd (beads)

**IMPORTANT**: This project uses **bd (beads)** for ALL issue tracking. Do NOT use markdown TODOs, task lists, or other tracking methods.

### Why bd?

- Dependency-aware: Track blockers and relationships between issues
- Git-friendly: Dolt-powered version control with native sync
- Agent-optimized: JSON output, ready work detection, discovered-from links
- Prevents duplicate tracking systems and confusion

### Quick Start

**Check for ready work:**

```bash
bd ready --json
```

**Create new issues:**

```bash
bd create "Issue title" --description="Detailed context" -t bug|feature|task -p 0-4 --json
bd create "Issue title" --description="What this issue is about" -p 1 --deps discovered-from:bd-123 --json
```

**Claim and update:**

```bash
bd update <id> --claim --json
bd update bd-42 --priority 1 --json
```

**Complete work:**

```bash
bd close bd-42 --reason "Completed" --json
```

### Issue Types

- `bug` - Something broken
- `feature` - New functionality
- `task` - Work item (tests, docs, refactoring)
- `epic` - Large feature with subtasks
- `chore` - Maintenance (dependencies, tooling)

### Priorities

- `0` - Critical (security, data loss, broken builds)
- `1` - High (major features, important bugs)
- `2` - Medium (default, nice-to-have)
- `3` - Low (polish, optimization)
- `4` - Backlog (future ideas)

### Workflow for AI Agents

1. **Check ready work**: `bd ready` shows unblocked issues
2. **Claim your task atomically**: `bd update <id> --claim`
3. **Work on it**: Implement, test, document
4. **Discover new work?** Create linked issue:
   - `bd create "Found bug" --description="Details about what was found" -p 1 --deps discovered-from:<parent-id>`
5. **Complete**: `bd close <id> --reason "Done"`

### Auto-Sync

bd automatically syncs via Dolt:

- Each write auto-commits to Dolt history
- Use `bd dolt push`/`bd dolt pull` for remote sync
- No manual export/import needed!

### Important Rules

- ✅ Use bd for ALL task tracking
- ✅ Always use `--json` flag for programmatic use
- ✅ Link discovered work with `discovered-from` dependencies
- ✅ Check `bd ready` before asking "what should I work on?"
- ❌ Do NOT create markdown TODO lists
- ❌ Do NOT use external issue trackers
- ❌ Do NOT duplicate tracking systems

For more details, see README.md and docs/QUICKSTART.md.

<!-- END BEADS INTEGRATION -->

## Landing the Plane (Full Moon Landing)

**When ending a work session**, you MUST complete ALL steps below to execute a "Full Moon Landing". Work is NOT complete until the moon pipeline passes and `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - YOU MUST RUN `moon run :ci-source` to ensure strict functional-rust clippy compliance and flawless tests.
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
