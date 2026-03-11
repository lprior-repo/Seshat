# Seshat Vision Document

## The Goal
Seshat bridges the gap between human-created diagrams and AI-driven automation. It started because drawing on Miro was great—but there was no way for AI to programmatically update and maintain those diagrams. 

We are building the ultimate tool for designing AI architecture. Seshat combines the best of all worlds:
- **D2 / Mermaid**: Declarative diagramming syntax and native subgraphs.
- **Excalidraw**: AI-native diagramming, multiplayer collaboration, and clean interaction.
- **Miro**: A polished human UI with drag-and-drop nodes, auto-snapping straight arrows, multi-select, subgraph generation, and auto-arrange capabilities powered by `petgraph` for DAG layouts.

The core principle is a **Real-Time Two-Way Sync**:
1. **Humans** get an incredibly fast, intuitive frontend built on Dioxus Fullstack. It leverages Optimistic UI to guarantee an **8ms frame budget** (120 FPS) for interactions like dragging nodes, even under heavy load with **up to 12 concurrent human users** modifying a 2000-node graph.
2. **AI** interacts with a rigorous single-log backend via a CLI or WebSockets, speaking in strict, bounded JSON specifications.

If there's ever a conflict between a human and the AI's architecture spec, the single-log backend enforces **Human Priority** via conditional appends. It will reject the AI's invalid spec and present a **Rich Ghosting Diff** to the human, who has the ultimate choice on how to proceed.

## Core Principles

- **Single-Log Multiplayer**: Built on **Restate** durable execution and **SQLite WAL mode**. All edits (human or AI) are routed through a single linearizable Write-Ahead Log. This eliminates complex distributed locking and guarantees absolute data integrity for concurrent users.
- **Performant Frontend**: Built in Dioxus Fullstack and Rust for speed and correctness. Uses pure WebSockets and MPSC pipelines to fan out real-time updates seamlessly.
- **Advanced Interaction**: Drag-and-drop icons, straight snapping arrows, multi-select subgraphs, and DAG-based auto-arrange.
- **Functional Rust**: Driven by strict engineering rigor—Data → Calculations → Actions. Zero panics, zero unwrap (`#![deny(clippy::unwrap_used)]`), and making illegal states unrepresentable through strongly typed domain boundaries.
- **Testing Rigor**: Inspired by Martin Fowler, Kent Beck, and David Farley. We leverage ATDD (Acceptance Test-Driven Development), property-based fuzzing (`proptest`), headless browser E2E tests, and an adversarial concurrent test harness. Deeply focused on code quality, testability, and clear Domain-Driven Design (DDD) boundaries.
