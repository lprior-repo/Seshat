# ADR-001: Use Dioxus 0.7 for Cross-Platform UI

## Status
Accepted

## Date
2026-03-08

## Context
Seshat needs a cross-platform UI framework that supports both desktop and web from a single codebase. The MVP requires shipping on desktop (Windows, macOS, Linux) and web browsers.

## Decision
We will use **Dioxus 0.7** as the UI framework for Seshat.

## Consequences

### Positive
- **Single codebase** - Desktop and web share 95%+ code
- **Rust-native** - No JavaScript interop overhead, type-safe
- **Reactive** - Fine-grained updates without virtual DOM diffing
- **Familiar patterns** - Similar to React but with Rust's type safety
- **Growing ecosystem** - Active development, good documentation

### Negative
- **Desktop requires WebView** - Depends on system WebView (Edge on Windows, WebKit on macOS/Linux)
- **WASM performance unknown** - May need optimization for 3000+ node diagrams
- **Mobile support immature** - Dioxus mobile is less mature than desktop

### Risks
- Dioxus rendering may not achieve 120 FPS with 3000 nodes - may need virtualization/LOD
- Web WASM performance gap vs desktop - requires cross-platform benchmarking
