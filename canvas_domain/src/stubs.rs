use diagram_models::document::{DiagramDocument, NodeId};
use diagram_models::envelope::EventEnvelope;
use diagram_models::history::History;
use dioxus::prelude::*;
use im::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchError {
    Failed,
}

pub enum LabelTargetType {
    Node,
    Edge,
}

pub fn dispatch_update_label(
    _tx: &Option<Coroutine<EventEnvelope>>,
    _target_id: &str,
    _target_type: LabelTargetType,
    _old_label: &str,
    _new_label: &str,
) -> Result<(), DispatchError> {
    Ok(())
}

pub struct ResizeBounds;
impl ResizeBounds {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        _id: NodeId,
        _ox: f64,
        _oy: f64,
        _ow: f64,
        _oh: f64,
        _nx: f64,
        _ny: f64,
        _nw: f64,
        _nh: f64,
    ) -> Self {
        Self
    }
}

pub fn dispatch_node_resize(
    _tx: &Option<Coroutine<EventEnvelope>>,
    _bounds: ResizeBounds,
) -> Result<(), DispatchError> {
    Ok(())
}

pub fn mutate_doc_with_history<F, E>(
    doc_signal: &mut Signal<DiagramDocument>,
    history_signal: &mut Signal<History>,
    f: F,
) -> Result<(), E>
where
    F: FnOnce(&DiagramDocument) -> Result<DiagramDocument, E>,
{
    let current = doc_signal.read().clone();
    let new_doc = f(&current)?;
    let new_history = history_signal.read().push(current);
    *history_signal.write() = new_history;
    *doc_signal.write() = new_doc;
    Ok(())
}

#[must_use]
pub fn drag_original_positions(
    doc: &DiagramDocument,
    selected_items: &HashSet<String>,
) -> HashMap<NodeId, (f64, f64)> {
    let selected_nodes = selected_items
        .iter()
        .map(|id| diagram_models::document::NodeId::new(id.clone()))
        .filter(|id| doc.document.nodes.contains_key(id))
        .collect::<HashSet<_>>();

    let with_children = std::iter::successors(Some(selected_nodes), |current| {
        let expanded = doc
            .document
            .nodes
            .iter()
            .fold(current.clone(), |acc, (id, node)| {
                if node
                    .parent
                    .as_ref()
                    .is_some_and(|parent| acc.contains(parent))
                {
                    acc.update(id.clone())
                } else {
                    acc
                }
            });

        (expanded.len() > current.len()).then_some(expanded)
    })
    .last()
    .unwrap_or_else(HashSet::new);

    with_children.iter().fold(HashMap::new(), |acc, id| {
        if let Some(node) = doc.document.nodes.get(id) {
            acc.update(id.clone(), (node.x.0, node.y.0))
        } else {
            acc
        }
    })
}
