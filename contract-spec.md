# Contract Specification: Pixel-Faithful HTML/CSS -> Dioxus 0.7 Migration

## Context and Target
- Feature: migrate the full `professional-flow-builder` visual design and interaction behavior into the main Dioxus app with parity.
- Source of truth UI: `professional-flow-builder/components/flow/*.tsx` and `professional-flow-builder/app/globals.css`.
- Target (main Dioxus site): `src/main.rs` root `App` plus `src/ui/*.rs` components.
- Spec alignment required: `specs/flow-wasm-v1.yaml` behaviors and acceptance criteria remain valid after migration.
- Dioxus constraints (mandatory): `use_signal`/`use_memo`/`use_resource` only, no `use_state`; prop reactivity via `ReadOnlySignal<T>` or `ReadSignal<T>`; Tailwind classes in `class:`; handlers use `move |_|`; component tree stays modular and flat.

## Scope
- In scope: visual structure, class-level style parity, spacing/typography parity, responsive behavior, animation intent parity, interaction behavior parity, and state semantics needed to produce that parity.
- Out of scope: new features not present in source UI, backend/API redesign, changing workflow graph domain model semantics.

## Assumptions
- The source Next/Tailwind design is the canonical visual baseline.
- Parity target is desktop-first with no regressions on narrow/mobile widths.
- Tailwind utility vocabulary available to Dioxus can represent all required class tokens.

## Open Questions
- None blocking for contract authoring. If any source token is unavailable in the Dioxus Tailwind pipeline, fallback policy is defined in Error Taxonomy and postconditions.

## Domain Model for Migration (Type-Driven)

### Newtypes and constrained primitives
```rust
pub struct ClassList(pub String); // non-empty, trimmed, no duplicate spaces
pub struct Px(pub f32); // finite, non-negative where documented
pub struct ZoomFactor(pub f32); // invariant: 0.15 <= value <= 3.0
pub struct CssToken(pub String); // must exist in approved token set
pub struct ComponentId(pub &'static str); // stable identity for parity checks
pub struct TestSelector(pub &'static str); // stable selectors for verification
```

### Sum types that make illegal states unrepresentable
```rust
pub enum HandleType {
    Source,
    Target,
}

pub enum SelectionState {
    None,
    NodeSelected { node_id: NodeId },
}

pub enum CanvasInteraction {
    Idle,
    Panning { start: FlowPosition, origin: FlowPosition },
    DraggingNode { node_id: NodeId, start: FlowPosition, origin: FlowPosition },
    Connecting { from: NodeId, handle: HandleType, cursor: FlowPosition },
}

pub enum ParityLevel {
    Exact,
    EquivalentFallback { reason: String },
}
```

### Structured parity contract entities
```rust
pub struct StructuralContract {
    pub component_id: ComponentId,
    pub required_dom_order: Vec<TestSelector>,
    pub required_class_tokens: Vec<CssToken>,
}

pub struct VisualContract {
    pub component_id: ComponentId,
    pub width_px: Option<Px>,
    pub height_px: Option<Px>,
    pub spacing_scale: Vec<Px>,
    pub parity: ParityLevel,
}

pub struct InteractionContract {
    pub component_id: ComponentId,
    pub state_machine: Vec<CanvasInteractionTransition>,
}

pub struct CanvasInteractionTransition {
    pub from: CanvasInteraction,
    pub event: String,
    pub to: CanvasInteraction,
}
```

## Contract Signatures (Fallible Operations)
All migration and parity-verification operations are fallible and must return `Result<T, MigrationError>`.

```rust
pub fn build_source_contract() -> Result<UiParityContract, MigrationError>;
pub fn validate_source_assets(contract: &UiParityContract) -> Result<(), MigrationError>;
pub fn map_source_tokens_to_dioxus(contract: &UiParityContract) -> Result<TokenMap, MigrationError>;
pub fn validate_component_structure(rendered: &RenderedTree, contract: &UiParityContract) -> Result<(), MigrationError>;
pub fn validate_visual_metrics(snapshot: &VisualSnapshot, baseline: &VisualBaseline) -> Result<(), MigrationError>;
pub fn validate_interaction_machine(trace: &InteractionTrace) -> Result<(), MigrationError>;
pub fn validate_responsive_layout(report: &ResponsiveReport) -> Result<(), MigrationError>;
pub fn validate_animation_intent(report: &AnimationReport) -> Result<(), MigrationError>;
pub fn finalize_migration_report(results: &[ParityCheck]) -> Result<MigrationReport, MigrationError>;
```

