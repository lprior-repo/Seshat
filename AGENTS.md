# OYA-FRONTEND Quality Gate Instructions

```jsonl
{"kind":"meta","project":"oya-frontend","version":"1.0.0","updated":"2026-02","format":"markdown-with-embedded-jsonl"}
{"kind":"visibility","role":"agent","can_see":["specs/flow-wasm-v1.yaml","tests/","src/","specs/linter/rules.yaml","ACCEPTANCE CRITERIA"],"cannot_see":["../scenarios-vault/","holdout assertions","step sequences","raw validation results"]}
{"kind":"framework","name":"dioxus","version":"0.7","reactivity":"use_signal, use_memo, use_resource","state_hierarchy":"use_signal (atomic), use_store + #[derive(Store)] (nested)","props":"ReadOnlySignal<T>","styling":"Tailwind class: attribute","events":"move |_| closures","architecture":"flat RSX, #[component] modular","assets":"asset!() macro"}
{"kind":"code_intelligence","tool":"codanna","required":"mcp__codanna__*","forbidden":["grep","rg","ripgrep","find","cat for search"],"workflow":["semantic_search_with_context → anchor","find_symbol/search_symbols → lock","get_calls/find_callers/analyze_impact → hints","Read tool → confirm"]}
{"kind":"build_system","tool":"moon","allowed":[":serve",":check",":test",":clippy",":fmt",":build-web",":ci --force"],"forbidden":["dx directly","cargo directly","npm directly"]}
{"kind":"quality_gate","approach":"hidden behavioral scenarios","feedback":"sanitized (category, spec reference, hints, no exact values)","purpose":"prevent teaching to test"}
{"kind":"invariants","rules":["never access ../scenarios-vault/","never ask about holdout scenarios","build genuine spec implementation","all acceptance criteria satisfied"]}
{"kind":"directive","text":"Use Dioxus 0.7 Signals and Stores. Never use use_state.","reason":"prevents legacy API, ensures reactivity"}
{"kind":"directive","text":"Use ReadOnlySignal<T> for component props","reason":"proper prop typing, reactive propagation"}
{"kind":"directive","text":"Use use_store with #[derive(Store)] for nested state","reason":"fine-grained reactivity for complex structures"}
{"kind":"directive","text":"Keep RSX trees flat — modularize into components","reason":"maintainability, hot-reloading"}
{"kind":"directive","text":"Use Tailwind utility classes in class: attributes","reason":"standardized styling, rapid prototyping"}
{"kind":"directive","text":"Handle events with move |_| closures","reason":"Signals are Copy, cloning unnecessary"}
{"kind":"directive","text":"Act as v0 for Dioxus","reason":"modern, accessible, perfectly-styled RSX"}
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
| **Workspace** | specs, tests, src visible; scenarios-vault hidden |
| **State** | `use_signal` / `use_store`, NEVER `use_state` |
| **Build** | Moon only, `moon run :ci --force` |
| **Code Search** | Codanna MCP tools ONLY |
| **Quality** | Hidden scenarios, sanitized feedback |
