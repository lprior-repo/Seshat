# Seshat Feature Set & Responsibilities

Seshat is explicitly designed as a **Two-Way Sync** platform bridging human design and AI generation. To achieve this cleanly, the feature set is strictly divided into Frontend (Human) and Backend (AI/CLI) responsibilities.

## 1. Frontend Feature Set (The Human UI)
*Built with Dioxus 0.7, targeting Desktop and Web (WASM). Pushes the DOM to its limits for 120 FPS at 3000+ nodes.*

### Core Interactions
- **Tools**: Select (single, multi, marquee), Pan, Draw Edges, Create Subgraphs, Add Text.
- **Node Management**: Drag and drop, resize, delete, and rotate nodes. Support for custom icons, initials fallback, and provider coloring.
- **Subgraphs**: Native container support. Users can group nodes, drag nodes in/out of subgraphs (reparenting), and move entire nested structures simultaneously.
- **Edge Routing**: Nodes bind to edges. Dragging a node keeps arrows attached. Support for straight snapping, orthogonal routing, and DAG layouts.

### Workflow & Viewport
- **Infinite Canvas**: Smooth panning, zooming (in, out, reset to 100%), and fit-to-content capabilities. Minimap support for large architecture diagrams.
- **Grid & Snapping**: Snap-to-grid movement, node alignment, and distribution.
- **Clipboard & History**: Full Undo/Redo stack. Copy, Paste, Cut, Duplicate with automatic ID remapping and collision prevention.
- **Export**: Render out architecture diagrams to PNG, SVG, or raw JSON.

---

## 2. Backend Feature Set (The AI & CLI Interface)
*Built in strictly functional Rust using SQLite in WAL mode. Zero panics, making illegal states unrepresentable.*

### Headless AI Automation
- **CLI Commands (`seshat`)**: A fully headless agent interface. AI interacts with the diagram via terminal commands rather than mimicking UI clicks.
- **JSON Contracts**: AI reads (`seshat export --format json`) and modifies (`seshat apply patch.json`) the graph using strict JSON schemas defined in the codebase.
- **Data Validation Layer**: The backend automatically rejects AI patches that violate core invariants (e.g., introducing a cycle into a strictly DAG architecture, or placing a node out of bounds). 

### Storage & Synchronization
- **SQLite WAL Storage**: The absolute source of truth. All events and snapshots are robustly stored.
- **Two-Way Sync Engine**: If an AI agent executes `seshat apply patch.json` in the background, the UI immediately reflects the changes via state synchronizers. 
- **Conflict Resolution**: If the Human is dragging a node while the AI attempts to mutate it, the backend enforces conflict resolution rules (Human UI generally wins, AI is handed a rejection diff to recalculate).

---

## Summary
- **Frontend** is for *Intuition & Speed* (Humans drag shapes).
- **Backend** is for *Rigor & Scale* (AI writes JSON specs).
- Both utilize the same pure `Calc` layer to update the `Data` models.
