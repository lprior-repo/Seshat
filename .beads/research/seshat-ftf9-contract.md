# Architectural Contract: UI Phase 2 - Tailwind & JS Interop Boundaries
**Bead:** seshat-ftf9
**Domain:** Frontend UI Rendering & Async Timers
**Status:** DRAFTED BY ARCH-DESIGN-QA

## 1. EARS (Eliminate Requirements Ambiguity)
*   **Ubiquitous Constraint:** THE SYSTEM SHALL use statically analyzable Tailwind CSS classes for all static visual styling.
*   **State-Driven (Dynamic):** WHEN dynamic, structurally calculated values are required (e.g. `x: {mid_x - 50.0}` or calculated transforms), THE SYSTEM SHALL apply ONLY those calculated properties via inline styles.
*   **Event-Driven (Timers):** WHEN a time-delayed UI transition is needed (e.g. Toast dismissal), THE SYSTEM SHALL use Rust-native `gloo_timers::future::sleep` within a spawned Dioxus async block.
*   **Unwanted (Anti-Pattern):** THE SYSTEM SHALL NOT use `document::eval` with interpolated JavaScript strings to invoke standard Web API timers (e.g. `setTimeout`).
*   **Unwanted (Anti-Pattern):** THE SYSTEM SHALL NOT mix static CSS properties (like `border-radius`, `padding`) into inline `style` tags.

## 2. KIRK Contracts (Domain Modeling)
### Pure Boundaries & Interop Strictness
1.  **No `eval`-based JS Engine Execution for Timers:**
    *   **Precondition:** Any time-delayed action MUST be encapsulated in an `async move` block using `gloo_timers::future::sleep(Duration::from_millis(...)).await`.
    *   **Postcondition:** The `setTimeout` JavaScript primitive is never directly referenced in the UI logic.
    *   **Type Level Guarantee:** We rely on `std::future::Future` completion instead of asynchronous string evaluation, making the delay strongly typed and bound to the component's drop lifecycle.
2.  **Strict Styling Partition:**
    *   **Precondition:** A UI property is either *Static* (known at compile time) or *Dynamic* (calculated per frame/render).
    *   **Invariant:** All *Static* properties MUST live in `class="..."`. All *Dynamic* properties MUST live in `style="..."`.
    *   **Invariant:** Visual layout MUST remain pixel-perfect relative to the pre-migration state.

## 3. Inversion (Exhaustive Error Taxonomy & Failure Modes)
What are all the ways this migration can completely destroy the UI?

*   **Failure Mode A: The JIT Blindspot (Silent Visual Regression).**
    *   *Cause:* The Tailwind compiler does not parse Rust string interpolation (`class="bg-{theme_color}"`). If we attempt to dynamically construct Tailwind classes, they will not be bundled, resulting in unstyled elements.
    *   *Constraint:* We MUST NOT construct Tailwind class names dynamically. If theme variables like `TEXT_MAIN` or `BG_ELEVATED` are needed, they MUST be injected via CSS variables at the root level (`var(--bg-elevated)`), or matched explicitly.
*   **Failure Mode B: Future Cancellation & Ghost State.**
    *   *Cause:* We switch from `setTimeout` to `gloo_timers::future::sleep` inside a `spawn(async move { ... })`. If the Dioxus component unmounts before the sleep completes, the `Future` is dropped.
    *   *Implication:* This is actually *safer* than `setTimeout` (which would cause a use-after-free panic or ghost state update if not cleared via `clearTimeout`). We MUST guarantee that dropping the Future does not leave the system in an illegal intermediate state.
*   **Failure Mode C: Excessive Re-Spawning (The Memory Leak).**
    *   *Cause:* `use_effect` dependencies are unstable, causing the `gloo_timers` sleep task to be spawned on every render frame.
    *   *Constraint:* The dependencies for scheduling a dismiss MUST be strictly guarded by `HashSet` membership (as currently implemented with `pending_remove` and `pending_dismiss`) to prevent runaway task spawning.

## 4. Second-Order Consequence Tracing
*   **If we remove `document::eval`:** We lose the ability to easily trigger an ad-hoc JS script without crossing the `wasm-bindgen` boundary formally. However, we gain strict memory safety. The Dioxus component lifecycle will now naturally prune pending timeouts if the user navigates away, eliminating a whole class of race conditions.
*   **If we migrate `style` to Tailwind `class`:** We significantly shrink the VDOM size per render cycle, as we are no longer passing long, repetitive string values for inline styles. The `style` attribute string interpolation is notoriously slow in web frameworks; moving to static classes improves diffing performance during drag/zoom operations on the Canvas.
*   **If we rely on CSS Variables:** We must ensure `diagram_tool/src/ui/theme.rs` correctly maps its constants (like `BG_BASE`, `TEXT_MAIN`) to root-level CSS variables if we intend to reference them in Tailwind like `bg-[var(--bg-base)]`.

## 5. Pre-Mortem (The 3 AM Red Build)
**Scenario:** It is 2 days from now. The bead is merged. Users open the diagram tool and report that edge labels are invisible, and toasts never disappear.
**Why did it happen?**
1.  **Edges:** The developer blindly converted `style="fill:{TEXT_MUTED}; font-size:{font_size}px;"` to `class="text-muted text-[{font_size}px]"`. Tailwind does not support dynamic arbitrary values in JIT compilation. The `font_size` MUST remain in `style` because it scales with zoom, while the fill color CAN be moved to `class` if defined in the theme.
2.  **Toasts:** The developer used `gloo_timers::future::sleep` inside `spawn`, but didn't `await` it correctly, or accidentally spawned it outside the reactive context, causing the future to immediately drop or block the main thread.

## Execution Directives for the Developer
1.  **Analyze `edge_layer.rs`:** Identify *exactly* which properties are static. Leave `font_size` and exact coordinate properties in `style`. Move `stroke`, `fill`, and `padding` to `class`.
2.  **Analyze `toast/render.rs`:** Remove `document::eval`. Import `gloo_timers::future::sleep`. Use it in `spawn(async move { ... })`. Verify that the `spawn` happens EXACTLY ONCE per toast lifecycle.
3.  **CSS Variable Verification:** Ensure that theme colors (e.g. `TEXT_MAIN`) are accessible via Tailwind (e.g., `text-[var(--text-main)]`) if they are replaced.

**DO NOT PROCEED to implementation until you have mathematically verified that the Tailwind JIT compiler will catch the classes you introduce, and that the async Future lifecycle perfectly matches the Toast dismissal logic.**