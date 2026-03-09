# Seshat Vision Document

## The Goal
Seshat bridges the gap between human-created diagrams and AI-driven automation. It started because drawing on Miro was great—but there was no way for AI to programmatically update and maintain those diagrams.

We are building the ultimate tool for designing AI architecture. Seshat combines the best of all worlds:
- **D2 / Mermaid**: Declarative diagramming syntax and native subgraphs.
- **Excalidraw**: AI-native diagramming and clean interaction.
- **Miro**: A polished human UI with drag-and-drop nodes, auto-snapping straight arrows, multi-select, subgraph generation, and auto-arrange capabilities powered by `petgraph` for DAG layouts.

The core principle is a **two-way sync**:
1. **Humans** get an incredibly fast, intuitive frontend built on Dioxus. It handles 3000 nodes at 120 FPS by pushing the DOM as far as possible (avoiding WebGL unless absolutely necessary).
2. **AI** interacts with a rigorous SQLite WAL-mode backend via a CLI, speaking in JSON specifications.

If there's ever a conflict between the human and the AI's architecture spec, an auto-correction layer will either reject the AI's invalid spec based on strict contracts or present a differential to the human, who has the ultimate choice.

## Core Principles

- **Two-Way Sync**: Human UI and AI database always stay in sync, with humans having the final say in conflicts.
- **Performant Frontend**: Built in Dioxus and Rust for speed and correctness, handling 3,000+ nodes smoothly.
- **Advanced Interaction**: Drag-and-drop icons, straight snapping arrows, multi-select subgraphs, and DAG-based auto-arrange.
- **Functional Rust**: Driven by strict engineering rigor—Data → Calculations → Actions. Zero panics, zero unwrap, making illegal states unrepresentable.
- **Testing Rigor**: Inspired by Martin Fowler, Kent Beck, and David Farley. We leverage end-to-end tests, mutation testing, and property-based testing. Deeply focused on code quality, testability, and clear Domain-Driven Design boundaries.
- **Source Control**: Everything backed by SQLite in WAL mode, entirely versioned and verifiable.
