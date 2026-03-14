#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::geometry::primitives::{Rectangle, AABB};
use crate::models::document::{Node, NodeId};
use im::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarqueeMode {
    Contain,
    Intersect,
}

pub struct SpatialIndex {
    grid: HashMap<(i32, i32), Vec<NodeId>>,
    cell_size: f64,
}

pub fn build_spatial_index(nodes: &HashMap<NodeId, Node>) -> SpatialIndex {
    let cell_size = 100.0;
    let grid = nodes.iter().fold(HashMap::new(), |acc, (id, node)| {
        let aabb = get_node_aabb(node);

        let start_x = (aabb.min_x / cell_size).floor() as i32;
        let start_y = (aabb.min_y / cell_size).floor() as i32;
        let end_x = (aabb.max_x / cell_size).floor() as i32;
        let end_y = (aabb.max_y / cell_size).floor() as i32;

        (start_x..=end_x).fold(acc, |acc, x| {
            (start_y..=end_y).fold(acc, |acc, y| {
                let mut cell: Vec<NodeId> = acc.get(&(x, y)).cloned().unwrap_or_default();
                cell.push(id.clone());
                acc.update((x, y), cell)
            })
        })
    });

    SpatialIndex { grid, cell_size }
}

fn get_node_aabb(node: &Node) -> AABB {
    let rotation = node
        .metadata
        .get("rotation")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);

    Rectangle::new(node.x.0, node.y.0, node.width.0, node.height.0)
        .with_rotation(rotation)
        .aabb()
}

pub fn query_spatial_index(
    index: &SpatialIndex,
    nodes: &HashMap<NodeId, Node>,
    marquee: AABB,
    mode: MarqueeMode,
) -> HashSet<NodeId> {
    gather_candidates(index, &marquee)
        .into_iter()
        .filter(|id| {
            nodes.get(id).map_or(false, |node| {
                let node_aabb = get_node_aabb(node);
                match mode {
                    MarqueeMode::Contain => contains_aabb(&marquee, &node_aabb),
                    MarqueeMode::Intersect => intersects_aabb(&marquee, &node_aabb),
                }
            })
        })
        .collect()
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::document::OrderedFloat;

    fn create_node(x: f64, y: f64, w: f64, h: f64) -> Node {
        Node {
            kind: crate::models::document::NodeKind::Node,
            icon: String::new(),
            label: String::new(),
            x: OrderedFloat::new_unchecked(x),
            y: OrderedFloat::new_unchecked(y),
            width: OrderedFloat::new_unchecked(w),
            height: OrderedFloat::new_unchecked(h),
            font_size: None,
            font_weight: None,
            locked: false,
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
            let x = (i as f64 * 137.5) % 10000.0;
            let y = (i as f64 * 137.5 * 1.618) % 10000.0;
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
