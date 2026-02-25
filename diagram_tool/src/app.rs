#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::history::History;
use crate::hooks::keyboard::use_global_keyboard;
use crate::models::document::{ArrowType, DiagramDocument, EdgeStyle};
use crate::models::validation::validate_document_data;
use crate::ui::canvas::Canvas;
use crate::ui::editor::ToolMode;
use crate::ui::minimap::Minimap;
use crate::ui::panels::PanelVisibility;
use crate::ui::properties::PropertiesPanel;
use crate::ui::sidebar::Sidebar;
use crate::ui::theme_provider::ThemeProvider;
use crate::ui::toast::{ToastQueue, Toaster};
use crate::ui::toolbar::{Toolbar, ToolbarStats};
use crate::ui::ValidationPanel;
use dioxus::prelude::*;

const MOBILE_BREAKPOINT: u32 = 768;
const SIDEBAR_OPEN_KEY: &str = "diagram_tool.sidebar_open";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SidebarUiState {
    pub is_mobile: bool,
    pub open: bool,
    pub open_mobile: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraggedIconPayload {
    pub icon_key: String,
    pub label: Option<String>,
}

impl Default for SidebarUiState {
    fn default() -> Self {
        Self {
            is_mobile: false,
            open: true,
            open_mobile: false,
        }
    }
}

#[allow(non_snake_case)]
#[allow(
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::needless_raw_string_hashes
)]
pub fn App() -> Element {
    use_context_provider(|| Signal::new(DiagramDocument::default()));
    let _dragging_icon = use_context_provider(|| Signal::new(Option::<DraggedIconPayload>::None));
    use_context_provider(|| Signal::new(History::new()));
    use_context_provider(|| Signal::new(ToolMode::Select));
    use_context_provider(|| Signal::new(EdgeStyle::Solid));
    use_context_provider(|| Signal::new(ArrowType::Arrow));
    use_context_provider(|| Signal::new(ToastQueue::default()));
    use_context_provider(|| Signal::new(PanelVisibility::default()));
    use_context_provider(|| Signal::new(ToolbarStats::default()));
    use_context_provider(|| Signal::new(SidebarUiState::default()));
    use_context_provider(|| Signal::new((1200.0_f64, 800.0_f64)));
    // Shared counter that the Validate button can increment to force re-validation.
    use_context_provider(|| Signal::new(0_u64));

    use_global_keyboard();

    let doc_signal = use_context::<Signal<DiagramDocument>>();
    let validate_trigger = use_context::<Signal<u64>>();
    let mut sidebar_ui = use_context::<Signal<SidebarUiState>>();
    let mut panels = use_context::<Signal<PanelVisibility>>();
    let mut toolbar_stats = use_context::<Signal<ToolbarStats>>();

    use_effect(move || {
        let mut eval = document::eval(&format!(
            r#"
                const BREAKPOINT = {};
                const readSidebarPreference = () => {{
                    let open = true;
                    try {{
                        const stored = localStorage.getItem('{}');
                        if (stored === 'true' || stored === 'false') {{
                            open = stored === 'true';
                        }}
                    }} catch (_) {{}}
                    dioxus.send({{ type: 'sidebar-open', open }});
                }};
                const emitViewport = () => {{
                    dioxus.send({{ type: 'viewport', isMobile: window.innerWidth < BREAKPOINT }});
                }};
                if (!window.__diagramToolViewportListenerInstalled) {{
                    window.__diagramToolViewportListenerInstalled = true;
                    window.addEventListener('resize', emitViewport);
                }}
                if (!window.__diagramToolSidebarHotkeyInstalled) {{
                    window.__diagramToolSidebarHotkeyInstalled = true;
                    window.addEventListener('keydown', (event) => {{
                        const active = document.activeElement;
                        const editing = active && (
                            active.tagName === 'INPUT' ||
                            active.tagName === 'TEXTAREA' ||
                            active.isContentEditable
                        );
                        const modifier = event.metaKey || event.ctrlKey;
                        const keyB = event.code === 'KeyB' || event.key === 'b' || event.key === 'B';
                        if (!event.defaultPrevented && modifier && keyB && !editing) {{
                            event.preventDefault();
                            dioxus.send({{ type: 'toggle-sidebar' }});
                        }}
                    }});
                }}
                if (!window.__diagramToolSidebarPrefLoaded) {{
                    window.__diagramToolSidebarPrefLoaded = true;
                    readSidebarPreference();
                }}
                if (!window.__diagramToolViewportMediaInstalled) {{
                    window.__diagramToolViewportMediaInstalled = true;
                    const mediaQuery = window.matchMedia('(max-width: {}px)');
                    mediaQuery.addEventListener('change', emitViewport);
                }}
                if (!window.__diagramToolViewportBooted) {{
                    window.__diagramToolViewportBooted = true;
                    emitViewport();
                }}
            "#,
            MOBILE_BREAKPOINT,
            SIDEBAR_OPEN_KEY,
            MOBILE_BREAKPOINT.saturating_sub(1)
        ));

        spawn(async move {
            while let Ok(msg) = eval.recv::<serde_json::Value>().await {
                match msg["type"].as_str().map_or("", |v| v) {
                    "viewport" => {
                        let is_mobile = msg["isMobile"].as_bool().is_some_and(|v| v);
                        sidebar_ui.with_mut(|state| {
                            state.is_mobile = is_mobile;
                            if !is_mobile {
                                state.open_mobile = false;
                            }
                        });
                    }
                    "sidebar-open" => {
                        let open = msg["open"].as_bool().unwrap_or(true);
                        sidebar_ui.with_mut(|state| {
                            state.open = open;
                        });
                    }
                    "toggle-sidebar" => {
                        if panels.read().sidebar {
                            sidebar_ui.with_mut(|state| {
                                if state.is_mobile {
                                    state.open_mobile = !state.open_mobile;
                                } else {
                                    state.open = !state.open;
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        let value = if state.open { "true" } else { "false" };
                                        let _eval = document::eval(&format!(
                                            "try {{ localStorage.setItem(\"{SIDEBAR_OPEN_KEY}\", \"{value}\"); }} catch (_) {{}}"
                                        ));
                                    }
                                }
                            });
                        } else {
                            panels.with_mut(|panel_state| panel_state.sidebar = true);
                            sidebar_ui.with_mut(|state| {
                                if state.is_mobile {
                                    state.open_mobile = true;
                                } else {
                                    state.open = true;
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        let _eval = document::eval(&format!(
                                            "try {{ localStorage.setItem(\"{SIDEBAR_OPEN_KEY}\", \"true\"); }} catch (_) {{}}"
                                        ));
                                    }
                                }
                            });
                        }
                    }
                    _ => {}
                }
            }
        });
    });

    let mut validation_issues = use_signal(move || {
        let doc = doc_signal.read();
        validate_document_data(&doc.document)
    });
    let mut last_validated_document = use_signal(move || doc_signal.read().document.clone());
    let mut last_validate_trigger = use_signal(move || *validate_trigger.read());

    use_effect(move || {
        let doc = doc_signal.read();
        let next = ToolbarStats {
            selected_count: doc.editor_state.selected_items.len(),
            node_count: doc.document.nodes.len(),
            edge_count: doc.document.edges.len(),
        };
        if *toolbar_stats.read() != next {
            toolbar_stats.set(next);
        }
    });

    use_effect(move || {
        let current_trigger = *validate_trigger.read();
        let current_document = doc_signal.read().document.clone();
        let should_validate = current_trigger != *last_validate_trigger.read()
            || current_document != *last_validated_document.read();

        if should_validate {
            validation_issues.set(validate_document_data(&current_document));
            last_validated_document.set(current_document);
            last_validate_trigger.set(current_trigger);
        }
    });

    rsx! {
        ThemeProvider {
            Toolbar {}
            Toaster {}

            div {
                display: "flex",
                flex: "1",
                overflow: "hidden",
                min_width: "0",

                if panels.read().sidebar {
                    Sidebar {}
                }
                div {
                    display: "flex",
                    flex: "1",
                    position: "relative",
                    Canvas {}
                    if panels.read().minimap {
                        Minimap {}
                    }
                }
                if panels.read().properties {
                    PropertiesPanel {}
                }
            }

            if panels.read().validation {
                ValidationPanel { issues: validation_issues }
            }
        }
    }
}
