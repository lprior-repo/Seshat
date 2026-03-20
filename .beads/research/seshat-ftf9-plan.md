# Phase 2 Tailwind and JS Interop Plan (seshat-ftf9)

## 1. Inline Styles to Tailwind CSS Mapping

The following files contain inline `style: "..."` attributes. Most of these should be migrated to Tailwind utility classes (`class: "..."`). Dynamic styles (e.g. `top: {y}px`, `background: {color}`) will retain a minimal `style` attribute alongside Tailwind classes.

### UI Components (`diagram_tool/src/ui/`)
- `toolbar.rs`: Replace `display: flex; align-items: center;` etc., with `flex items-center ...`.
- `properties.rs`: Convert panel layouts to `flex flex-col gap-2 p-2 ...`.
- `sidebar/mod.rs`, `sidebar/icon_tile.rs`: Replace raw grid/flex/padding/border styles with equivalent Tailwind classes (e.g., `grid grid-cols-4 gap-1`).
- `toast/render.rs`: Toast notification styling (fixed positioning, animations, shadow) maps to `fixed right-4 top-16 z-50 flex flex-col gap-2 ...`
- `theme_provider.rs`: Theme CSS variable definitions (might need to coordinate with `tailwind.config.js`).

### Sidebar Primitives (`diagram_tool/src/ui/sidebar_primitives/`)
- `core.rs`, `group.rs`, `menu.rs`, `provider.rs`, `trigger.rs`: These primitives use heavily parameterized styles. The base styling should move to Tailwind (e.g. `display: flex`, margins, borders), leaving only true variable overrides.

### Canvas & Node Layer (`diagram_tool/src/ui/canvas/`)
- `root_container/mod.rs`: `flex: 1; position: relative; overflow: hidden;` -> `flex-1 relative overflow-hidden`.
- `canvas_view/overlays.rs`, `canvas_view/selection_handles.rs`: Absolute positioning needs to keep `style="left: {x}px; top: {y}px"` but standard styling (borders, z-index, background) goes to Tailwind (`absolute border-blue-500 z-10 ...`).
- `node_layer/node_element.rs`, `node_layer/node_content.rs`, `node_layer/inline_edit.rs`: Move flexbox, border-radius, background, shadows to Tailwind.
- `node_layer/connection_dots.rs`: `position: absolute; border-radius: 999px;` -> `absolute rounded-full`.
- `edge_layer.rs`: SVG styling can use Tailwind classes for strokes and fills.
- `toolbar.rs`: Canvas-specific floating toolbar styling maps to `absolute z-20 border rounded-md shadow-lg`.

## 2. JS Interop (`document::eval`) to Rust/WASM Equivalents

The codebase currently uses `dioxus::prelude::document::eval` to execute raw JavaScript strings. These should be migrated to proper Rust Web APIs using `web-sys` / `wasm-bindgen` or structured JS interop to improve performance, type safety, and security.

### Core Architecture / App-Level
- `app/autosave_hooks.rs`: Interacting with local storage or indexedDB.
  - **Equivalent:** Use `web_sys::window().unwrap().local_storage()` directly in Rust.
- `app/validation.rs`: Could be using JS for DOM measurement or specific validation logic.
  - **Equivalent:** Perform logic in Rust, or use `web_sys` equivalents for DOM measurements.
- `ui/theme_provider.rs`: Setting document theme/classes or reading preferences.
  - **Equivalent:** `web_sys::window().unwrap().document().unwrap().document_element().unwrap().class_list()`.

### Local Storage & Persistence
- `ui/sidebar_persistence.rs`
- `ui/toolbar/persistence/open.rs`
- `ui/sidebar_primitives/provider.rs`
  - **Equivalent:** Replace JS string execution with `web_sys::Storage` bindings to read/write state locally.

### Canvas / DOM Measurements / Handlers
- `ui/canvas/root_handlers/resize.rs`
- `ui/canvas/root_handlers/raf.rs`
- `ui/canvas/root_handlers/middle_pan.rs`
- `ui/canvas/root_handlers/touch.rs`
- `ui/canvas/root_handlers/keyboard.rs`
- `hooks/keyboard.rs`
  - **Equivalent:** Use `gloo-events` or `web_sys::EventTarget::add_event_listener_with_callback` to register listeners properly in Rust instead of evaluating JS strings. Request Animation Frame (RAF) should use `web_sys::window().unwrap().request_animation_frame()`.

### Utilities & Testing
- `ui/mobile.rs`: Feature detection or mobile-specific DOM hacks.
  - **Equivalent:** Check `web_sys::window().unwrap().navigator().user_agent()` or screen dimensions natively.
- `ui/toast/render.rs`: Possible animation or DOM cleanup JS.
  - **Equivalent:** Handle via Dioxus state and CSS transitions (Tailwind).
- `ui/toolbar/export_actions.rs`: Dealing with canvas rendering or blob downloads (`URL.createObjectURL`, `a.click()`).
  - **Equivalent:** Use `web_sys` for `HtmlAnchorElement`, create `Blob` via `web_sys::Blob::new_with_u8_array_sequence`, and trigger `.click()` programmatically via `HtmlElement`.
- `hooks/e2e_reset.rs`: E2E test hooks resetting global state.
  - **Equivalent:** Expose a `#[wasm_bindgen]` Rust function that E2E tests can call directly, bypassing `eval`.