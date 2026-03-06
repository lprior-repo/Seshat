import re

with open("diagram_tool/src/ui/canvas/interaction_reducer.rs", "r") as f:
    content = f.read()

# 1. Replace resize_target_ids
old_func = """fn resize_target_ids(doc: &DiagramDocument) -> Vec<NodeId> {
    let selected = selected_node_ids(doc);
    let selected_set = selected.iter().cloned().collect::<HashSet<_>>();

    let selected_subgraphs = selected
        .iter()
        .filter_map(|id| doc.document.nodes.get(id).map(|node| (id, node)))
        .filter(|(_, node)| node.kind == NodeKind::Subgraph)
        .map(|(_, node)| (node.x.0, node.y.0, node.width.0, node.height.0))
        .collect::<Vec<_>>();

    if selected_subgraphs.is_empty() {
        return selected;
    }

    doc.document
        .nodes
        .iter()
        .fold(selected_set, |acc, (id, node)| {
            let node_rect = (node.x.0, node.y.0, node.width.0, node.height.0);
            let included = selected_subgraphs
                .iter()
                .any(|subgraph_rect| crate::ui::canvas::math::within(*subgraph_rect, node_rect));
            if included {
                let mut updated = acc;
                let _ = updated.insert(id.clone());
                updated
            } else {
                acc
            }
        })
        .into_iter()
        .collect::<Vec<_>>()
}"""

new_func = """fn resize_target_ids(doc: &DiagramDocument) -> Vec<NodeId> {
    let selected = selected_node_ids(doc);
    let node_geometry = doc.document.nodes.iter().map(|(id, node)| {
        (id.clone(), (node.x.0, node.y.0, node.width.0, node.height.0, node.kind == NodeKind::Subgraph))
    }).collect::<im::HashMap<_, _>>();
    
    crate::ui::canvas::drag_math::calculate_resize_target_ids(&selected, &node_geometry)
}"""

content = content.replace(old_func, new_func)

# 2. Extract subgraph_tests and remove from interaction_reducer
subgraph_tests_start = content.find("/// Subgraph/container interaction tests (bd-sa6)")
subgraph_tests_end = content.find("// =============================================================================", subgraph_tests_start)

subgraph_tests_content = content[subgraph_tests_start:subgraph_tests_end]
content = content[:subgraph_tests_start] + content[subgraph_tests_end:]

# Write the updated interaction_reducer
with open("diagram_tool/src/ui/canvas/interaction_reducer.rs", "w") as f:
    f.write(content)

# Append subgraph_tests to drag_math.rs
# But first we need to replace some imports in subgraph_tests
subgraph_tests_content = subgraph_tests_content.replace(
    "use super::{resize_target_ids, InteractionMode};",
    "use super::interaction_reducer::{InteractionMode};\n    use super::drag_math::calculate_resize_target_ids;"
)
# We also need to rewrite the calls to resize_target_ids inside the tests.
# let targets = resize_target_ids(&doc);
# -> 
# let selected = doc.editor_state.selected_items.iter().map(|s| NodeId::new(s.clone())).collect::<Vec<_>>();
# let node_geometry = doc.document.nodes.iter().map(|(id, node)| { (id.clone(), (node.x.0, node.y.0, node.width.0, node.height.0, node.kind == NodeKind::Subgraph)) }).collect::<im::HashMap<_, _>>();
# let targets = calculate_resize_target_ids(&selected, &node_geometry);

with open("diagram_tool/src/ui/canvas/drag_math.rs", "a") as f:
    f.write("\n" + subgraph_tests_content)
