#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::history::History;
use crate::hooks::e2e_reset::use_e2e_reset_hook;
use crate::hooks::keyboard::use_global_keyboard;
use crate::models::document::{ArrowType, DiagramDocument, EdgeStyle, Revision};
use crate::models::validation::validate_document_data;
use crate::ui::canvas::Canvas;
use crate::ui::commands::Clipboard;
use crate::ui::editor::ToolMode;
use crate::ui::minimap::Minimap;
use crate::ui::mobile::{use_sidebar_mobile_bridge, SidebarUiState};
use crate::ui::panels::PanelVisibility;
use crate::ui::sidebar::Sidebar;
use crate::ui::theme_provider::ThemeProvider;
use crate::ui::toast::{ToastQueue, Toaster};
use crate::ui::toolbar::{auto_save, Toolbar, ToolbarStats};

use crate::ui::ValidationPanel;
#[allow(unused_imports)]
use auto_save::AUTO_SAVE_KEY;
use dioxus::prelude::*;

const VALIDATION_IDLE_MS: u64 = 220;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraggedIconPayload {
    pub icon_key: String,
    pub label: Option<String>,
    pub image_data_url: Option<String>,
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
    use_context_provider(|| Signal::new(Option::<Clipboard>::None));
    use_context_provider(|| Signal::new(ToolMode::Select));
    use_context_provider(|| Signal::new(EdgeStyle::Solid));
    use_context_provider(|| Signal::new(ArrowType::Default));
    use_context_provider(|| Signal::new(ToastQueue::default()));
    use_context_provider(|| Signal::new(PanelVisibility::default()));
    use_context_provider(|| Signal::new(ToolbarStats::default()));
    use_context_provider(|| Signal::new(SidebarUiState::default()));
    use_context_provider(|| Signal::new((1200.0_f64, 800.0_f64)));
    // Shared counter that the Validate button can increment to force re-validation.
    use_context_provider(|| Signal::new(0_u64));

    use_global_keyboard();
    use_e2e_reset_hook();

    let doc_signal = use_context::<Signal<DiagramDocument>>();
    let validate_trigger = use_context::<Signal<u64>>();
    let sidebar_ui = use_context::<Signal<SidebarUiState>>();
    let panels = use_context::<Signal<PanelVisibility>>();
    let mut toolbar_stats = use_context::<Signal<ToolbarStats>>();

    use_sidebar_mobile_bridge(sidebar_ui, panels);

    // Auto-save: track last saved revision for change detection (WASM only)
    #[cfg(target_arch = "wasm32")]
    let mut last_saved_revision = use_signal(auto_save::default_revision);
    #[cfg(not(target_arch = "wasm32"))]
    let _last_saved_revision = auto_save::default_revision();

    let mut validation_issues = use_signal(move || {
        let doc = doc_signal.read();
        validate_document_data(&doc.document)
    });
    let mut last_validated_revision = use_signal(move || doc_signal.read().revision);
    let mut last_validate_trigger = use_signal(move || *validate_trigger.read());
    let mut queued_validation_revision = use_signal(|| Option::<Revision>::None);
    let mut validation_job = use_signal(|| 0_u64);

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
        if current_trigger != *last_validate_trigger.read() {
            let current_document = doc_signal.read().document.clone();
            validation_issues.set(validate_document_data(&current_document));
            last_validated_revision.set(doc_signal.read().revision);
            last_validate_trigger.set(current_trigger);
            queued_validation_revision.set(None);
            validation_job.with_mut(|job| {
                *job = job.saturating_add(1);
            });
            return;
        }

        let doc = doc_signal.read();
        let current_revision = doc.revision;
        let already_validated = current_revision == *last_validated_revision.read();
        let already_queued = queued_validation_revision
            .read()
            .as_ref()
            .is_some_and(|queued| *queued == current_revision);

        if already_validated || already_queued {
            return;
        }

        queued_validation_revision.set(Some(current_revision));

        let next_job = (*validation_job.read()).saturating_add(1);
        validation_job.set(next_job);
        let current_document = doc.document.clone();
        drop(doc);

        let validation_job_signal = validation_job;
        let mut validation_issues_signal = validation_issues;
        let mut last_validated_revision_signal = last_validated_revision;
        let mut queued_validation_revision_signal = queued_validation_revision;
        let mut eval = document::eval(&format!(
            "setTimeout(() => dioxus.send({{ job: {next_job} }}), {VALIDATION_IDLE_MS});"
        ));

        spawn(async move {
            let Ok(message) = eval.recv::<serde_json::Value>().await else {
                return;
            };
            let fired_job = message["job"].as_u64().map_or(0, |value| value);

            if fired_job != next_job {
                return;
            }

            if *validation_job_signal.read() != next_job {
                return;
            }

            let still_queued = queued_validation_revision_signal
                .read()
                .as_ref()
                .is_some_and(|queued| *queued == current_revision);

            if !still_queued {
                return;
            }

            validation_issues_signal.set(validate_document_data(&current_document));
            last_validated_revision_signal.set(current_revision);
            queued_validation_revision_signal.set(None);
        });
    });

    // Auto-save: Load from localStorage on first mount (WASM only)
    #[cfg(target_arch = "wasm32")]
    {
        let doc_signal = doc_signal.clone();
        let tool_signal = use_context::<Signal<ToolMode>>();
        let edge_style_signal = use_context::<Signal<EdgeStyle>>();
        let arrow_type_signal = use_context::<Signal<ArrowType>>();
        let last_saved_revision = last_saved_revision.clone();

        use_effect(move || {
            let mut eval = document::eval(&format!(
                r#"
                (() => {{
                    const key = "{AUTO_SAVE_KEY}";
                    let data = null;
                    try {{
                        data = localStorage.getItem(key);
                    }} catch (_) {{}}
                    dioxus.send({{ data }});
                }})();
                "#
            ));

            let mut doc_signal = doc_signal.clone();
            let mut tool_signal = tool_signal.clone();
            let mut edge_style_signal = edge_style_signal.clone();
            let mut arrow_type_signal = arrow_type_signal.clone();
            let mut last_saved_revision = last_saved_revision.clone();

            spawn(async move {
                if let Ok(msg) = eval.recv::<serde_json::Value>().await {
                    if let Some(data) =
                        msg["data"]
                            .as_str()
                            .and_then(|s| if s.is_empty() { None } else { Some(s) })
                    {
                        if let Ok(saved) = auto_save::deserialize_diagram(data) {
                            // Restore document state from localStorage
                            let mut doc = doc_signal.write();
                            *doc = saved.document;
                            last_saved_revision.set(doc.revision);

                            // Restore tool mode
                            if let Some(mode) = ToolMode::from_persisted_key(&saved.tool_mode) {
                                *tool_signal.write() = mode;
                            }

                            // Restore edge/arrow styles
                            *edge_style_signal.write() = saved.edge_style;
                            *arrow_type_signal.write() = saved.arrow_type;
                        }
                    }
                }
            });
        });
    }

    // Auto-save: Save to localStorage when document changes (WASM only)
    #[cfg(target_arch = "wasm32")]
    {
        let doc_signal = doc_signal.clone();
        let tool_signal = use_context::<Signal<ToolMode>>();
        let edge_style_signal = use_context::<Signal<EdgeStyle>>();
        let arrow_type_signal = use_context::<Signal<ArrowType>>();
        let mut last_saved_revision = last_saved_revision.clone();

        use_effect(move || {
            let doc = doc_signal.read();
            let current_revision = doc.revision;

            if auto_save::has_revision_changed(current_revision, Some(*last_saved_revision.read()))
            {
                let saved = auto_save::AutoSavedDiagram::new(
                    &doc,
                    &tool_signal.read(),
                    *edge_style_signal.read(),
                    *arrow_type_signal.read(),
                );

                if let Ok(json) = auto_save::serialize_diagram(&saved) {
                    if let Ok(payload_literal) = serde_json::to_string(&json) {
                        let _eval = document::eval(&format!(
                            r#"
                            (() => {{
                                try {{
                                    localStorage.setItem("{AUTO_SAVE_KEY}", {payload_literal});
                                }} catch (_) {{}}
                            }})();
                            "#
                        ));
                    }
                }

                last_saved_revision.set(current_revision);
            }
        });
    }

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
                if panels.read().validation {
                    ValidationPanel { issues: validation_issues }
                }
            }
        }
    }
}
