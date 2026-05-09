#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

pub mod auto_save;
mod mobile_menu;
pub mod persistence;
mod persistence_compat;

// Re-export persistence functions for use by other modules (e.g., keyboard shortcuts)
pub use persistence::{open_workspace, save_workspace, WorkspaceSignals};

#[cfg(test)]
mod tests;

use crate::app::AppState;
use crate::mutation::error::MutationError;
use crate::ui::commands::{
    apply_delete_selected, apply_redo, apply_toggle_edge_direction, apply_undo, apply_zoom_in,
    apply_zoom_out, apply_zoom_reset,
};
use crate::ui::editor::ToolMode;
use crate::ui::icons::{Icon, IconKind};
use crate::ui::theme::{ThemeMode, ACCENT};
use dioxus::prelude::*;
use mobile_menu::MobileToolbarMenu;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct ToolbarStats {
    pub selected_count: usize,
    pub node_count: usize,
    pub edge_count: usize,
    pub revision: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonState {
    #[default]
    Default,
    Active,
    Disabled,
}

#[derive(Clone, PartialEq, Default)]
pub enum ButtonVariant {
    #[default]
    Standard,
    Tool {
        title: &'static str,
    },
    Destructive {
        title: &'static str,
    },
    GridToggle {
        title: &'static str,
    },
}

impl ButtonVariant {
    fn title(&self) -> &'static str {
        match self {
            Self::Standard => "",
            Self::Tool { title } => title,
            Self::Destructive { title } => title,
            Self::GridToggle { title } => title,
        }
    }

    fn fill_color(&self) -> Option<&'static str> {
        match self {
            Self::Destructive { .. } => Some("#ef4444"),
            _ => None,
        }
    }

    fn active_bg_class(&self) -> &'static str {
        match self {
            Self::GridToggle { .. } => "bg-[oklch(0.4_0.1_160)]",
            _ => "bg-[var(--accent-soft)]",
        }
    }
}

