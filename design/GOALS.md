# Repository Goals & Vision

This document outlines the high-level goals, philosophical North Star, and overarching objectives for the Seshat repository. It serves to align human contributors and AI agents on technical decisions, feature prioritization, architecture, and project milestones.

---

## 🎯 The Core Mission

Seshat bridges the gap between **human-created diagrams and AI-driven automation**. The core problem we are solving is that drawing diagrams (e.g., on Miro or Excalidraw) is fantastic for human brainstorming, but leaves the resulting architecture dead to programmatic AI agents. 

We are building the ultimate tool for designing software architecture alongside AI. Seshat combines the best of three worlds:
1. **D2 / Mermaid**: Declarative diagramming syntax, deterministic structure, and native subgraphs.
2. **Excalidraw**: AI-native diagramming, seamless multiplayer collaboration, and clean interaction models.
3. **Miro**: A highly polished human UI with drag-and-drop nodes, auto-snapping straight arrows, multi-select, subgraph generation, and auto-arrange capabilities powered by `petgraph` for DAG layouts.

---

## 🤝 The Two-Way Sync Principle

At the heart of Seshat is a **Real-Time Two-Way Sync** between humans and AI, operating on the exact same underlying architecture state:

1. **Human Interaction (Optimistic UI)**: Humans get an incredibly fast, intuitive frontend built on **Dioxus 0.7 Fullstack**. It leverages Optimistic UI patterns to guarantee an **8ms frame budget (120 FPS)** for interactions like dragging nodes, panning, and zooming. This performance target must be maintained even under heavy load with up to 12 concurrent human users modifying a 2000-node graph on a standard DOM layout (avoiding WebGL where possible).
2. **AI Interaction (CLI / Specs)**: AI interacts with the backend via a rigorous CLI or WebSockets, speaking in strict, bounded JSON specifications (`seshat validate patch.json`). It performs refactoring, scaffolding, and subgraph generation.

### Human Priority & Conflict Resolution
If there's ever a conflict between a human and the AI's architecture spec, the backend enforces a predictable resolution using **CRDT semantics** (LWW-Element-Set) tied to a Hybrid Logical Clock. The AI can recalculate its geometry around the human's new node positions and retry.

---

## 🏗️ Architectural Foundations

1. **The Event Sourcing Architecture**: Built on **SQLite with sqlx** in WAL mode. All edits (human or AI) are appended to a linearizable event log. Conflicts are automatically resolved using **LWW-Element-Set CRDTs** (Last-Writer-Wins) tied to a Hybrid Logical Clock. This eliminates complex distributed locking and guarantees absolute data integrity for concurrent users.
2. **Functional Rust Strictness**: Driven by rigorous engineering—**Data → Calculations → Actions**. Zero panics, zero unwrap (`#![deny(clippy::unwrap_used)]`), and making illegal states unrepresentable through strongly typed domain boundaries (enums over booleans).
3. **Persistent History**: Undo/redo capabilities are powered by **rpds (Persistent Data Structures)** to maintain full, immutable snapshots of the diagram document, guaranteeing "perfect inverse" state restoration without mutation bugs.

---

## 📈 Success Metrics

- **Performance**: Maintain 120 FPS (8ms frame times) for core interactions (dragging, panning) on 2000+ node documents in the browser.
- **AI Integration**: AI agents must be able to successfully parse, validate, and propose complex architectural changes via CLI patches 100% of the time without corrupting the graph state.
- **Robustness**: Maintain 100% adherence to zero-panic, zero-unwrap policies in the core logic. Moon CI pipeline (`moon run :ci-source`) must always pass locally and remote.

## 🔮 Future Vision

Over the next 6-12 months, Seshat will evolve from a diagramming tool into an active partner in software architecture. It will be capable of reading an architecture diagram and generating the requisite infrastructure, code skeletons, and compliance checks, serving as the single source of truth for repository structure.
