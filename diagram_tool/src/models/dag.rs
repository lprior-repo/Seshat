#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::document::{Edge, EdgeId, Node, NodeId};
use im::HashMap;
use std::collections::{HashSet, VecDeque};
use tap::Tap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CycleError {
    #[error("Cycle detected involving edge {0}")]
    CycleDetected(EdgeId),
}

/// Pure calculation to validate DAG property using Kahn's algorithm via state reduction.
pub fn validate_dag(
    nodes: &HashMap<NodeId, Node>,
    edges: &HashMap<EdgeId, Edge>,
) -> Result<(), CycleError> {
    let in_degree_init = nodes
        .keys()
        .map(|id| (id.clone(), 0))
        .collect::<HashMap<NodeId, usize>>();

    let (adjacency, in_degree) = edges
        .values()
        .filter(|e| nodes.contains_key(&e.source) && nodes.contains_key(&e.target))
        .fold(
            (HashMap::<NodeId, Vec<NodeId>>::new(), in_degree_init),
            |(adj, deg), edge| {
                (
                    adj.get(&edge.source).map_or_else(
                        || adj.update(edge.source.clone(), vec![edge.target.clone()]),
                        |neighbors| {
                            adj.update(
                                edge.source.clone(),
                                neighbors.clone().tap_mut(|n| n.push(edge.target.clone())),
                            )
                        },
                    ),
                    deg.get(&edge.target).map_or_else(
                        || deg.update(edge.target.clone(), 1),
                        |&count| deg.update(edge.target.clone(), count + 1),
                    ),
                )
            },
        );

    let initial_queue = in_degree
        .iter()
        .filter(|&(_, &deg)| deg == 0)
        .map(|(id, _)| id.clone())
        .collect::<VecDeque<NodeId>>();

    let final_state = (0..nodes.len()).fold(
        (initial_queue, in_degree, 0),
        |(mut q, degs, count), _| match q.pop_front() {
            Some(node_id) => {
                let neighbors = adjacency.get(&node_id).map_or_else(Vec::new, Clone::clone);
                let (next_q, next_degs) =
                    neighbors
                        .into_iter()
                        .fold((q, degs), |(mut cq, cd), neighbor| {
                            let next_count = cd
                                .get(&neighbor)
                                .copied()
                                .map_or(0, |c| c.saturating_sub(1));
                            if next_count == 0 {
                                cq.push_back(neighbor.clone());
                            }
                            (cq, cd.update(neighbor, next_count))
                        });
                (next_q, next_degs, count + 1)
            }
            None => (q, degs, count),
        },
    );

    if final_state.2 == nodes.len() {
        Ok(())
    } else {
        let cycle_nodes: HashSet<NodeId> = final_state
            .1
            .iter()
            .filter_map(|(id, &deg)| (deg != 0).then_some(id.clone()))
            .collect();

        Err(
            match edges.iter().find(|(_, edge)| {
                let endpoints_in_cycle = usize::from(cycle_nodes.contains(&edge.source))
                    + usize::from(cycle_nodes.contains(&edge.target));
                endpoints_in_cycle == 2
            }) {
                Some((id, _)) => CycleError::CycleDetected(id.clone()),
                None => CycleError::CycleDetected(EdgeId::new(String::from("unknown"))),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{validate_dag, CycleError};
    use crate::models::document::{
        ArrowType, Edge, EdgeId, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    };
    use im::HashMap;

    fn node() -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: String::new(),
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(100.0),
            height: OrderedFloat(60.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: im::vector![],
            metadata: HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    fn edge(source: &NodeId, target: &NodeId) -> Edge {
        Edge {
            source: source.clone(),
            target: target.clone(),
            label: String::new(),
            style: crate::models::document::EdgeStyle::Solid,
            arrow_type: ArrowType::Default,
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.5),
            directed: true,
            bend_points: im::vector![],
            tags: im::vector![],
            metadata: HashMap::new(),
            font_size: None,
        }
    }

    #[test]
    fn given_linear_graph_when_validated_then_it_is_acyclic() {
        let a = NodeId::new(String::from("a"));
        let b = NodeId::new(String::from("b"));
        let c = NodeId::new(String::from("c"));

        let nodes = HashMap::new()
            .update(a.clone(), node())
            .update(b.clone(), node())
            .update(c.clone(), node());

        let edges = HashMap::new()
            .update(EdgeId::new(String::from("e1")), edge(&a, &b))
            .update(EdgeId::new(String::from("e2")), edge(&b, &c));

        assert!(validate_dag(&nodes, &edges).is_ok());
    }

    #[test]
    fn given_cycle_when_validated_then_it_returns_cycle_error() {
        let a = NodeId::new(String::from("a"));
        let b = NodeId::new(String::from("b"));

        let nodes = HashMap::new()
            .update(a.clone(), node())
            .update(b.clone(), node());

        let edges = HashMap::new()
            .update(EdgeId::new(String::from("e1")), edge(&a, &b))
            .update(EdgeId::new(String::from("e2")), edge(&b, &a));

        let result = validate_dag(&nodes, &edges);
        assert!(result.is_err());
        assert!(matches!(result, Err(CycleError::CycleDetected(_))));
    }

    #[test]
    fn given_edge_with_missing_endpoint_when_validated_then_it_is_ignored_for_cycle_detection() {
        let a = NodeId::new(String::from("a"));
        let missing = NodeId::new(String::from("missing"));

        let nodes = HashMap::new().update(a.clone(), node());
        let edges = HashMap::new().update(EdgeId::new(String::from("e1")), edge(&a, &missing));

        assert!(validate_dag(&nodes, &edges).is_ok());
    }

    #[test]
    fn given_edge_with_missing_source_and_existing_target_when_validated_then_it_does_not_create_false_cycle(
    ) {
        let existing = NodeId::new(String::from("existing"));
        let missing = NodeId::new(String::from("missing"));

        let nodes = HashMap::new().update(existing.clone(), node());
        let edges =
            HashMap::new().update(EdgeId::new(String::from("e1")), edge(&missing, &existing));

        assert!(validate_dag(&nodes, &edges).is_ok());
    }

    #[test]
    fn given_two_incoming_edges_when_validated_then_degree_reduction_stays_acyclic() {
        let a = NodeId::new(String::from("a"));
        let b = NodeId::new(String::from("b"));
        let c = NodeId::new(String::from("c"));

        let nodes = HashMap::new()
            .update(a.clone(), node())
            .update(b.clone(), node())
            .update(c.clone(), node());

        let edges = HashMap::new()
            .update(EdgeId::new(String::from("e1")), edge(&a, &c))
            .update(EdgeId::new(String::from("e2")), edge(&b, &c));

        assert!(validate_dag(&nodes, &edges).is_ok());
    }

    #[test]
    fn given_reachable_cycle_after_acyclic_prefix_when_validated_then_cycle_is_detected() {
        let a = NodeId::new(String::from("a"));
        let b = NodeId::new(String::from("b"));
        let c = NodeId::new(String::from("c"));

        let nodes = HashMap::new()
            .update(a.clone(), node())
            .update(b.clone(), node())
            .update(c.clone(), node());

        let edges = HashMap::new()
            .update(EdgeId::new(String::from("e1")), edge(&a, &b))
            .update(EdgeId::new(String::from("e2")), edge(&b, &c))
            .update(EdgeId::new(String::from("e3")), edge(&c, &b));

        let result = validate_dag(&nodes, &edges);
        assert!(result.is_err());
    }

    #[test]
    fn given_mixed_edges_when_cycle_detected_then_reported_edge_is_from_cycle_component() {
        let a = NodeId::new(String::from("a"));
        let b = NodeId::new(String::from("b"));
        let c = NodeId::new(String::from("c"));
        let d = NodeId::new(String::from("d"));

        let cycle_e1 = EdgeId::new(String::from("cycle-1"));
        let cycle_e2 = EdgeId::new(String::from("cycle-2"));
        let tree_e = EdgeId::new(String::from("tree"));

        let nodes = HashMap::new()
            .update(a.clone(), node())
            .update(b.clone(), node())
            .update(c.clone(), node())
            .update(d.clone(), node());

        let edges = HashMap::new()
            .update(cycle_e1.clone(), edge(&a, &b))
            .update(cycle_e2.clone(), edge(&b, &a))
            .update(tree_e.clone(), edge(&c, &d));

        let result = validate_dag(&nodes, &edges);
        assert!(result.is_err());

        let reported = match result {
            Err(CycleError::CycleDetected(id)) => id,
            Ok(()) => EdgeId::new(String::from("unexpected-ok")),
        };

        assert!(reported == cycle_e1 || reported == cycle_e2);
        assert_ne!(reported, tree_e);
    }
}