## UI Structure Contract (Exact Sections)

### Root frame
- Must render one full-height container equivalent to source `div.flex.h-screen.w-full.flex-col.bg-background.overflow-hidden`.
- Must include top toolbar row, then a second row with sidebar + canvas + optional config panel.

### Toolbar contract
- Required regions in order: left name/stats, center zoom controls, right actions.
- Required behaviors:
  - workflow name editable text input.
  - node/edge counters visible and updated reactively.
  - zoom out, zoom in, fit view controls update zoom state.
  - execute button present and actionable.

### Node sidebar contract
- Fixed width equivalent to `w-[260px]` at desktop sizes.
- Header row, search row, scrollable grouped template list.
- Category groups in canonical order: trigger, action, logic, output.
- Each template item supports both click-add and drag payload add.

### Canvas contract
- Canvas area uses absolute layered composition:
  - grid background layer
  - transform layer with translated/scaled content
  - edges svg layer
  - nodes html layer
  - minimap overlay in bottom-right
- Transform style must be single source of truth: `translate(pan.x, pan.y) scale(zoom)`.

### Node card contract
- Each node card uses absolute positioning with fixed width `220px`.
- Must include top target handle, body, accent bar, bottom source handle.
- Must render icon + label + description + status indicator.
- Selected node visual emphasis (ring + border + shadow) must be preserved.

### Config panel contract
- Right-side panel appears only when node is selected.
- Slide-in animation intent from right is required.
- Contains header, scrollable content body, and footer actions.
- Required footer actions: duplicate + delete.

## Layout and Responsive Behavior Contract

## Preconditions
- Viewport dimensions are available and finite.
- Node dimensions are constant (`220x68`) for edge/fit/minimap geometry.
- Canvas pan/zoom state initialized before first interaction.

## Postconditions
- Desktop (`>= 1280px`): sidebar, canvas, and config panel coexist without overlap clipping.
- Tablet (`768px - 1279px`): no content loss; side regions may compress but remain operable.
- Mobile (`< 768px`): no hard overflow that blocks primary interactions; toolbar actions remain reachable.
- Minimap stays pinned to bottom-right of canvas viewport and does not block core drag/connection paths.

## Invariants
- `0.15 <= zoom <= 3.0` always holds.
- Transform math is consistent across node drag, edge render, and temporary edge preview.
- Grid background position tracks pan and grid size tracks zoom.
- Canvas interaction state machine is in exactly one state at a time.

## Animation Intent Contract
- Config panel entrance preserves right-to-left slide intent within 150-300ms duration.
- Node handle hover preserves scale-up affordance.
- Selected/hover transitions preserve soft transition timing (100-200ms class transition intent).
- Any token substitutions must keep semantic intent ("primary emphasis", "muted border", "destructive action").

## Interaction Behavior Contract

## Preconditions
- Pointer events are delivered to the canvas root and node/handle sub-elements.
- Node IDs and connection endpoints are valid prior to mutation.

## Postconditions
- Canvas background mouse-down clears selection and starts pan mode.
- Node mouse-down starts drag mode for that node only.
- Handle mouse-down starts connecting mode with temporary edge preview.
- Mouse-up finalizes valid connection, rejects self/duplicate connection, then clears transient connect state.
- Wheel modifies zoom toward pointer position and clamps zoom range.

## Interaction state transition invariants
- `Idle -> Panning` only via background mouse down.
- `Idle -> DraggingNode` only via node mouse down.
- `Idle -> Connecting` only via handle mouse down.
- `Connecting -> Idle` must clear both `connecting_from` and `temp_edge`.
- `DraggingNode -> Idle` and `Panning -> Idle` must terminate on mouse up/leave.

