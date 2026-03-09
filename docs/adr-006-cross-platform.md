# ADR-006: Cross-Platform via WASM + Platform Renderers

## Status
Accepted

## Date
2026-03-08

## Context
MVP requires shipping on both desktop and web. Desktop users expect native performance; web users expect browser compatibility.

## Decision
We will use **conditional compilation** to target both platforms:

```rust
// Desktop only modules
#[cfg(not(target_arch = "wasm32"))]
pub mod store;

#[cfg(not(target_arch = "wasm32"))]
pub mod cli;

// Shared modules (always compiled)
pub mod models;
pub mod geometry;
pub mod viewport;
```

### Build Targets

| Target | Platform | Renderer |
|--------|----------|----------|
| `dx run` | Desktop | Native WebView |
| `dx serve` | Web | WASM + HTML Canvas |

## Feature Flags

```toml
[features]
default = ["desktop", "server"]
web = ["dioxus/web"]
desktop = ["dioxus/desktop"]
server = ["dioxus/server"]
```

## Consequences

### Positive
- **Single codebase** - 95%+ shared code
- **Same logic** - Bug fixes apply to both
- **Same tests** - Cross-platform verification

### Negative
- **Platform gaps** - Some modules cannot compile to WASM
- **Different I/O** - Desktop has filesystem, web does not
- **Performance variance** - WASM may lag desktop

### Platform-Specific Modules

| Module | Desktop | Web |
|--------|---------|-----|
| `store` | ✅ SQLite | ❌ |
| `cli` | ✅ CLI tools | ❌ |
| `perf` | ✅ benchmarks | ❌ |
| `ui` | ✅ | ✅ |
| `models` | ✅ | ✅ |
