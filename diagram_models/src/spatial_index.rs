#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use crate::document::{Node, NodeId};
use crate::geometry::{Point, Rectangle, AABB};
use ahash::AHashMap;
use im::{HashMap, HashSet};

/// Spatial index grid using `ahash::AHashMap` for WASM performance.
/// `AHashMap` uses a fixed seed so lookups work correctly.
/// This avoids the overhead of `im::HashMap`'s persistent data structure.
type SpatialGrid = AHashMap<(i32, i32), Vec<NodeId>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarqueeMode {
    Contain,
    Intersect,
}

pub struct SpatialIndex {
    grid: SpatialGrid,
    cell_size: f64,
}

#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn build_spatial_index(nodes: &HashMap<NodeId, Node>) -> SpatialIndex {
    let cell_size = 100.0;
    let mut grid: SpatialGrid = AHashMap::with_capacity(nodes.len());

    for (id, node) in nodes.iter() {
        let aabb = get_node_aabb(node);

        let start_x = (aabb.min_x / cell_size).floor() as i32;
        let start_y = (aabb.min_y / cell_size).floor() as i32;
        let end_x = (aabb.max_x / cell_size).floor() as i32;
        let end_y = (aabb.max_y / cell_size).floor() as i32;

        for x in start_x..=end_x {
            for y in start_y..=end_y {
                // Use entry API to avoid repeated lookups
                match grid.entry((x, y)) {
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(vec![id.clone()]);
                    }
                    std::collections::hash_map::Entry::Occupied(mut entry) => {
                        entry.get_mut().push(id.clone());
                    }
                }
            }
        }
    }

    SpatialIndex { grid, cell_size }
}

fn get_node_aabb(node: &Node) -> AABB {
    let rotation = node.rotation();

    Rectangle::new(node.x.0, node.y.0, node.width.0, node.height.0)
        .with_rotation(rotation)
        .aabb()
}

#[must_use]
pub fn query_spatial_index(
    index: &SpatialIndex,
    nodes: &HashMap<NodeId, Node>,
    marquee: AABB,
    mode: MarqueeMode,
) -> HashSet<NodeId> {
    gather_candidates(index, &marquee)
        .into_iter()
        .filter(|id| {
            nodes.get(id).is_some_and(|node| {
                let node_aabb = get_node_aabb(node);
                match mode {
                    MarqueeMode::Contain => contains_aabb(&marquee, &node_aabb),
                    MarqueeMode::Intersect => intersects_aabb(&marquee, &node_aabb),
                }
            })
        })
        .collect()
}

#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn gather_candidates(index: &SpatialIndex, marquee: &AABB) -> HashSet<NodeId> {
    let start_x = (marquee.min_x / index.cell_size).floor() as i32;
    let start_y = (marquee.min_y / index.cell_size).floor() as i32;
    let end_x = (marquee.max_x / index.cell_size).floor() as i32;
    let end_y = (marquee.max_y / index.cell_size).floor() as i32;

    (start_x..=end_x)
        .flat_map(|x| (start_y..=end_y).map(move |y| (x, y)))
        .filter_map(|cell_coords| index.grid.get(&cell_coords))
        .flatten()
        .cloned()
        .collect()
}

fn intersects_aabb(a: &AABB, b: &AABB) -> bool {
    a.min_x <= b.max_x && a.max_x >= b.min_x && a.min_y <= b.max_y && a.max_y >= b.min_y
}

fn contains_aabb(a: &AABB, b: &AABB) -> bool {
    b.min_x >= a.min_x && b.max_x <= a.max_x && b.min_y >= a.min_y && b.max_y <= a.max_y
}

/// Performs a point query against the spatial index.
/// Returns the node ID with the highest `z_index` that contains the point, if any.
///
/// This is optimized for hit testing - instead of scanning all nodes (O(n)),
/// it only checks nodes in the grid cell containing the point (O(1) average case).
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn point_query(
    index: &SpatialIndex,
    nodes: &HashMap<NodeId, Node>,
    point: &Point,
) -> Option<NodeId> {
    let cell_x = (point.x / index.cell_size).floor() as i32;
    let cell_y = (point.y / index.cell_size).floor() as i32;
    let cell_key = (cell_x, cell_y);

    index.grid.get(&cell_key).and_then(|cell_nodes| {
        // Find the node with highest z_index that contains the point
        cell_nodes
            .iter()
            .filter_map(|id| {
                nodes.get(id).and_then(|node| {
                    // Check if point is inside this node's bounds
                    let nx = node.x.0;
                    let ny = node.y.0;
                    let nw = node.width.0;
                    let nh = node.height.0;

                    // Fast bounds check first
                    if point.x < nx || point.x > nx + nw || point.y < ny || point.y > ny + nh {
                        return None;
                    }

                    // For nodes with rotation, we need the full AABB check
                    let rotation = node.rotation();

                    if rotation == 0.0 {
                        // Fast path: no rotation, bounds check was sufficient
                        Some((node.z_index, id.clone()))
                    } else {
                        // Full rotated AABB check
                        let node_aabb = get_node_aabb(node);
                        let point_aabb = AABB::new(point.x, point.y, point.x, point.y);
                        if intersects_aabb(&point_aabb, &node_aabb) {
                            Some((node.z_index, id.clone()))
                        } else {
                            None
                        }
                    }
                })
            })
            // Sort by z_index descending and take the top one
            .max_by_key(|(z_index, _)| *z_index)
            .map(|(_, id)| id)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{LockState, OrderedFloat};

    fn create_node(x: f64, y: f64, w: f64, h: f64) -> Node {
        Node {
            kind: crate::document::NodeKind::Node,
            icon: String::new(),
            label: String::new(),
            x: OrderedFloat::new_unchecked(x),
            y: OrderedFloat::new_unchecked(y),
            width: OrderedFloat::new_unchecked(w),
            height: OrderedFloat::new_unchecked(h),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    #[test]
    fn test_marquee_query_returns_correct_nodes_for_small_set() {
        let mut nodes = HashMap::new();
        let id1 = NodeId::new("n1".to_string());
        nodes.insert(id1.clone(), create_node(10.0, 10.0, 50.0, 50.0));

        let index = build_spatial_index(&nodes);
        let marquee = AABB::new(0.0, 0.0, 70.0, 70.0);
        let results = query_spatial_index(&index, &nodes, marquee, MarqueeMode::Contain);

        assert!(results.contains(&id1));
    }

    #[test]
    fn test_marquee_query_scales_to_3000_nodes_within_time_limit() {
        let mut nodes = HashMap::new();
        for i in 0..3000 {
            let id = NodeId::new(format!("n{i}"));
            let x = (f64::from(i) * 137.5) % 10000.0;
            let y = (f64::from(i) * 137.5 * 1.618) % 10000.0;
            nodes.insert(id, create_node(x, y, 50.0, 50.0));
        }

        let index = build_spatial_index(&nodes);
        let marquee = AABB::new(1000.0, 1000.0, 1500.0, 1500.0);

        let start_query = std::time::Instant::now();
        let results = query_spatial_index(&index, &nodes, marquee, MarqueeMode::Intersect);
        let query_duration = start_query.elapsed();

        assert!(
            query_duration.as_millis() < 16,
            "Query took too long: {query_duration:?}"
        );

        let expected: HashSet<_> = nodes
            .iter()
            .filter(|(_, n)| {
                let node_aabb = get_node_aabb(n);
                intersects_aabb(&marquee, &node_aabb)
            })
            .map(|(id, _)| id.clone())
            .collect();

        assert_eq!(results, expected);
    }
}
