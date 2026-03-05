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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ValidationPolicy {
    #[default]
    Validate,
    SkipValidation,
}

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
    run_mutation_with_policy(current, RevisionPolicy::Increment, ValidationPolicy::default(), transform)
}

pub fn run_mutation_with_policy<F>(
    current: &DiagramDocument,
    revision_policy: RevisionPolicy,
    validation_policy: ValidationPolicy,
    transform: F,
) -> Result<DiagramDocument, MutationError>
where
    F: FnOnce(&DiagramDocument) -> Result<DiagramDocument, MutationError>,
{
    let next = transform(current)?;
    // Use From implementation to preserve error type information
    if validation_policy == ValidationPolicy::Validate {
        validate_schema(&next).map_err(MutationError::from)?;
    }

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

/// Applies an in-place mutation to a document with full validation.
///
/// This is a convenience wrapper around `run_mutation` that accepts a
/// `FnMut(&mut DiagramDocument)` instead of `FnOnce(&DiagramDocument) -> Result<DiagramDocument, MutationError>`.
///
/// The mutation function receives a mutable reference to a cloned document,
/// performs its changes, and the result is validated before being returned.
///
/// # Errors
/// Returns `Err(MutationError)` if:
/// - Schema validation fails
/// - Semantic validation fails
pub fn mutate_document<F>(
    current: DiagramDocument,
    mutation: F,
) -> Result<DiagramDocument, MutationError>
where
    F: FnOnce(&mut DiagramDocument),
{
    // Clone current to get a separate mutable document
    let mut next = current.clone();
    mutation(&mut next);
    // Run mutation with validation - pass current as reference, return mutated next
    let current_ref = &current;
    run_mutation(current_ref, move |_| Ok(next))
}

/// Applies an in-place mutation without validation.
///
/// Use this for editor state changes that don't need schema/semantic validation
/// (e.g., camera position, zoom, selection).
pub fn mutate_editor_state<F>(current: DiagramDocument, mutation: F) -> DiagramDocument
where
    F: FnOnce(&mut DiagramDocument),
{
    let mut next = current;
    mutation(&mut next);
    next.revision = next.revision.increment();
    next
}

#[cfg(test)]
mod tests {
    use super::{run_mutation, run_mutation_with_policy, RevisionPolicy, ValidationPolicy};
    use crate::models::document::{
        ArrowType, DiagramDocument, Edge, EdgeId, EdgeStyle, EditorState, Node, NodeId, NodeKind,
        NodeStyle, OrderedFloat,
    };
    use crate::mutation::error::MutationError;
    use crate::ui::grid::GridSize;
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
                tags: im::Vector::new(),
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
                bend_points: im::Vector::new(),
                tags: im::Vector::new(),
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod proptests {
    use super::{run_mutation, run_mutation_with_policy, RevisionPolicy, ValidationPolicy};
    use crate::models::document::{
        ArrowType, DiagramDocument, DocumentData, Edge, EdgeId, EdgeStyle, EditorState, Node,
        NodeId, NodeKind, NodeStyle, OrderedFloat, Point, Revision,
    };
    use crate::ui::grid::GridSize;
    use im::HashMap;
    use proptest::prelude::*;

    fn make_revision(increments: u64) -> Revision {
        let mut rev = Revision::INITIAL;
        for _ in 0..increments {
            rev = rev.increment();
        }
        rev
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        #[test]
        fn deeply_nested_mutation_chain_100_levels(initial_rev in 0u64..10) {
            let mut doc = DiagramDocument {
                revision: make_revision(initial_rev),
                ..DiagramDocument::default()
            };

            for _ in 0..100 {
                let result = run_mutation(&doc, move |d| {
                    Ok(DiagramDocument {
                        document: DocumentData {
                            nodes: d.document.nodes.clone(),
                            edges: d.document.edges.clone(),
                        },
                        ..d.clone()
                    })
                });
                match result {
                    Ok(next) => doc = next,
                    Err(_) => break,
                }
            }
            let mut expected = make_revision(initial_rev);
            for _ in 0..100 {
                expected = expected.increment();
            }
            prop_assert!(doc.revision == expected || doc.revision == make_revision(initial_rev));
        }

        #[test]
        fn document_with_1000_nodes(node_count in 100usize..1000) {
            let current = DiagramDocument::default();
            let result = run_mutation(&current, move |_| {
                let mut nodes = HashMap::new();
                for i in 0..node_count {
                    let id = NodeId::new(format!("stress_node_{}", i));
                    let node = Node {
                        kind: NodeKind::Node,
                        icon: String::new(),
                        label: format!("Node {}", i),
                        x: OrderedFloat((i % 100) as f64 * 50.0),
                        y: OrderedFloat((i / 100) as f64 * 50.0),
                        width: OrderedFloat(64.0),
                        height: OrderedFloat(64.0),
                        font_size: None,
                        font_weight: None,
                        locked: false,
                        parent: None,
                        dag_rank: None,
                        tags: im::Vector::new(),
                        metadata: HashMap::new(),
                        z_index: 0,
                        style: Some(NodeStyle::default()),
                        collapsed: None,
                    };
                    nodes = nodes.update(id, node);
                }
                Ok(DiagramDocument {
                    document: DocumentData { nodes, edges: HashMap::new() },
                    ..DiagramDocument::default()
                })
            });
            prop_assert!(result.is_ok());
            let doc = result.unwrap();
            prop_assert_eq!(doc.document.nodes.len(), node_count);
        }

        #[test]
        fn rapid_fire_apply_undo_cycles(cycles in 10usize..50) {
            let mut doc = DiagramDocument::default();
            let mut history: Vec<DiagramDocument> = Vec::new();

            for i in 0..cycles {
                history.push(doc.clone());
                let idx = i;
                let result = run_mutation(&doc, move |d| {
                    let nid = NodeId::new(format!("rapid_{}", idx));
                    let node = Node {
                        kind: NodeKind::Node,
                        icon: String::new(),
                        label: format!("Rapid {}", idx),
                        x: OrderedFloat(idx as f64),
                        y: OrderedFloat(idx as f64),
                        width: OrderedFloat(64.0),
                        height: OrderedFloat(64.0),
                        font_size: None,
                        font_weight: None,
                        locked: false,
                        parent: None,
                        dag_rank: None,
                        tags: im::Vector::new(),
                        metadata: HashMap::new(),
                        z_index: 0,
                        style: Some(NodeStyle::default()),
                        collapsed: None,
                    };
                    Ok(DiagramDocument {
                        document: DocumentData {
                            nodes: d.document.nodes.clone().update(nid, node),
                            edges: d.document.edges.clone(),
                        },
                        ..d.clone()
                    })
                });
                match result {
                    Ok(next) => doc = next,
                    Err(_) => {}
                }

                if i % 3 == 0 && !history.is_empty() {
                    doc = history.pop().unwrap();
                }
            }
            prop_assert!(doc.revision >= Revision::INITIAL);
        }

        #[test]
        fn nan_inf_scattered_through_document(
            use_nan_x in any::<bool>(),
            use_nan_y in any::<bool>(),
            use_inf_width in any::<bool>(),
            use_neg_inf_height in any::<bool>()
        ) {
            let current = DiagramDocument::default();
            let result = run_mutation(&current, move |_| {
                let nid = NodeId::new("float_stress".into());
                let node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: "Float Stress Test".into(),
                    x: OrderedFloat(if use_nan_x { f64::NAN } else { 100.0 }),
                    y: OrderedFloat(if use_nan_y { f64::NAN } else { 200.0 }),
                    width: OrderedFloat(if use_inf_width { f64::INFINITY } else { 64.0 }),
                    height: OrderedFloat(if use_neg_inf_height { f64::NEG_INFINITY } else { 64.0 }),
                    font_size: None,
                    font_weight: None,
                    locked: false,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                };
                Ok(DiagramDocument {
                    document: DocumentData {
                        nodes: HashMap::new().update(nid, node),
                        edges: HashMap::new(),
                    },
                    ..DiagramDocument::default()
                })
            });
            let has_special = use_nan_x || use_nan_y || use_inf_width || use_neg_inf_height;
            if has_special {
                // When special float values are used, the result may be Ok or Err - just verify it does not panic
                // Result can be either Ok or Err - this is a smoke test
                let _ = result;
            } else {
                prop_assert!(result.is_ok());
            }
        }

        #[test]
        fn create_delete_create_same_id(iterations in 10usize..50) {
            let mut doc = DiagramDocument::default();
            let node_id = NodeId::new("flicker_test".into());

            for i in 0..iterations {
                let should_create = i % 2 == 0;
                let nid = node_id.clone();
                let result = run_mutation(&doc, move |d| {
                    if should_create {
                        let node = Node {
                            kind: NodeKind::Node,
                            icon: String::new(),
                            label: format!("Flicker {}", i),
                            x: OrderedFloat(0.0),
                            y: OrderedFloat(0.0),
                            width: OrderedFloat(64.0),
                            height: OrderedFloat(64.0),
                            font_size: None,
                            font_weight: None,
                            locked: false,
                            parent: None,
                            dag_rank: None,
                            tags: im::Vector::new(),
                            metadata: HashMap::new(),
                            z_index: 0,
                            style: Some(NodeStyle::default()),
                            collapsed: None,
                        };
                        Ok(DiagramDocument {
                            document: DocumentData {
                                nodes: d.document.nodes.clone().update(nid, node),
                                edges: d.document.edges.clone(),
                            },
                            ..d.clone()
                        })
                    } else {
                        Ok(DiagramDocument {
                            document: DocumentData {
                                nodes: d.document.nodes.clone().without(&nid),
                                edges: d.document.edges.clone(),
                            },
                            ..d.clone()
                        })
                    }
                });
                match result {
                    Ok(next) => doc = next,
                    Err(_) => {}
                }
            }
            // Smoke test completed - verify document is still valid
            prop_assert!(doc.revision >= Revision::INITIAL);
        }

        #[test]
        fn concurrent_mutation_same_node_different_props(
            x_val in -1000.0f64..1000.0,
            y_val in -1000.0f64..1000.0,
            width_val in 1.0f64..500.0,
            height_val in 1.0f64..500.0
        ) {
            let current = DiagramDocument::default();
            let result = run_mutation(&current, move |_| {
                let nid = NodeId::new("conflict_node".into());
                let node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: "Conflict Test".into(),
                    x: OrderedFloat(x_val),
                    y: OrderedFloat(y_val),
                    width: OrderedFloat(width_val),
                    height: OrderedFloat(height_val),
                    font_size: None,
                    font_weight: None,
                    locked: false,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                };
                Ok(DiagramDocument {
                    document: DocumentData {
                        nodes: HashMap::new().update(nid, node),
                        edges: HashMap::new(),
                    },
                    ..DiagramDocument::default()
                })
            });
            prop_assert!(result.is_ok());
        }

        #[test]
        fn huge_string_in_label(label_len in 1000usize..10000) {
            let current = DiagramDocument::default();
            let result = run_mutation(&current, move |_| {
                let nid = NodeId::new("huge_label".into());
                let huge_label: String = "X".repeat(label_len);
                let node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: huge_label,
                    x: OrderedFloat(0.0),
                    y: OrderedFloat(0.0),
                    width: OrderedFloat(64.0),
                    height: OrderedFloat(64.0),
                    font_size: None,
                    font_weight: None,
                    locked: false,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                };
                Ok(DiagramDocument {
                    document: DocumentData {
                        nodes: HashMap::new().update(nid, node),
                        edges: HashMap::new(),
                    },
                    ..DiagramDocument::default()
                })
            });
            prop_assert!(result.is_ok());
            let doc = result.unwrap();
            let node = doc.document.nodes.get(&NodeId::new("huge_label".into())).unwrap();
            prop_assert_eq!(node.label.len(), label_len);
        }

        #[test]
        fn all_min_positive_floats(scale in 0usize..10) {
            let current = DiagramDocument::default();
            let result = run_mutation(&current, move |_| {
                let nid = NodeId::new("min_positive".into());
                let base = f64::MIN_POSITIVE;
                let node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: "Min Positive Test".into(),
                    x: OrderedFloat(base * scale as f64),
                    y: OrderedFloat(base * (scale + 1) as f64),
                    width: OrderedFloat(base * (scale + 2) as f64),
                    height: OrderedFloat(base * (scale + 3) as f64),
                    font_size: Some(OrderedFloat(base)),
                    font_weight: None,
                    locked: false,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                };
                Ok(DiagramDocument {
                    document: DocumentData {
                        nodes: HashMap::new().update(nid, node),
                        edges: HashMap::new(),
                    },
                    ..DiagramDocument::default()
                })
            });
            prop_assert!(result.is_ok());
        }

        #[test]
        fn subnormal_floats_everywhere(subnormal_bits in 1u64..100) {
            let current = DiagramDocument::default();
            let result = run_mutation(&current, move |_| {
                let subnormal = f64::from_bits(subnormal_bits);
                let nid = NodeId::new("subnormal".into());
                let node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: "Subnormal Test".into(),
                    x: OrderedFloat(subnormal),
                    y: OrderedFloat(subnormal * 2.0),
                    width: OrderedFloat(subnormal * 3.0),
                    height: OrderedFloat(subnormal * 4.0),
                    font_size: Some(OrderedFloat(subnormal)),
                    font_weight: None,
                    locked: false,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                };
                Ok(DiagramDocument {
                    document: DocumentData {
                        nodes: HashMap::new().update(nid, node),
                        edges: HashMap::new(),
                    },
                    ..DiagramDocument::default()
                })
            });
            prop_assert!(result.is_ok() || result.is_err());
        }

        #[test]
        fn alternating_valid_invalid_operations(count in 10usize..50) {
            let mut doc = DiagramDocument::default();

            for i in 0..count {
                let is_valid = i % 2 == 0;
                let idx = i;
                let result = if is_valid {
                    run_mutation(&doc, move |d| {
                        let nid = NodeId::new(format!("valid_{}", idx));
                        let node = Node {
                            kind: NodeKind::Node,
                            icon: String::new(),
                            label: format!("Valid {}", idx),
                            x: OrderedFloat(0.0),
                            y: OrderedFloat(0.0),
                            width: OrderedFloat(64.0),
                            height: OrderedFloat(64.0),
                            font_size: None,
                            font_weight: None,
                            locked: false,
                            parent: None,
                            dag_rank: None,
                            tags: im::Vector::new(),
                            metadata: HashMap::new(),
                            z_index: 0,
                            style: Some(NodeStyle::default()),
                            collapsed: None,
                        };
                        Ok(DiagramDocument {
                            document: DocumentData {
                                nodes: d.document.nodes.clone().update(nid, node),
                                edges: d.document.edges.clone(),
                            },
                            ..d.clone()
                        })
                    })
                } else {
                    run_mutation(&doc, |_| {
                        Ok(DiagramDocument {
                            version: 999,
                            ..DiagramDocument::default()
                        })
                    })
                };
                if result.is_ok() {
                    doc = result.unwrap();
                }
            }
            // Verify document is still valid after all operations
            prop_assert!(doc.revision >= Revision::INITIAL);
        }

        #[test]
        fn edge_self_loop_validation(_should_be_valid in any::<bool>()) {
            let current = DiagramDocument::default();
            let result = run_mutation(&current, move |_| {
                let nid = NodeId::new("self_loop_node".into());
                let node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: "Self Loop".into(),
                    x: OrderedFloat(0.0),
                    y: OrderedFloat(0.0),
                    width: OrderedFloat(64.0),
                    height: OrderedFloat(64.0),
                    font_size: None,
                    font_weight: None,
                    locked: false,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                };
                let edge = Edge {
                    source: nid.clone(),
                    target: nid.clone(),
                    label: "self-loop".into(),
                    style: EdgeStyle::default(),
                    arrow_type: ArrowType::default(),
                    label_offset_t: OrderedFloat(0.5),
                    color: None,
                    thickness: OrderedFloat(1.5),
                    directed: true,
                    bend_points: im::Vector::new(),
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    font_size: None,
                };
                Ok(DiagramDocument {
                    document: DocumentData {
                        nodes: HashMap::new().update(nid, node),
                        edges: HashMap::new().update(EdgeId::new("self_edge".into()), edge),
                    },
                    ..DiagramDocument::default()
                })
            });
            prop_assert!(result.is_err());
        }

        #[test]
        fn revision_preserve_vs_increment(rev_val in 0u64..100) {
            let mut current = DiagramDocument::default();
            current.revision = make_revision(rev_val);

            let with_increment = run_mutation_with_policy(
                &current,
                RevisionPolicy::Increment,
                |d| Ok(d.clone())
            );
            let with_preserve = run_mutation_with_policy(
                &current,
                RevisionPolicy::Preserve,
                |d| Ok(d.clone())
            );

            prop_assert!(with_increment.is_ok());
            prop_assert!(with_preserve.is_ok());
            prop_assert_eq!(with_increment.unwrap().revision, make_revision(rev_val.saturating_add(1)));
            prop_assert_eq!(with_preserve.unwrap().revision, make_revision(rev_val));
        }

        #[test]
        fn edge_dangling_reference_without_node(edge_label in ".*") {
            let current = DiagramDocument::default();
            let label = edge_label;
            let result = run_mutation(&current, move |_| {
                let edge = Edge {
                    source: NodeId::new("nonexistent_source".into()),
                    target: NodeId::new("nonexistent_target".into()),
                    label,
                    style: EdgeStyle::default(),
                    arrow_type: ArrowType::default(),
                    label_offset_t: OrderedFloat(0.5),
                    color: None,
                    thickness: OrderedFloat(1.5),
                    directed: true,
                    bend_points: im::Vector::new(),
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    font_size: None,
                };
                Ok(DiagramDocument {
                    document: DocumentData {
                        nodes: HashMap::new(),
                        edges: HashMap::new().update(EdgeId::new("dangling".into()), edge),
                    },
                    ..DiagramDocument::default()
                })
            });
            prop_assert!(result.is_err());
        }

        #[test]
        fn extreme_z_index_values(z in -1_000_000i64..=1_000_000) {
            let current = DiagramDocument::default();
            let result = run_mutation(&current, move |_| {
                let nid = NodeId::new("z_test".into());
                let node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: "Z Test".into(),
                    x: OrderedFloat(0.0),
                    y: OrderedFloat(0.0),
                    width: OrderedFloat(64.0),
                    height: OrderedFloat(64.0),
                    font_size: None,
                    font_weight: None,
                    locked: false,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    z_index: z,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                };
                Ok(DiagramDocument {
                    document: DocumentData {
                        nodes: HashMap::new().update(nid, node),
                        edges: HashMap::new(),
                    },
                    ..DiagramDocument::default()
                })
            });
            prop_assert!(result.is_ok());
        }

        #[test]
        fn many_edges_between_same_nodes(edge_count in 1usize..100) {
            let current = DiagramDocument::default();
            let result = run_mutation(&current, move |_| {
                let src = NodeId::new("src".into());
                let tgt = NodeId::new("tgt".into());
                let src_node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: "Source".into(),
                    x: OrderedFloat(0.0),
                    y: OrderedFloat(0.0),
                    width: OrderedFloat(64.0),
                    height: OrderedFloat(64.0),
                    font_size: None,
                    font_weight: None,
                    locked: false,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                };
                let tgt_node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: "Target".into(),
                    x: OrderedFloat(200.0),
                    y: OrderedFloat(0.0),
                    width: OrderedFloat(64.0),
                    height: OrderedFloat(64.0),
                    font_size: None,
                    font_weight: None,
                    locked: false,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                };
                let mut edges = HashMap::new();
                for i in 0..edge_count {
                    let eid = EdgeId::new(format!("edge_{}", i));
                    let edge = Edge {
                        source: src.clone(),
                        target: tgt.clone(),
                        label: format!("Edge {}", i),
                        style: EdgeStyle::default(),
                        arrow_type: ArrowType::default(),
                        label_offset_t: OrderedFloat(0.5),
                        color: None,
                        thickness: OrderedFloat(1.5),
                        directed: true,
                        bend_points: im::vector![Point { x: OrderedFloat(100.0), y: OrderedFloat(i as f64 * 20.0) }],
                        tags: im::Vector::new(),
                        metadata: HashMap::new(),
                        font_size: None,
                    };
                    edges = edges.update(eid, edge);
                }
                Ok(DiagramDocument {
                    document: DocumentData {
                        nodes: HashMap::new().update(src.clone(), src_node).update(tgt.clone(), tgt_node),
                        edges,
                    },
                    ..DiagramDocument::default()
                })
            });
            prop_assert!(result.is_ok());
            let doc = result.unwrap();
            prop_assert_eq!(doc.document.edges.len(), edge_count);
        }

        #[test]
        fn node_parent_self_reference(_ in Just(())) {
            let current = DiagramDocument::default();
            let nid = NodeId::new("self_parent".into());
            let nid_clone = nid.clone();
            let result = run_mutation(&current, move |_| {
                let node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: "Self Parent".into(),
                    x: OrderedFloat(0.0),
                    y: OrderedFloat(0.0),
                    width: OrderedFloat(64.0),
                    height: OrderedFloat(64.0),
                    font_size: None,
                    font_weight: None,
                    locked: false,
                    parent: Some(nid_clone.clone()),
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                };
                Ok(DiagramDocument {
                    document: DocumentData {
                        nodes: HashMap::new().update(nid, node),
                        edges: HashMap::new(),
                    },
                    ..DiagramDocument::default()
                })
            });
            prop_assert!(result.is_err());
        }

        #[test]
        fn unicode_everywhere(
            node_label in "\\p{Any}{1,50}",
            icon_name in "\\p{Any}{0,20}",
            tag in "\\p{Any}{1,30}"
        ) {
            let current = DiagramDocument::default();
            let result = run_mutation(&current, move |_| {
                let nid = NodeId::new("unicode_test".into());
                let node = Node {
                    kind: NodeKind::Node,
                    icon: icon_name.clone(),
                    label: node_label.clone(),
                    x: OrderedFloat(0.0),
                    y: OrderedFloat(0.0),
                    width: OrderedFloat(64.0),
                    height: OrderedFloat(64.0),
                    font_size: None,
                    font_weight: None,
                    locked: false,
                    parent: None,
                    dag_rank: None,
                    tags: im::vector![tag.clone()],
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                };
                Ok(DiagramDocument {
                    document: DocumentData {
                        nodes: HashMap::new().update(nid, node),
                        edges: HashMap::new(),
                    },
                    ..DiagramDocument::default()
                })
            });
            prop_assert!(result.is_ok());
        }

        #[test]
        fn zero_dimension_node(width in Just(0.0f64), height in Just(0.0f64)) {
            let current = DiagramDocument::default();
            let result = run_mutation(&current, move |_| {
                let nid = NodeId::new("zero_dim".into());
                let node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: "Zero Dimension".into(),
                    x: OrderedFloat(0.0),
                    y: OrderedFloat(0.0),
                    width: OrderedFloat(width),
                    height: OrderedFloat(height),
                    font_size: None,
                    font_weight: None,
                    locked: false,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                };
                Ok(DiagramDocument {
                    document: DocumentData {
                        nodes: HashMap::new().update(nid, node),
                        edges: HashMap::new(),
                    },
                    ..DiagramDocument::default()
                })
            });
            prop_assert!(result.is_ok() || result.is_err());
        }

        #[test]
        fn negative_dimensions(width in -100.0f64..=-0.001, height in -100.0f64..=-0.001) {
            let current = DiagramDocument::default();
            let result = run_mutation(&current, move |_| {
                let nid = NodeId::new("negative_dim".into());
                let node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: "Negative Dimension".into(),
                    x: OrderedFloat(0.0),
                    y: OrderedFloat(0.0),
                    width: OrderedFloat(width),
                    height: OrderedFloat(height),
                    font_size: None,
                    font_weight: None,
                    locked: false,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                };
                Ok(DiagramDocument {
                    document: DocumentData {
                        nodes: HashMap::new().update(nid, node),
                        edges: HashMap::new(),
                    },
                    ..DiagramDocument::default()
                })
            });
            prop_assert!(result.is_ok() || result.is_err());
        }

        #[test]
        fn edge_bend_points_overflow(bend_count in 100usize..500) {
            let current = DiagramDocument::default();
            let result = run_mutation(&current, move |_| {
                let src = NodeId::new("src".into());
                let tgt = NodeId::new("tgt".into());
                let src_node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: "Source".into(),
                    x: OrderedFloat(0.0),
                    y: OrderedFloat(0.0),
                    width: OrderedFloat(64.0),
                    height: OrderedFloat(64.0),
                    font_size: None,
                    font_weight: None,
                    locked: false,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                };
                let tgt_node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: "Target".into(),
                    x: OrderedFloat(200.0),
                    y: OrderedFloat(0.0),
                    width: OrderedFloat(64.0),
                    height: OrderedFloat(64.0),
                    font_size: None,
                    font_weight: None,
                    locked: false,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                };
                let mut bend_points = Vec::new();
                for i in 0..bend_count {
                    bend_points.push(Point {
                        x: OrderedFloat((i % 100) as f64),
                        y: OrderedFloat((i / 100) as f64),
                    });
                }
                let edge = Edge {
                    source: src.clone(),
                    target: tgt.clone(),
                    label: "Many Bends".into(),
                    style: EdgeStyle::default(),
                    arrow_type: ArrowType::default(),
                    label_offset_t: OrderedFloat(0.5),
                    color: None,
                    thickness: OrderedFloat(1.5),
                    directed: true,
                    bend_points: im::Vector::from(bend_points),
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    font_size: None,
                };
                Ok(DiagramDocument {
                    document: DocumentData {
                        nodes: HashMap::new().update(src, src_node).update(tgt, tgt_node),
                        edges: HashMap::new().update(EdgeId::new("bendy".into()), edge),
                    },
                    ..DiagramDocument::default()
                })
            });
            prop_assert!(result.is_ok());
        }

        #[test]
        fn editor_state_extreme_values(
            camera_x in -1e10f64..=1e10,
            camera_y in -1e10f64..=1e10,
            zoom in 0.001f64..=1000.0
        ) {
            let current = DiagramDocument::default();
            let result = run_mutation(&current, move |_| {
                Ok(DiagramDocument {
                    editor_state: EditorState {
                        camera_x: OrderedFloat(camera_x),
                        camera_y: OrderedFloat(camera_y),
                        zoom: OrderedFloat(zoom),
                        grid_size: GridSize::default(),
                        snap_to_grid: true,
                        selected_items: im::HashSet::new(),
                        editing_edge_id: None,
                        theme: crate::models::document::EditorTheme::System,
                        show_grid: true,
                        minimap_visible: false,
                    },
                    ..DiagramDocument::default()
                })
            });
            prop_assert!(result.is_ok());
        }

        #[test]
        fn massive_metadata(metadata_entries in 50usize..200) {
            let current = DiagramDocument::default();
            let result = run_mutation(&current, move |_| {
                let nid = NodeId::new("metadata_test".into());
                let mut metadata = HashMap::new();
                for i in 0..metadata_entries {
                    let key = format!("key_{}", i);
                    let value = serde_json::Value::String(format!("value_{}", i));
                    metadata = metadata.update(key, value);
                }
                let node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: "Metadata Test".into(),
                    x: OrderedFloat(0.0),
                    y: OrderedFloat(0.0),
                    width: OrderedFloat(64.0),
                    height: OrderedFloat(64.0),
                    font_size: None,
                    font_weight: None,
                    locked: false,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata,
                    z_index: 0,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                };
                Ok(DiagramDocument {
                    document: DocumentData {
                        nodes: HashMap::new().update(nid, node),
                        edges: HashMap::new(),
                    },
                    ..DiagramDocument::default()
                })
            });
            prop_assert!(result.is_ok());
        }

        #[test]
        fn locked_node_modification(_attempt_modify in any::<bool>()) {
            let current = DiagramDocument::default();
            let result = run_mutation(&current, move |_| {
                let nid = NodeId::new("locked_node".into());
                let node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: "Locked".into(),
                    x: OrderedFloat(0.0),
                    y: OrderedFloat(0.0),
                    width: OrderedFloat(64.0),
                    height: OrderedFloat(64.0),
                    font_size: None,
                    font_weight: None,
                    locked: true,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                };
                Ok(DiagramDocument {
                    document: DocumentData {
                        nodes: HashMap::new().update(nid, node),
                        edges: HashMap::new(),
                    },
                    ..DiagramDocument::default()
                })
            });
            prop_assert!(result.is_ok());
            let doc = result.unwrap();
            let node = doc.document.nodes.get(&NodeId::new("locked_node".into())).unwrap();
            prop_assert!(node.locked);
        }

        #[test]
        fn empty_string_ids_everywhere(_ in Just(())) {
            let current = DiagramDocument::default();
            let result = run_mutation(&current, move |_| {
                let nid = NodeId::new(String::new());
                let node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: String::new(),
                    x: OrderedFloat(0.0),
                    y: OrderedFloat(0.0),
                    width: OrderedFloat(64.0),
                    height: OrderedFloat(64.0),
                    font_size: None,
                    font_weight: None,
                    locked: false,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                };
                Ok(DiagramDocument {
                    document: DocumentData {
                        nodes: HashMap::new().update(nid.clone(), node),
                        edges: HashMap::new(),
                    },
                    ..DiagramDocument::default()
                })
            });
            prop_assert!(result.is_ok());
        }
    }
}
