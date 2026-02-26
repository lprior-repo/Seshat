#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::document::DiagramDocument;
use crate::models::schema::validate_schema;
use crate::models::validation::validate_document;
use crate::mutation::error::MutationError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RevisionPolicy {
    Increment,
    Preserve,
}

pub fn run_mutation<F>(
    current: &DiagramDocument,
    transform: F,
) -> Result<DiagramDocument, MutationError>
where
    F: FnOnce(&DiagramDocument) -> Result<DiagramDocument, MutationError>,
{
    run_mutation_with_policy(current, RevisionPolicy::Increment, transform)
}

pub fn run_mutation_with_policy<F>(
    current: &DiagramDocument,
    revision_policy: RevisionPolicy,
    transform: F,
) -> Result<DiagramDocument, MutationError>
where
    F: FnOnce(&DiagramDocument) -> Result<DiagramDocument, MutationError>,
{
    let next = transform(current)?;
    validate_schema(&next).map_err(|err| MutationError::Schema(err.to_string()))?;

    let issues = validate_document(&next);
    issues.first().map_or_else(
        || {
            let revision = match revision_policy {
                RevisionPolicy::Increment => current.revision.increment(),
                RevisionPolicy::Preserve => current.revision,
            };
            Ok(DiagramDocument { revision, ..next })
        },
        |issue| Err(MutationError::from_issue(issue)),
    )
}

#[cfg(test)]
mod tests {
    use super::{run_mutation, run_mutation_with_policy, RevisionPolicy};
    use crate::models::document::{
        ArrowType, DiagramDocument, Edge, EdgeId, EdgeStyle, Node, NodeId, NodeKind, NodeStyle,
        OrderedFloat,
    };
    use crate::mutation::error::MutationError;
    use im::HashMap;

    fn make_node(id: &str) -> (NodeId, Node) {
        (
            NodeId::new(id.to_string()),
            Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: id.to_string(),
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(64.0),
                height: OrderedFloat(64.0),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: None,
                dag_rank: None,
                tags: Vec::new(),
                metadata: HashMap::new(),
                z_index: 0,
                style: Some(NodeStyle::default()),
                collapsed: None,
            },
        )
    }

    fn make_edge(id: &str, src: &str, tgt: &str) -> (EdgeId, Edge) {
        (
            EdgeId::new(id.to_string()),
            Edge {
                source: NodeId::new(src.to_string()),
                target: NodeId::new(tgt.to_string()),
                label: String::new(),
                style: EdgeStyle::default(),
                arrow_type: ArrowType::default(),
                label_offset_t: OrderedFloat(0.5),
                color: None,
                thickness: OrderedFloat(1.5),
                directed: true,
                bend_points: Vec::new(),
                tags: Vec::new(),
                metadata: HashMap::new(),
                font_size: None,
            },
        )
    }

    #[test]
    fn given_invalid_version_transform_when_run_mutation_then_it_fails_closed_with_schema_error() {
        let current = DiagramDocument::default();
        let result = run_mutation(&current, |doc| {
            Ok(DiagramDocument {
                version: 99,
                ..doc.clone()
            })
        });

        assert!(matches!(result, Err(MutationError::Schema(_))));
    }

    #[test]
    fn given_valid_transform_when_run_mutation_then_revision_increments_once() {
        let current = DiagramDocument::default();
        let result = run_mutation(&current, |doc| Ok(doc.clone()));

        let next = result.ok();
        assert!(next.is_some());
        assert_eq!(
            next.map(|doc| doc.revision),
            Some(current.revision.increment())
        );
    }

    #[test]
    fn given_preserve_policy_when_run_mutation_then_revision_is_not_incremented() {
        let current = DiagramDocument::default();
        let result =
            run_mutation_with_policy(&current, RevisionPolicy::Preserve, |doc| Ok(doc.clone()));

        let next = result.ok();
        assert!(next.is_some());
        assert_eq!(next.map(|doc| doc.revision), Some(current.revision));
    }

    #[test]
    fn given_preserve_policy_with_stale_transformed_revision_when_run_mutation_then_current_revision_wins(
    ) {
        let mut current = DiagramDocument::default();
        current.revision = current.revision.increment();

        let result = run_mutation_with_policy(&current, RevisionPolicy::Preserve, |_| {
            Ok(DiagramDocument::default())
        });

        assert!(result.is_ok());
        assert_eq!(result.ok().map(|doc| doc.revision), Some(current.revision));
    }

    #[test]
    fn given_transform_that_creates_cycle_when_run_mutation_then_it_fails_closed() {
        let current = DiagramDocument::default();
        let result = run_mutation(&current, |_| {
            let (aid, a) = make_node("A");
            let (bid, b) = make_node("B");
            let (e1id, e1) = make_edge("e1", "A", "B");
            let (e2id, e2) = make_edge("e2", "B", "A");

            Ok(DiagramDocument {
                document: crate::models::document::DocumentData {
                    nodes: HashMap::new().update(aid, a).update(bid, b),
                    edges: HashMap::new().update(e1id, e1).update(e2id, e2),
                },
                ..DiagramDocument::default()
            })
        });

        assert!(matches!(result, Err(MutationError::Schema(_))));
    }
}
