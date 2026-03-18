use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconKind {
    FolderOpen,
    Save,
    Select,
    Pan,
    Edge,
    Subgraph,
    Undo,
    Redo,
    ZoomIn,
    ZoomOut,
    Trash,
}

impl IconKind {
    #[must_use]
    pub const fn path(self) -> &'static str {
        match self {
            Self::FolderOpen => "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4 M17 8l-5-5-5 5 M12 3v12",
            Self::Save => "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4 M7 10l5 5 5-5 M12 15V3",
            Self::Select => "M3 3l7.07 16.97 2.51-7.39 7.39-2.51L3 3z",
            Self::Pan => "M18 11V6a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v0 M14 10V4a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v2 M10 10.5V6a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v8 M6 14v1a7 7 0 0 0 14 0v-4a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v0",
            Self::Edge => "M5 12h14 M12 5l7 7-7 7",
            Self::Subgraph => "M19 5H5a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2z",
            Self::Undo => "M3 7v6h6 M3 13a9 9 0 0 1 15-6.7L21 9",
            Self::Redo => "M21 7v6h-6 M21 13A9 9 0 0 0 6 6.3L3 9",
            Self::ZoomIn => "M11 19A8 8 0 1 0 11 3a8 8 0 0 0 0 16z M21 21l-4.35-4.35 M11 8v6 M8 11h6",
            Self::ZoomOut => "M11 19A8 8 0 1 0 11 3a8 8 0 0 0 0 16z M21 21l-4.35-4.35 M8 11h6",
            Self::Trash => "M3 6h18 M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2 M10 11v6 M14 11v6",
        }
    }
}

#[component]
pub fn Icon(kind: IconKind, color: Option<&'static str>, size: Option<u32>) -> Element {
    let px = size.unwrap_or(20);
    let fill = color.unwrap_or("currentColor");

    rsx! {
        svg {
            width: "{px}",
            height: "{px}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: fill,
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "{kind.path()}" }
        }
    }
}
