use crate::models::document::DiagramDocument;
use im::HashSet;

#[must_use]
pub fn with_auto_selected_edges(
    doc: &DiagramDocument,
    selected_items: &HashSet<String>,
) -> HashSet<String> {
    doc.document
        .edges
        .iter()
        .fold(selected_items.clone(), |acc, (id, edge)| {
            let source_selected = selected_items.contains(&edge.source.to_string());
            let target_selected = selected_items.contains(&edge.target.to_string());
            if source_selected && target_selected {
                acc.update(id.to_string())
            } else {
                acc
            }
        })
}
