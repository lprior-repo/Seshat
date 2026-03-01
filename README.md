# Seshat

A high-performance, two-way sync diagram tool built in Rust. 

## The Vision

Seshat bridges the gap between human-created diagrams and AI-driven automation. It started because drawing on Miro was great—but there was no way for AI to programmatically update and maintain those diagrams.

- **Humans**: Get a clean, intuitive drag-and-drop UI for creating cloud-native architecture diagrams, workflows, value stream maps, and more.
- **AI**: Can read and write directly to a rigorous SQLite backend, ensuring accurate, version-controlled diagrams.

## The Why

The best diagrams live in version control. D2, Mermaid, and other tools are great—but they lack a proper backend. Miro is great for UI but can't be version-controlled or updated by AI.

Seshat combines:
- **D2** / **Mermaid** - declarative diagramming syntax
- **Escaladra** - AI-native diagramming
- **Miro** - clean human UI

With source control as the source of truth, AI can generate, update, and validate diagrams while humans get a polished interface.

## Key Features

- **Two-Way Sync**: Human UI and AI database always stay in sync
- **Performant**: Built in Rust for speed and correctness
- **Functional Rust**: Zero panics, zero unwrap, data→calc→actions pattern
- **Quality First**: Thoroughly tested and vetted despite being built in spare time
- **Source Control**: Everything in SQLite, fully versioned

## Tech Stack

- **Frontend**: Dioxus (Rust-based UI framework)
- **Backend**: SQLite with rusqlite
- **Build**: Moon for CI/CD
- **VCS**: Jujutsu (jj) for version control
- **Code Intelligence**: Codanna for semantic search

## Quick Start

```bash
# Start the dev server
cd diagram_tool && cargo run

# Or run the web version
cargo run --features web
```

## Development

See [docs/](docs/) for engineering documentation and [AGENTS.md](AGENTS.md) for AI agent integration.