#[component]
fn Divider() -> Element {
    rsx! {
        div {
            class: "w-[1px] h-6 bg-[var(--border-subtle)] mx-1"
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct IconButtonProps {
    pub test_id: &'static str,
    pub state: ButtonState,
    pub onclick: EventHandler<MouseEvent>,
    pub icon: IconKind,
    #[props(default)]
    pub variant: ButtonVariant,
}

#[component]
fn IconButton(props: IconButtonProps) -> Element {
    let bg = match props.state {
        ButtonState::Active => props.variant.active_bg_class(),
        _ => "bg-transparent hover:bg-[var(--bg-elevated)]",
    };
    let border = match props.state {
        ButtonState::Active => "border-[var(--accent)]",
        _ => "border-transparent",
    };
    let fill = match props.state {
        ButtonState::Active => Some(ACCENT),
        _ => props.variant.fill_color(),
    };
    let opacity = match props.state {
        ButtonState::Disabled => "opacity-40",
        _ => "opacity-100",
    };
    let cursor = match props.state {
        ButtonState::Disabled => "cursor-not-allowed",
        _ => "cursor-pointer",
    };
    let tooltip = props.variant.title();

    rsx! {
        button {
            "data-testid": props.test_id,
            class: "w-9 h-9 flex items-center justify-center rounded-md border {border} {bg} {cursor} {opacity} p-0 outline-none mx-0.5 text-foreground transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-[var(--accent)] focus-visible:outline-offset-2",
            title: "{tooltip}",
            "aria-label": "{tooltip}",
            onclick: move |evt| {
                if props.state != ButtonState::Disabled {
                    props.onclick.call(evt);
                }
            },
            Icon { kind: props.icon, color: fill, size: 20 }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct TextButtonProps {
    pub test_id: &'static str,
    pub state: ButtonState,
    pub onclick: EventHandler<MouseEvent>,
    pub text: &'static str,
    pub title: &'static str,
}

#[component]
fn TextButton(props: TextButtonProps) -> Element {
    let bg = match props.state {
        ButtonState::Active => "bg-[var(--accent-soft)]",
        _ => "bg-transparent hover:bg-[var(--bg-elevated)]",
    };
    let border = match props.state {
        ButtonState::Active => "border-[var(--accent)]",
        _ => "border-transparent",
    };
    let fill = match props.state {
        ButtonState::Active => "text-[var(--accent)]",
        _ => "text-foreground",
    };
    let opacity = match props.state {
        ButtonState::Disabled => "opacity-40",
        _ => "opacity-100",
    };
    let cursor = match props.state {
        ButtonState::Disabled => "cursor-not-allowed",
        _ => "cursor-pointer",
    };

    rsx! {
        button {
            "data-testid": props.test_id,
            class: "w-9 h-9 flex items-center justify-center rounded-md border {border} {bg} {cursor} {opacity} p-0 outline-none mx-0.5 {fill} text-[18px] font-medium font-mono transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-[var(--accent)] focus-visible:outline-offset-2",
            title: "{props.title}",
            "aria-label": "{props.title}",
            onclick: move |evt| {
                if props.state != ButtonState::Disabled {
                    props.onclick.call(evt);
                }
            },
            "{props.text}"
        }
    }
}

#[component]
fn LabelButton(
    test_id: &'static str,
    icon: Option<IconKind>,
    text: &'static str,
    onclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! {
        button {
            "data-testid": test_id,
            class: "h-9 px-2 flex items-center justify-center rounded-md border border-transparent bg-transparent hover:bg-[var(--bg-elevated)] cursor-pointer p-0 outline-none mx-0.5 text-foreground text-[13px] transition-colors gap-1.5 focus-visible:outline focus-visible:outline-2 focus-visible:outline-[var(--accent)] focus-visible:outline-offset-2",
            title: "{text}",
            "aria-label": "{text}",
            onclick: move |evt| {
                if let Some(handler) = &onclick {
                    handler.call(evt);
                }
            },
            if let Some(i) = icon {
                Icon { kind: i, size: 16 }
            }
            span { "{text}" }
        }
    }
}

#[component]
fn ToolbarThemeToggle() -> Element {
    let mut theme_mode = use_context::<Signal<ThemeMode>>();
    let current = *theme_mode.read();
    let label = current.label();
    let aria_label = format!("Theme: {label}. Change theme");

    rsx! {
        button {
            class: "h-9 px-2.5 flex items-center justify-center rounded-md border border-border bg-[var(--toolbar-bg)] text-muted-foreground hover:text-foreground hover:bg-[var(--bg-elevated)] cursor-pointer text-[12px] transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-[var(--accent)] focus-visible:outline-offset-2",
            "data-testid": "theme-toggle-btn",
            title: "{aria_label}",
            "aria-label": "{aria_label}",
            onclick: move |_| {
                let next = theme_mode.read().next();
                theme_mode.set(next);
            },
            "{label}"
        }
    }
}

#[component]
pub fn Toolbar() -> Element {
    let app_state = use_context::<AppState>();
    let mut doc_signal = app_state.document;
    let session_signal = app_state.session;
    let history_signal = app_state.history;
    let mut tool_signal = app_state.tool_mode;
    let mut arrow_type_signal = app_state.arrow_type;
    let toasts = app_state.toasts;

    let toolbar_stats = app_state.toolbar_stats;
    let stats = *toolbar_stats.read();
    let viewport_size_signal = app_state.viewport_size;

    let undo_disabled = !history_signal.read().can_undo();
    let redo_disabled = !history_signal.read().can_redo();
    let doc = doc_signal.read();
    let zoom_percent = (doc.editor_state.zoom.0 * 100.0).round();
    let show_grid = doc.editor_state.show_grid;
    let is_dirty = session_signal.read().is_dirty();
    drop(doc);

    // Update window title with dirty indicator
    #[cfg(target_arch = "wasm32")]
    {
        use_effect(move || {
            let dirty = session_signal.read().is_dirty();
            let title = if dirty { "Seshat *" } else { "Seshat" };
            let _eval = document::eval(&format!(
                r#"(function() {{ window.document.title = "{title}"; }})()"#
            ));
        });
    }

    rsx! {
        div {
            "data-testid": "toolbar-root",
            class: "relative h-14 shrink-0 bg-surface flex items-center justify-between gap-2 px-2 sm:px-4 border-b border-border-subtle w-full overflow-hidden",

            // Left section: Tools and actions
            div {
                class: "flex items-center min-w-0",

                // Tools
                IconButton {
                    test_id: "tool-select",
                    state: if *tool_signal.read() == ToolMode::Select { ButtonState::Active } else { ButtonState::Default },
                    onclick: move |_| tool_signal.set(ToolMode::Select),
                    icon: IconKind::Select,
                    variant: ButtonVariant::Tool { title: "Select (V)" }
                }
                IconButton {
                    test_id: "tool-pan",
                    state: if *tool_signal.read() == ToolMode::Pan { ButtonState::Active } else { ButtonState::Default },
                    onclick: move |_| tool_signal.set(ToolMode::Pan),
                    icon: IconKind::Pan,
                    variant: ButtonVariant::Tool { title: "Pan (H)" }
                }
                IconButton {
                    test_id: "tool-edge",
                    state: if *tool_signal.read() == ToolMode::Edge { ButtonState::Active } else { ButtonState::Default },
                    onclick: move |_| tool_signal.set(ToolMode::Edge),
                    icon: IconKind::Edge,
                    variant: ButtonVariant::Tool { title: "Edge (L)" }
                }
                TextButton {
                    test_id: "tool-text",
                    state: if *tool_signal.read() == ToolMode::Text { ButtonState::Active } else { ButtonState::Default },
                    onclick: move |_| tool_signal.set(ToolMode::Text),
                    text: "T",
                    title: "Text (T)"
                }
                div { class: "hidden sm:flex items-center",
                    IconButton {
                        test_id: "tool-subgraph",
                        state: if *tool_signal.read() == ToolMode::Subgraph { ButtonState::Active } else { ButtonState::Default },
                        onclick: move |_| tool_signal.set(ToolMode::Subgraph),
                        icon: IconKind::Subgraph,
                        variant: ButtonVariant::Tool { title: "Subgraph (R)" }
                    }
                    IconButton {
                        test_id: "tool-grid",
                        state: if show_grid { ButtonState::Active } else { ButtonState::Default },
                        onclick: move |_| {
                            let mut d = doc_signal.write();
                            d.editor_state.show_grid = !d.editor_state.show_grid;
                        },
                        icon: IconKind::Grid,
                        variant: ButtonVariant::GridToggle { title: "Toggle Grid" }
                    }
                }

                div { class: "hidden sm:flex items-center",
                    Divider {}

                    // History
                    IconButton {
                        test_id: "toolbar-undo",
                        state: if undo_disabled { ButtonState::Disabled } else { ButtonState::Default },
                        onclick: move |_| { apply_undo(doc_signal, history_signal); },
                        icon: IconKind::Undo,
                        variant: ButtonVariant::Tool { title: "Undo" }
                    }
                    IconButton {
                        test_id: "toolbar-redo",
                        state: if redo_disabled { ButtonState::Disabled } else { ButtonState::Default },
                        onclick: move |_| { apply_redo(doc_signal, history_signal); },
                        icon: IconKind::Redo,
                        variant: ButtonVariant::Tool { title: "Redo" }
                    }
                }

                div { class: "hidden md:flex items-center",
                    Divider {}

                    // Zoom
                    IconButton {
                        test_id: "zoom-in",
                        state: ButtonState::Default,
                        onclick: move |_| { let _ = apply_zoom_in(doc_signal, history_signal, *viewport_size_signal.read()); },
                        icon: IconKind::ZoomIn,
                        variant: ButtonVariant::Tool { title: "Zoom In" }
                    }
                    button {
                        "data-testid": "zoom-reset",
                        "data-zoom-percent": "{zoom_percent:.0}",
                        class: "text-foreground font-mono text-[13px] mx-2 cursor-pointer select-none hover:text-[var(--accent)] transition-colors w-[45px] text-center rounded outline-none focus-visible:outline focus-visible:outline-2 focus-visible:outline-[var(--accent)] focus-visible:outline-offset-2",
                        onclick: move |_| { let _ = apply_zoom_reset(doc_signal, history_signal, *viewport_size_signal.read()); },
                        title: "Reset zoom",
                        "aria-label": "Reset zoom",
                        "{zoom_percent:.0}%"
                    }
                    IconButton {
                        test_id: "zoom-out",
                        state: ButtonState::Default,
                        onclick: move |_| { let _ = apply_zoom_out(doc_signal, history_signal, *viewport_size_signal.read()); },
                        icon: IconKind::ZoomOut,
                        variant: ButtonVariant::Tool { title: "Zoom Out" }
                    }

                    Divider {}

                    // Delete
                    IconButton {
                        test_id: "toolbar-delete",
                        state: if stats.selected_count == 0 { ButtonState::Disabled } else { ButtonState::Default },
                        onclick: move |_| {
                            let selected_nodes: Vec<String> = {
                                let doc = doc_signal.read();
                                canvas_domain::selection_geometry::selected_node_ids(&doc)
                                    .into_iter()
                                    .map(|id| id.to_string())
                                    .collect()
                            };
                            let dispatch_result = crate::ui::dispatch::dispatch_node_delete_batch(&None, &selected_nodes);
                            match dispatch_result {
                                Ok(_) => crate::ui::commands::apply_clear_selection(doc_signal),
                                Err(_) => {
                                    let _ = apply_delete_selected(doc_signal, history_signal);
                                }
                            }
                        },
                        icon: IconKind::Trash,
                        variant: ButtonVariant::Destructive { title: "Delete" }
                    }

                    Divider {}

                    // Style settings
                    LabelButton {
                        test_id: "style-line",
                        icon: Some(IconKind::Minus),
                        text: "Solid",
                        onclick: None,
                    }
                    LabelButton {
                        test_id: "style-arrow-type",
                        icon: Some(IconKind::ChevronRight),
                        text: match *arrow_type_signal.read() {
                            diagram_models::document::ArrowType::Default => "Default",
                            diagram_models::document::ArrowType::Sharp => "Sharp",
                            diagram_models::document::ArrowType::Curved => "Curved",
                            diagram_models::document::ArrowType::Step => "Step",
                            diagram_models::document::ArrowType::Straight => "Straight",
                        },
                        onclick: move |_| {
                            use diagram_models::document::ArrowType;
                            let next_type = match *arrow_type_signal.read() {
                                ArrowType::Default => ArrowType::Sharp,
                                ArrowType::Sharp => ArrowType::Curved,
                                ArrowType::Curved => ArrowType::Step,
                                ArrowType::Step => ArrowType::Straight,
                                ArrowType::Straight => ArrowType::Default,
                            };
                            arrow_type_signal.set(next_type);
                            let _ = crate::ui::commands::apply_arrow_type_to_selection(doc_signal, history_signal, next_type);
                        }
                    }
                    IconButton {
                        test_id: "style-arrow",
                        state: ButtonState::Default,
                        onclick: move |_| {
                            let _ = apply_toggle_edge_direction(doc_signal, history_signal);
                        },
                        icon: IconKind::ArrowRight,
                        variant: ButtonVariant::Tool { title: "Toggle edge direction" }
                    }
                }
            }

                // Right section: Export/Stats
                div {
                    class: "flex items-center gap-2 shrink-0",

                    div {
                        class: "hidden sm:flex items-center",
                        IconButton {
                            test_id: "toolbar-export",
                            state: if is_dirty { ButtonState::Active } else { ButtonState::Default },
                            onclick: move |_| {
                                save_workspace(doc_signal, session_signal, toasts);
                            },
                            icon: IconKind::Upload,
                            variant: ButtonVariant::Tool { title: "Export (Ctrl+S)" }
                        }
                        IconButton {
                            test_id: "toolbar-import",
                            state: ButtonState::Default,
                            onclick: move |_| {
                                let signals = WorkspaceSignals {
                                    doc: doc_signal,
                                    session: session_signal,
                                    history: history_signal,
                                };
                                open_workspace(signals, toasts);
                            },
                            icon: IconKind::Download,
                            variant: ButtonVariant::Tool { title: "Import (Ctrl+O)" }
                        }
                    }

                    MobileToolbarMenu {}
                    ToolbarThemeToggle {}

                div {
                    class: "hidden lg:block w-[1px] h-6 bg-[var(--border-subtle)] ml-1 mr-3"
                }

                div {
                    class: "hidden lg:flex text-muted-foreground font-mono text-[12px] gap-3 whitespace-nowrap",
                    span { "data-testid": "counter-nodes", "data-count": "{stats.node_count}", "{stats.node_count} nodes" }
                    span { "data-testid": "counter-edges", "data-count": "{stats.edge_count}", "{stats.edge_count} edges" }
                    span { "data-testid": "counter-selected", "data-count": "{stats.selected_count}", "Rev {stats.revision}" }
                }
            }
        }
    }
}

pub const fn mutation_error_code(err: &MutationError) -> &'static str {
    match err {
        MutationError::Schema(_) => "schema_error",
        MutationError::Semantic(_) => "semantic_error",
    }
}
