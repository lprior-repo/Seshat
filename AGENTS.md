# AGENTS

```jsonl
{"skill": "codanna", "description": "Code intelligence - semantic search, symbol lookups, call graphs", "commands": ["codanna init", "codanna index", "codanna serve --watch"], "mcp": "codanna --config .codanna/settings.toml serve --watch"}
{"skill": "moon", "description": "Build system - task running, caching, CI/CD", "commands": ["moon run <task>", "moon :ci-hardening --force", "moon check", "moon test"]}
{"skill": "functional-rust", "description": "Zero panics/unwrap/mut - Data→Calc→Actions pattern", "rules": ["No panics", "No unwrap", "No mut by default", "Result<T, E> for errors"]}
{"skill": "go-skill", "description": "BRCLI-first execution - top-priority bead to main", "workflow": "1. jj new main 2. Pick bead from .beads/issues.jsonl 3. Implement 4. jj commit 5. jj git push"}
{"skill": "landing-skill", "description": "Session completion - validates quality, syncs main, closes bead", "commands": ["/land"]}
{"workflow": "jj new main → codanna serve --watch → moon run :check --force → implement → jj commit → jj git push → /land", "stack": "codanna + moon + functional-rust + go-skill"}
```

To learn more about this project, see `docs/`.