## Error Taxonomy (Exhaustive)
```rust
#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("required source file missing: {path}")]
    SourceFileMissing { path: String },

    #[error("source parse failed for {path}: {reason}")]
    SourceParseFailed { path: String, reason: String },

    #[error("required component missing in source contract: {component}")]
    SourceComponentMissing { component: String },

    #[error("required class token missing: component={component}, token={token}")]
    RequiredClassMissing { component: String, token: String },

    #[error("unsupported css token for dioxus pipeline: {token}")]
    UnsupportedCssToken { token: String },

    #[error("token mapping collision: source={source}, target={target}")]
    TokenMappingCollision { source: String, target: String },

    #[error("dom structure mismatch in {component}: expected={expected}, actual={actual}")]
    DomStructureMismatch { component: String, expected: String, actual: String },

    #[error("layout metric out of tolerance: component={component}, metric={metric}, expected={expected}, actual={actual}")]
    LayoutToleranceExceeded { component: String, metric: String, expected: String, actual: String },

    #[error("responsive regression at breakpoint {breakpoint}: {reason}")]
    ResponsiveRegression { breakpoint: String, reason: String },

    #[error("animation intent regression: animation={animation}, reason={reason}")]
    AnimationIntentRegression { animation: String, reason: String },

    #[error("invalid interaction transition: from={from}, event={event}, to={to}")]
    InvalidInteractionTransition { from: String, event: String, to: String },

    #[error("invalid connection attempt: {reason}")]
    InvalidConnectionAttempt { reason: String },

    #[error("node not found for interaction: {node_id}")]
    NodeNotFound { node_id: String },

    #[error("edge render target missing: source={source}, target={target}")]
    EdgeEndpointMissing { source: String, target: String },

    #[error("viewport invariant violated: {reason}")]
    ViewportInvariantViolation { reason: String },

    #[error("local storage read failure: {reason}")]
    LocalStorageReadFailure { reason: String },

    #[error("local storage write failure: {reason}")]
    LocalStorageWriteFailure { reason: String },

    #[error("local storage data corrupted: {reason}")]
    LocalStorageDataCorrupted { reason: String },

    #[error("minimap render regression: {reason}")]
    MinimapRegression { reason: String },

    #[error("parity verification failed: {reason}")]
    ParityVerificationFailed { reason: String },
}
```

## Failure Analysis Matrix (Function -> Failure Modes)

### build_source_contract
- Invalid input: missing any canonical source file -> `SourceFileMissing`.
- Invariant break: cannot infer required component tree -> `SourceComponentMissing`.
- Edge cases: optional decorative classes absent -> allow `ParityLevel::EquivalentFallback`, not hard failure.

### map_source_tokens_to_dioxus
- Invalid input: unknown token namespace -> `UnsupportedCssToken`.
- Invariant break: two source tokens mapped to same semantic slot incorrectly -> `TokenMappingCollision`.
- Edge case: source has opacity shorthand unsupported in target pipeline -> `UnsupportedCssToken`.

### validate_component_structure
- Invalid input: rendered tree missing required selector -> `DomStructureMismatch`.
- Invariant break: wrong order of sections (toolbar/canvas/panel) -> `DomStructureMismatch`.
- Edge case: optional config panel absent while no selection -> allowed (no error).

### validate_visual_metrics
- Invalid input: missing baseline snapshot -> `ParityVerificationFailed`.
- Invariant break: width/spacing/radius out of tolerance -> `LayoutToleranceExceeded`.
- Edge case: sub-pixel anti-alias differences -> tolerated under configured threshold.

### validate_interaction_machine
- Invalid input: event trace references unknown node -> `NodeNotFound`.
- Invariant break: illegal state jump (e.g., `Idle -> Connecting` without handle event) -> `InvalidInteractionTransition`.
- Edge case: mouse leave while dragging -> must normalize to `Idle` without panic.

### validate_responsive_layout
- Invalid input: missing viewport report entry -> `ParityVerificationFailed`.
- Invariant break: controls not reachable at breakpoint -> `ResponsiveRegression`.
- Edge case: minimap hidden by policy on narrow screens -> allowed only if documented in contract variant.

### validate_animation_intent
- Invalid input: animation metadata unavailable -> `AnimationIntentRegression`.
- Invariant break: panel appears without directional motion -> `AnimationIntentRegression`.
- Edge case: reduced-motion preference active -> allowed equivalent behavior with preserved state visibility.

## Definition of Done
- Structural parity achieved for toolbar, sidebar, canvas, node card, edges, minimap, config panel.
- Style parity achieved for key classes/tokens and semantic color hierarchy.
- Interaction parity achieved for pan, zoom, drag, connect, select, duplicate, delete, and panel toggle.
- Responsive and animation contracts validated and passing.
- Every `MigrationError` variant has at least one corresponding test case.

## Non-goals
- Changing workflow execution semantics.
- Rebranding visual identity beyond source baseline.
- Adding new widgets not present in source UI.
