# Dioxus 0.7 Frontend Guide

Seshat is built on **Dioxus 0.7**, embracing its core philosophies of fine-grained reactivity, `Copy` state via Signals, and a unified architecture that handles both Desktop and Web from a single codebase.

This document details the exact constraints and patterns required when touching UI code.

---

## ⚡ Reactivity & State Management

Dioxus 0.7 entirely deprecates `use_state`. All reactivity is driven by **Signals**. Signals are `Copy + Send + Sync`, meaning you do not need to clone them to pass them into closures or async blocks.

### 1. `use_signal` for Atomic State
Use `use_signal` for simple, individual values (e.g., UI toggle states, hover states).

```rust
// Defining a signal
let mut hovered_node = use_signal(|| Option::<NodeId>::None);

// Reading (auto-subscribes the component to updates)
let current = hovered_node(); // or hovered_node.read()

// Writing (triggers re-render)
hovered_node.set(Some(node_id));
```

### 2. `use_store` for Collections
For nested structures, vectors, or hash maps, do not wrap a giant struct in a single signal if you only need fine-grained updates. Instead, use `use_store` with `#[derive(Store)]` (if applicable), or rely on our immutable data structures (`rpds` / `im`).

### 3. Passing Props with `ReadSignal<T>`
Always use `ReadSignal<T>` when a child component needs to react to a changing value from its parent. This accepts `Signal`, `Memo`, or plain primitives with auto-conversion.

```rust
#[derive(PartialEq, Props, Clone)]
pub struct NodeProps {
    pub node_id: NodeId,
    pub position: ReadSignal<Position>, // Child reacts when position changes
}
```

---

## 🏎️ Performance Optimizations (120 FPS Target)

Seshat is required to maintain an **8ms frame budget** for rendering up to 3,000 concurrent nodes.

1. **Avoid VDOM Thrashing:** Dioxus components are memoized by default and only re-render if their `Props` change (evaluated via `PartialEq`). Be extremely careful not to pass structs that change every frame unless absolutely necessary.
2. **Raw Event Bypasses:** For intense `mousemove`, `pan`, and `zoom` events, we bypass the standard Dioxus event listener system to prevent crushing the VDOM with thousands of events per second. We use `document::eval` to mount vanilla JS listeners that dispatch throttled, refined JSON events back to the Rust UI via channels. (See `diagram_tool/src/ui/canvas.rs`).
3. **Inline Transforms:** Do not update React-like "state" just to move a node by 1 pixel if it triggers a cascade. Prefer updating inline CSS styles (`transform: translate(x,y)`) bound directly to signals.

---

## 🛑 WASM & Platform Constraints (ADR-006)

Our Dioxus application ships to both Desktop (`dx run`) and Web (`dx serve`). 

Because the web target compiles to `wasm32-unknown-unknown`, **you must obey strict compile-time boundaries**:

1. **No Tokio/I/O in the UI:** 
   NEVER include `tokio`, `mio`, `sqlx`, or `reqwest` (with default TLS features) in the `wasm32-unknown-unknown` UI codebase. 
2. **Conditional Compilation:**
   Isolate server, database, and filesystem operations using `#[cfg(not(target_arch = "wasm32"))]`.
   ```rust
   #[cfg(not(target_arch = "wasm32"))]
   pub mod store_sqlx;
   ```
3. **No Fullstack Flag on Web:**
   The `fullstack` feature MUST NOT be active when building purely for the Web frontend. Use `default = ["web"]` in `Cargo.toml`.

---

## 🎨 Styling

We use **Tailwind CSS** natively integrated via the Dioxus CLI. 
- You do not need to run an external Tailwind watcher.
- Inject the stylesheet at the root of your application:
  ```rust
  rsx! {
      document::Stylesheet { href: asset!("/assets/tailwind.css") }
      // ...
  }
  ```
