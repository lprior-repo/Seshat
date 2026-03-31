mod handlers;

use crate::ui::canvas::canvas_view::{
    edge_preview_overlay, rubber_band_overlay, subgraph_preview_overlay,
};
use crate::ui::canvas::edge_layer::EdgeLayer;
use crate::ui::canvas::grid_layer::GridLayer;
use crate::ui::canvas::node_layer::NodeLayer;
use crate::ui::canvas::state::CanvasState;
use crate::ui::canvas::toolbar::{SelectionPill, Toolbar};
use crate::ui::theme::{
    ThemeMode, ACCENT_DASH_BORDER, BG_BASE, BG_ELEVATED, EDGE_DEFAULT, EDGE_SELECTED,
};
use canvas_domain::interaction_reducer::{InteractionMode, ResizeHandle};
use canvas_domain::perf::to_screen_coords;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use handlers::{
    handle_double_click, handle_mouse_down, handle_mouse_move, handle_mouse_up, handle_wheel,
};

#[component]
pub fn RootContainer(state: CanvasState) -> Element {
    let mut drag_over = state.drag_over;

    let bg_color = BG_BASE;
    let border_style = if *drag_over.read() {
        ACCENT_DASH_BORDER
    } else {
        "none"
    };
    let cursor_style = if *state.space_pressed.read() {
        if *state.space_pan_active.read() {
            "grabbing"
        } else {
            "grab"
        }
    } else {
        match *state.interaction_mode.read() {
            InteractionMode::Panning { .. } => "grabbing",
            InteractionMode::DrawingEdge { .. } => "crosshair",
            InteractionMode::RubberBand { .. } => "crosshair",
            InteractionMode::ResizingSelection { handle, .. } => match handle {
                ResizeHandle::Nw | ResizeHandle::Se => "nwse-resize",
                ResizeHandle::Ne | ResizeHandle::Sw => "nesw-resize",
                ResizeHandle::N | ResizeHandle::S => "ns-resize",
                ResizeHandle::E | ResizeHandle::W => "ew-resize",
            },
            InteractionMode::DraggingSelection { .. } => "move",
            _ => "default",
        }
    };

    let state_drop = state.clone();
    let handle_drop = move |evt: Event<dioxus::prelude::DragData>| {
        crate::ui::canvas::root_handlers::handle_drop(
            evt,
            state_drop.drag_over,
            state_drop.dragging_icon,
            state_drop.doc_signal,
            state_drop.history_signal,
            state_drop.canvas_origin,
        );
    };

    let state_dbclick = state.clone();
    let state_wheel = state.clone();
    let state_mousedown = state.clone();
    let state_mousemove = state.clone();
    let state_mouseup = state.clone();

    rsx! {
        div {
            class: "canvas-container flex-1 relative overflow-hidden overscroll-none touch-none select-none box-border",
            "data-testid": "canvas-root",
            style: "background: radial-gradient(circle at 24% 12%, {BG_ELEVATED} 0%, {bg_color} 66%); cursor: {cursor_style}; border: {border_style};",

            ondragover: move |evt| { evt.prevent_default(); },
            ondragenter: move |evt| { evt.prevent_default(); drag_over.set(true); },
            ondragleave: move |_| { drag_over.set(false); },
            ondrop: handle_drop,
            oncontextmenu: move |evt| evt.prevent_default(),
            onauxclick: move |evt| {
                if evt.data.trigger_button() == Some(MouseButton::Auxiliary) {
                    evt.prevent_default();
                }
            },
            ondoubleclick: move |evt| handle_double_click(state_dbclick.clone(), evt),
            onwheel: move |evt| handle_wheel(state_wheel.clone(), evt),
            onmousedown: move |evt| handle_mouse_down(state_mousedown.clone(), evt),
            onmousemove: move |evt| handle_mouse_move(state_mousemove.clone(), evt),
            onmouseup: move |evt| handle_mouse_up(state_mouseup.clone(), evt),
            onmouseleave: move |_| {},

            div {
                "data-testid": "canvas-hit-layer",
                class: "absolute inset-0 pointer-events-none opacity-0"
            }

            svg {
                class: "absolute top-0 left-0 w-full h-full pointer-events-none z-0",
                defs {
                    marker {
                        id: "arrowhead",
                        marker_width: "10",
                        marker_height: "7",
                        ref_x: "9",
                        ref_y: "3.5",
                        orient: "auto",
                        polygon { points: "0 0, 10 3.5, 0 7", fill: "{EDGE_DEFAULT}" }
                    }
                    marker {
                        id: "arrowhead-selected",
                        marker_width: "10",
                        marker_height: "7",
                        ref_x: "9",
                        ref_y: "3.5",
                        orient: "auto",
                        polygon { points: "0 0, 10 3.5, 0 7", fill: "{EDGE_SELECTED}" }
                    }
                    marker {
                        id: "arrow-pending",
                        marker_width: "10",
                        marker_height: "7",
                        ref_x: "9",
                        ref_y: "3.5",
                        orient: "auto",
                        polygon { points: "0 0, 10 3.5, 0 7", fill: "{EDGE_SELECTED}", opacity: "0.5" }
                    }
                    // Reverse arrow for bidirectional edges (points left)
                    marker {
                        id: "arrowhead-start",
                        marker_width: "10",
                        marker_height: "7",
                        ref_x: "1",
                        ref_y: "3.5",
                        orient: "auto",
                        polygon { points: "10 0, 0 3.5, 10 7", fill: "{EDGE_DEFAULT}" }
                    }
                    marker {
                        id: "arrowhead-start-selected",
                        marker_width: "10",
                        marker_height: "7",
                        ref_x: "1",
                        ref_y: "3.5",
                        orient: "auto",
                        polygon { points: "10 0, 0 3.5, 10 7", fill: "{EDGE_SELECTED}" }
                    }
                }

                GridLayer { doc_signal: state.doc_signal, viewport_size: state.viewport_size }

                EdgeLayer {
                    doc_signal: state.doc_signal,
                    history_signal: state.history_signal,
                    editor_state: state.editor_state,
                    edit_value: state.edit_value,
                    viewport_size: state.viewport_size,
                    interaction_mode: state.interaction_mode,
                    canvas_origin: state.canvas_origin,
                    node_viewport_trigger: state.node_viewport_trigger,
                    db_tx: state.db_tx
                }

                {
                    let mode_now = state.interaction_mode.read().clone();
                    let doc_now = state.doc_signal.read();
                    let edge_overlay = edge_preview_overlay(&mode_now, &doc_now, to_screen_coords);
                    let band_overlay = rubber_band_overlay(&mode_now, &doc_now, to_screen_coords);
                    let subgraph_overlay = subgraph_preview_overlay(&mode_now, &doc_now, to_screen_coords);
                    rsx! {
                        {edge_overlay}
                        {band_overlay}
                        {subgraph_overlay}
                    }
                }
            }

            NodeLayer {
                doc_signal: state.doc_signal,
                history_signal: state.history_signal,
                tool_signal: state.tool_signal,
                interaction_mode: state.interaction_mode,
                editor_state: state.editor_state,
                edit_value: state.edit_value,
                viewport_size: state.viewport_size,
                ordered_node_cache: state.ordered_node_cache,
                node_viewport_trigger: state.node_viewport_trigger,
                canvas_origin: state.canvas_origin,
                shift_pressed: state.shift_pressed,
                ctrl_pressed: state.ctrl_pressed,
                meta_pressed: state.meta_pressed,
                space_pressed: state.space_pressed,
                multi_touch_active: state.multi_touch_active,
                space_pan_active: state.space_pan_active,
                pending_pointer_sample: state.pending_pointer_sample,
                db_tx: state.db_tx
            }

            Toolbar {
                doc_signal: state.doc_signal,
                history_signal: state.history_signal,
                interaction_mode: state.interaction_mode
            }
            SelectionPill { doc_signal: state.doc_signal }
            ThemeToggle {}
        }
    }
}

/// Visible theme toggle button positioned at the bottom-right of the canvas.
/// Cycles through `System` → `Light` → `Dark` → `White` → `System` on click.
/// Reads/writes the `theme_mode` `Signal` provided by `ThemeProvider`.
#[component]
fn ThemeToggle() -> Element {
    let mut theme_mode = use_context::<Signal<ThemeMode>>();
    let current = *theme_mode.read();
    let label = current.label();

    rsx! {
        button {
            class: "absolute right-[12px] bottom-[12px] z-[20] rounded-[8px] text-[11px] px-[9px] py-[5px] backdrop-blur-sm border border-border bg-[var(--toolbar-bg)]/90 text-muted-foreground shadow-lg cursor-pointer hover:opacity-80 transition-opacity",
            "data-testid": "theme-toggle-btn",
            onclick: move |_| {
                let next = theme_mode.read().next();
                theme_mode.set(next);
            },
            "{label}"
        }
    }
}
