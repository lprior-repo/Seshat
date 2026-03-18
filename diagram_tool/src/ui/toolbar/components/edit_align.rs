use crate::history::History;
use crate::ui::theme::{ERROR, TEXT_MAIN};
use crate::ui::toolbar::components::base::ToolbarButton;
use crate::ui::toolbar::{actions, ToolbarStats};
use diagram_models::document::DiagramDocument;
use diagram_models::envelope::EventEnvelope;
use dioxus::prelude::*;

#[derive(Clone, Copy)]
enum EditAction {
    Delete,
    Copy,
    Paste,
    Back,
    Forward,
    ToBack,
    ToFront,
}

impl EditAction {
    fn info(&self) -> (&'static str, &'static str) {
        match self {
            Self::Delete => ("toolbar-delete", "Delete"),
            Self::Copy => ("toolbar-copy", "Copy"),
            Self::Paste => ("toolbar-paste", "Paste"),
            Self::Back => ("toolbar-send-backward", "Back"),
            Self::Forward => ("toolbar-bring-forward", "Forward"),
            Self::ToBack => ("toolbar-send-to-back", "To Back"),
            Self::ToFront => ("toolbar-bring-to-front", "To Front"),
        }
    }

    fn disabled(&self, stats: &ToolbarStats) -> bool {
        if let Self::Paste = self {
            !actions::can_paste()
        } else {
            stats.selected_count == 0
        }
    }

    fn exec(
        &self,
        doc: Signal<DiagramDocument>,
        hist: Signal<History>,
        tx: Option<Coroutine<EventEnvelope>>,
    ) {
        match self {
            Self::Delete => actions::delete_selection(doc, hist),
            Self::Copy => actions::copy_selection(doc),
            Self::Paste => actions::paste_selection(doc, hist),
            Self::Back => actions::send_backward(doc, hist),
            Self::Forward => actions::bring_forward(doc, hist),
            Self::ToBack => actions::send_to_back(doc, hist, tx),
            Self::ToFront => actions::bring_to_front(doc, hist, tx),
        }
    }
}

#[component]
pub fn EditGroup() -> Element {
    let doc = use_context::<Signal<DiagramDocument>>();
    let hist = use_context::<Signal<History>>();
    let tx = use_context::<Option<Coroutine<EventEnvelope>>>();
    let stats = *use_context::<Signal<ToolbarStats>>().read();

    rsx! {
        for action in [
            EditAction::Delete,
            EditAction::Copy,
            EditAction::Paste,
            EditAction::Back,
            EditAction::Forward,
            EditAction::ToBack,
            EditAction::ToFront,
        ] {
            {
                let (test_id, label) = action.info();
                let is_delete = matches!(action, EditAction::Delete);
                let color = if is_delete && stats.selected_count > 0 { ERROR } else { TEXT_MAIN };
                let opacity = if is_delete && stats.selected_count == 0 { 0.6 } else { 1.0 };

                rsx! {
                    ToolbarButton {
                        test_id,
                        onclick: move |_| action.exec(doc, hist, tx),
                        disabled: action.disabled(&stats),
                        color,
                        opacity,
                        "{label}"
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum AlignAction {
    Left,
    CenterH,
    Right,
    Top,
    CenterV,
    Bottom,
    DistH,
    DistV,
}

impl AlignAction {
    fn info(&self) -> (&'static str, &'static str, usize) {
        match self {
            Self::Left => ("toolbar-align-left", "Left", 2),
            Self::CenterH => ("toolbar-align-center-h", "H-Center", 2),
            Self::Right => ("toolbar-align-right", "Right", 2),
            Self::Top => ("toolbar-align-top", "Top", 2),
            Self::CenterV => ("toolbar-align-middle-v", "V-Center", 2),
            Self::Bottom => ("toolbar-align-bottom", "Bottom", 2),
            Self::DistH => ("toolbar-distribute-h", "Dist H", 3),
            Self::DistV => ("toolbar-distribute-v", "Dist V", 3),
        }
    }

    fn exec(&self, doc: Signal<DiagramDocument>, hist: Signal<History>) {
        match self {
            Self::Left => actions::align_left(doc, hist),
            Self::CenterH => actions::align_center_horizontal(doc, hist),
            Self::Right => actions::align_right(doc, hist),
            Self::Top => actions::align_top(doc, hist),
            Self::CenterV => actions::align_middle_vertical(doc, hist),
            Self::Bottom => actions::align_bottom(doc, hist),
            Self::DistH => actions::distribute_horizontal(doc, hist),
            Self::DistV => actions::distribute_vertical(doc, hist),
        }
    }
}

#[component]
pub fn AlignmentGroup() -> Element {
    let doc = use_context::<Signal<DiagramDocument>>();
    let hist = use_context::<Signal<History>>();
    let stats = *use_context::<Signal<ToolbarStats>>().read();

    rsx! {
        for action in [
            AlignAction::Left,
            AlignAction::CenterH,
            AlignAction::Right,
            AlignAction::Top,
            AlignAction::CenterV,
            AlignAction::Bottom,
            AlignAction::DistH,
            AlignAction::DistV,
        ] {
            {
                let (test_id, label, min_sel) = action.info();

                rsx! {
                    ToolbarButton {
                        test_id,
                        onclick: move |_| action.exec(doc, hist),
                        disabled: stats.selected_count < min_sel,
                        "{label}"
                    }
                }
            }
        }
    }
}
