#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::document::{Edge, EdgeId, Node, NodeId};
use im::HashMap;
use std::collections::VecDeque;
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
                            let next_count =
                                cd.get(&neighbor)
                                    .map_or(0, |&c| if c > 0 { c - 1 } else { 0 });
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
        Err(
            match edges.iter().find(|(_, edge)| {
                match (
                    final_state.1.get(&edge.target),
                    final_state.1.get(&edge.source),
                ) {
                    (Some(&td), Some(&sd)) => td > 0 && sd > 0,
                    _ => false,
                }
            }) {
                Some((id, _)) => CycleError::CycleDetected(id.clone()),
                None => CycleError::CycleDetected(EdgeId::new(String::from("unknown"))),
            },
        )
    }
}
