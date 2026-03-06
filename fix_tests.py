with open("diagram_tool/src/ui/canvas/drag_math.rs", "r") as f:
    content = f.read()

content = content.replace(
    "use super::interaction_reducer::{InteractionMode};",
    "use crate::ui::canvas::interaction_reducer::{InteractionMode};"
)

content = content.replace(
    "use super::drag_math::calculate_resize_target_ids;",
    "use super::calculate_resize_target_ids;"
)

content = content.replace(
    "let targets = resize_target_ids(&doc);",
    """let selected = doc.editor_state.selected_items.iter().map(|s| crate::models::document::NodeId::new(s.clone())).collect::<Vec<_>>();
        let node_geometry = doc.document.nodes.iter().map(|(id, node)| {
            (id.clone(), (node.x.0, node.y.0, node.width.0, node.height.0, node.kind == crate::models::document::NodeKind::Subgraph))
        }).collect::<im::HashMap<_, _>>();
        let targets = super::calculate_resize_target_ids(&selected, &node_geometry);"""
)

content = content.replace(
    "let _ = super::finalize_motion_release(&mut mode, &mut doc);",
    "let _ = crate::ui::canvas::interaction_reducer::finalize_motion_release(&mut mode, &mut doc);"
)

with open("diagram_tool/src/ui/canvas/drag_math.rs", "w") as f:
    f.write(content)
