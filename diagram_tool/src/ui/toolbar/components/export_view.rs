use crate::ui::panels::PanelVisibility;
use crate::ui::theme::{ThemeMode, ACCENT, ACCENT_SOFT, BG_BASE, BORDER, TEXT_MAIN};
use crate::ui::toast::ToastQueue;
use crate::ui::toolbar::components::base::ToolbarButton;
use crate::ui::toolbar::{actions, export_actions};
use diagram_models::document::DiagramDocument;
use dioxus::prelude::*;

#[derive(Clone, Copy)]
enum ExportAction {
    Png,
    Svg,
    Json,
}

impl ExportAction {
    fn info(&self) -> (&'static str, &'static str) {
        match self {
            Self::Png => ("toolbar-export-png", "Export PNG"),
            Self::Svg => ("toolbar-export-svg", "Export SVG"),
            Self::Json => ("toolbar-export-json", "Export JSON"),
        }
    }

    fn exec(&self, doc: Signal<DiagramDocument>, toasts: Signal<ToastQueue>) {
        match self {
            Self::Png => export_actions::export_png(doc, toasts),
            Self::Svg => export_actions::export_svg(doc, toasts),
            Self::Json => export_actions::export_json(doc, toasts),
        }
    }
}

#[component]
pub fn ExportGroup() -> Element {
    let doc = use_context::<Signal<DiagramDocument>>();
    let toasts = use_context::<Signal<ToastQueue>>();

    rsx! {
        for action in [ExportAction::Png, ExportAction::Svg, ExportAction::Json] {
            {
                let (test_id, label) = action.info();

                rsx! {
                    ToolbarButton {
                        test_id,
                        onclick: move |_| action.exec(doc, toasts),
                        "{label}"
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum PanelToggle {
    Icons,
    Mini,
    Valid,
}

impl PanelToggle {
    fn info(&self) -> (&'static str, &'static str) {
        match self {
            Self::Icons => ("panel-icons-toggle", "Icons"),
            Self::Mini => ("panel-mini-toggle", "Mini"),
            Self::Valid => ("panel-valid-toggle", "Valid"),
        }
    }

    fn is_enabled(&self, panels: &PanelVisibility) -> bool {
        match self {
            Self::Icons => panels.sidebar,
            Self::Mini => panels.minimap,
            Self::Valid => panels.validation,
        }
    }

    fn toggle(&self, panels: &mut PanelVisibility) {
        match self {
            Self::Icons => panels.sidebar = !panels.sidebar,
            Self::Mini => panels.minimap = !panels.minimap,
            Self::Valid => panels.validation = !panels.validation,
        }
    }
}

#[component]
pub fn ViewAndThemeGroup() -> Element {
    let mut theme_mode_signal = use_context::<Signal<ThemeMode>>();
    let mut panel_visibility = use_context::<Signal<PanelVisibility>>();
    let doc_signal = use_context::<Signal<DiagramDocument>>();

    rsx! {
        select {
            style: "padding: 6px 8px; min-width: 110px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN}; cursor: pointer; font-size: 12px;",
            value: "{theme_mode_signal.read().persisted_key()}",
            onchange: move |evt| {
                if let Some(next) = ThemeMode::from_persisted_key(&evt.value()) {
                    theme_mode_signal.set(next);
                }
            },
            for mode in [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark] {
                option {
                    value: "{mode.persisted_key()}",
                    "{mode.label()} theme"
                }
            }
        }

        for toggle in [PanelToggle::Icons, PanelToggle::Mini, PanelToggle::Valid] {
            {
                let enabled = toggle.is_enabled(&panel_visibility.read());
                let (test_id, label) = toggle.info();
                let bg = if enabled { ACCENT_SOFT } else { BG_BASE };
                let border = if enabled { ACCENT } else { BORDER };

                rsx! {
                    ToolbarButton {
                        test_id,
                        onclick: move |_| panel_visibility.with_mut(|p| toggle.toggle(p)),
                        bg,
                        border,
                        extra_style: "font-size: 11px;",
                        "{label}"
                    }
                }
            }
        }

        {
            let grid_enabled = doc_signal.read().editor_state.show_grid;
            let grid_bg = if grid_enabled { ACCENT_SOFT } else { BG_BASE };
            let grid_border = if grid_enabled { ACCENT } else { BORDER };

            rsx! {
                ToolbarButton {
                    test_id: "grid-toggle",
                    onclick: move |_| actions::toggle_grid(doc_signal),
                    bg: grid_bg,
                    border: grid_border,
                    extra_style: "font-size: 11px;",
                    "Grid"
                }
            }
        }
    }
}
