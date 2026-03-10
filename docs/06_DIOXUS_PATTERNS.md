# Dioxus 0.7 Frontend Patterns

Seshat uses **Dioxus 0.7**, pushing the DOM to its limits to handle 3,000+ nodes smoothly at 120 FPS. We actively avoid WebGL unless absolutely necessary.

## State Management
We heavily rely on `Signal` and `GlobalSignal` for reactive state, avoiding heavy clones when possible.

### Using Signals
Instead of old `use_state` hooks, use `use_signal`:

```rust
let mut hovered_node = use_signal(|| Option::<NodeId>::None);
let mut editing_node = use_signal(|| Option::<NodeId>::None);

// Reading
let current = hovered_node.read();

// Writing
hovered_node.set(Some(node_id));
```

### Context
Pass app-level state down via Context:
```rust
let mut doc_signal = use_context::<Signal<DiagramDocument>>();
let mut history_signal = use_context::<Signal<History>>();

// Mutating state via context
doc_signal.with_mut(|doc| {
    doc.editor_state.selected_items = new_selection;
});
```

## Performance & DOM Handling
- **Raw Event Listeners**: For intense drag/pan/zoom interactions, we sometimes mount vanilla JS listeners via `document::eval` to bypass framework overhead, dispatching refined JSON events back to Dioxus via channels/messages. (See `diagram_tool/src/ui/canvas.rs`).
- **Conditional Rendering**: Avoid unmounting/remounting large DOM subtrees frequently. Keep the DOM stable and update styles or positions.
- **WASM Constraints**: **CRITICAL** - Do not import `tokio`, `sqlx`, `reqwest` in UI components. Wrap backend boundaries in `#[cfg(not(target_arch = "wasm32"))]`.

## Event Handling
Use the `use_effect` hook coupled with document evals for raw global events, but standard `onclick`, `onmousedown` for simple localized component events.
