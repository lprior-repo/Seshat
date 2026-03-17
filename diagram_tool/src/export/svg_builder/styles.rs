use super::fonts::xml_escape;
use diagram_models::document::Edge;

pub fn get_edge_stroke_color(edge: &Edge) -> String {
    edge.color
        .as_deref()
        .map_or_else(|| "black".to_string(), xml_escape)
}
