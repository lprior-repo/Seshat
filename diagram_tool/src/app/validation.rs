use dioxus::prelude::*;

use diagram_models::document::{DiagramDocument, Revision};
use diagram_models::validation::{validate_document, ValidationIssue};

use super::types::VALIDATION_IDLE_MS;

pub fn collect_validation_issues(doc: &DiagramDocument) -> Vec<ValidationIssue> {
    validate_document(doc)
}

pub fn use_validation_state(
    doc_signal: Signal<DiagramDocument>,
    validate_trigger: Signal<u64>,
) -> Signal<Vec<ValidationIssue>> {
    let mut validation_issues = use_signal(move || collect_validation_issues(&doc_signal.read()));
    let mut last_validated_revision = use_signal(move || doc_signal.read().revision);
    let mut last_validate_trigger = use_signal(move || *validate_trigger.read());
    let mut queued_validation_revision = use_signal(|| Option::<Revision>::None);
    let mut validation_job = use_signal(|| 0_u64);

    use_effect(move || {
        let current_trigger = *validate_trigger.read();
        if current_trigger != *last_validate_trigger.read() {
            let current_document = doc_signal.read().clone();
            validation_issues.set(collect_validation_issues(&current_document));
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
        let current_document = doc.clone();
        drop(doc);

        let mut eval = document::eval(&format!(
            "setTimeout(() => dioxus.send({{ job: {next_job} }}), {VALIDATION_IDLE_MS});"
        ));

        spawn(async move {
            let Ok(message) = eval.recv::<serde_json::Value>().await else {
                return;
            };
            let fired_job = message["job"].as_u64().map_or(0, |value| value);

            if fired_job != next_job || *validation_job.read() != next_job {
                return;
            }

            let still_queued = queued_validation_revision
                .read()
                .as_ref()
                .is_some_and(|queued| *queued == current_revision);

            if !still_queued {
                return;
            }

            validation_issues.set(collect_validation_issues(&current_document));
            last_validated_revision.set(current_revision);
            queued_validation_revision.set(None);
        });
    });

    validation_issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use diagram_models::document::{Edge, EdgeId, LockState, Node, NodeId, NodeKind, OrderedFloat};
    use diagram_models::validation::ValidationCode;
    use im::{HashMap, Vector};

    fn create_test_node(x: f64, y: f64, w: f64, h: f64) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "test".to_string(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(w),
            height: OrderedFloat(h),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    fn create_test_edge(source: &str, target: &str) -> Edge {
        Edge {
            source: NodeId::new(source.to_string()),
            target: NodeId::new(target.to_string()),
            label: "".to_string(),
            style: Default::default(),
            arrow_type: Default::default(),
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.5),
            directed: true,
            bend_points: Vector::new(),
            tags: Vector::new(),
            metadata: HashMap::new(),
            font_size: None,
            source_port: None,
            target_port: None,
        }
    }

    #[test]
    fn test_validation_happy_path() {
        let mut doc = DiagramDocument::default();
        doc.document.nodes.insert(
            NodeId::new("n1".to_string()),
            create_test_node(0.0, 0.0, 100.0, 100.0),
        );
        doc.document.nodes.insert(
            NodeId::new("n2".to_string()),
            create_test_node(200.0, 0.0, 100.0, 100.0),
        );
        doc.document
            .edges
            .insert(EdgeId::new("e1".to_string()), create_test_edge("n1", "n2"));

        let issues = collect_validation_issues(&doc);
        assert!(issues.is_empty(), "Expected no issues, got: {:?}", issues);
    }

    #[test]
    fn test_validation_invalid_numeric() {
        let mut doc = DiagramDocument::default();
        let bad_node = create_test_node(0.0, 0.0, -10.0, 100.0);
        doc.document
            .nodes
            .insert(NodeId::new("bad".to_string()), bad_node);

        let issues = collect_validation_issues(&doc);
        assert!(issues
            .iter()
            .any(|i| i.code == ValidationCode::INVALID_NUMERIC));
    }

    #[test]
    fn test_validation_dangling_edge() {
        let mut doc = DiagramDocument::default();
        doc.document.nodes.insert(
            NodeId::new("n1".to_string()),
            create_test_node(0.0, 0.0, 100.0, 100.0),
        );
        doc.document.edges.insert(
            EdgeId::new("e1".to_string()),
            create_test_edge("n1", "missing"),
        );

        let issues = collect_validation_issues(&doc);
        assert!(issues
            .iter()
            .any(|i| i.code == ValidationCode::EDGE_DANGLING));
    }

    #[test]
    fn test_validation_invalid_parent() {
        let mut doc = DiagramDocument::default();
        let mut child = create_test_node(0.0, 0.0, 100.0, 100.0);
        child.parent = Some(NodeId::new("missing_parent".to_string()));
        doc.document
            .nodes
            .insert(NodeId::new("child".to_string()), child);

        let issues = collect_validation_issues(&doc);
        assert!(issues
            .iter()
            .any(|i| i.code == ValidationCode::INVALID_PARENT));
    }

    #[test]
    fn test_validation_cycle() {
        let mut doc = DiagramDocument::default();
        doc.document.nodes.insert(
            NodeId::new("n1".to_string()),
            create_test_node(0.0, 0.0, 100.0, 100.0),
        );
        doc.document.nodes.insert(
            NodeId::new("n2".to_string()),
            create_test_node(200.0, 0.0, 100.0, 100.0),
        );
        doc.document
            .edges
            .insert(EdgeId::new("e1".to_string()), create_test_edge("n1", "n2"));
        doc.document
            .edges
            .insert(EdgeId::new("e2".to_string()), create_test_edge("n2", "n1"));

        let issues = collect_validation_issues(&doc);
        assert!(issues.iter().any(|i| i.code == ValidationCode::DAG_CYCLE));
    }
}
