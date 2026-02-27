# Agent Instructions: Autonomous Development Loop

```jsonl
{"kind":"meta","project":"oya-frontend","version":"1.0.0","updated":"2026-02","format":"markdown-with-embedded-jsonl"}
{"kind":"workflow","name":"core_loop","steps":["bv --robot-triage → br claim <bead-id>","zjj spawn <bead-id>","tdd15 + red-queen + functional skills","moon run :ci-hardening --force","zjj done"]}
{"kind":"build_system","tool":"moon","allowed":[":serve",":check",":test",":clippy",":fmt",":build-web",":e2e-smoke",":e2e-full",":ci --force",":ci-hardening --force"],"forbidden":["dx directly","cargo directly","npm directly"]}
{"kind":"framework","name":"dioxus","version":"0.7","state":"use_signal (atomic), use_store (nested)","props":"ReadOnlySignal<T>","styling":"Tailwind class: attribute","closures":"move |_|","assets":"asset!() macro","syntax":"rsx! only, no HTML tags"}
{"kind":"code_intelligence","tool":"codanna","required":"mcp__codanna__*","forbidden":["grep","rg","ripgrep","find","cat for search"],"workflow":["semantic_search_with_context → anchor","find_symbol/search_symbols → lock","get_calls/find_callers/analyze_impact → hints","Read tool → confirm"]}
{"kind":"zero_policy","rules":["no .unwrap() or .expect()","no panic!() or unsafe","Result<T,Error> with combinators"]}
{"kind":"ui_design","approach":"v0","principles":["high-quality Tailwind","ARIA accessibility","keyboard navigation","flat RSX trees","#[component] functions"]}
{"kind":"landing","requirements":["moon run :ci-hardening --force passes","zjj done executed","git push succeeds"]}
{"kind":"tool","name":"semantic_search_with_context","purpose":"Search by concept with full context","when":"Start here for exploration"}
{"kind":"tool","name":"find_symbol","purpose":"Exact symbol lookup","when":"Know the exact name"}
{"kind":"tool","name":"search_symbols","purpose":"Fuzzy text search","when":"Partial name matches"}
{"kind":"tool","name":"get_calls","purpose":"What function calls","when":"Call graph outbound"}
{"kind":"tool","name":"find_callers","purpose":"What calls function","when":"Call graph inbound"}
{"kind":"tool","name":"analyze_impact","purpose":"Full dependency graph","when":"Change impact analysis"}
{"kind":"tool","name":"search_documents","purpose":"Search markdown/docs","when":"Find project docs"}
{"kind":"tool","name":"get_index_info","purpose":"Index statistics","when":"Verify index health"}
```

## Quick Reference

| Area | Rule |
|------|------|
| **Triage** | `bv --robot-triage` → `br claim <id>` → `zjj spawn <id>` |
| **Build** | `moon run :ci-hardening --force` (mandatory `--force`) |
| **State** | `use_signal` / `use_store`, NEVER `use_state` |
| **Code Search** | Codanna MCP tools ONLY, no grep/rg/find |
| **Errors** | `Result<T, Error>` with combinators, no unwrap |

## Workspace Layout

This is a Cargo workspace with two members:

| Crate | Path | Target | Purpose |
|-------|------|--------|---------|
| `diagram_tool` | `diagram_tool/` | Desktop (native) | Primary development target — Dioxus desktop diagram editor |
| `oya-frontend` | `.` | WASM (web) | OYA frontend — Dioxus web app |

### Building

```bash
# Build diagram_tool (default)
cargo build

# Run diagram_tool desktop app
cargo run -p diagram_tool

# Check oya-frontend (requires wasm32 target)
cargo check --package oya-frontend --target wasm32-unknown-unknown

# Run all tests
cargo test --workspace

# Check only diagram_tool
cargo check --package diagram_tool
```

### Cargo.lock files
- `seshat/Cargo.lock` — root workspace lock (used by workspace builds)
- `diagram_tool/Cargo.lock` — kept for historical reference but superseded by workspace lock
