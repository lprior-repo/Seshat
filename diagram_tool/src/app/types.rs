use crate::history::History;
use diagram_models::document::DiagramDocument;

pub const VALIDATION_IDLE_MS: u64 = 220;

#[derive(Clone)]
pub struct DiagramTab {
    pub id: String,
    pub name: String,
    pub doc: DiagramDocument,
    pub history: History,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraggedIconPayload {
    pub icon_key: String,
    pub label: Option<String>,
    pub image_url: Option<String>,
}
