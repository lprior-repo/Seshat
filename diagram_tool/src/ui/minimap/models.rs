#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use diagram_models::document::{DocumentData, NodeKind, Revision};

pub type EdgeSegment = (f64, f64, f64, f64);
pub type NodeRect = (bool, f64, f64, f64, f64, String);
pub type ProjectedNodeRect = (bool, f64, f64, f64, f64, &'static str);

#[derive(Clone)]
pub struct MinimapProjection {
    pub edge_segments: Vec<EdgeSegment>,
    pub node_rects: Vec<ProjectedNodeRect>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionKey {
    pub revision: Revision,
    pub min_x_bits: u64,
    pub min_y_bits: u64,
    pub scale_bits: u64,
}

impl ProjectionKey {
    pub const fn from_state(revision: Revision, min_x: f64, min_y: f64, scale: f64) -> Self {
        Self {
            revision,
            min_x_bits: min_x.to_bits(),
            min_y_bits: min_y.to_bits(),
            scale_bits: scale.to_bits(),
        }
    }
}

#[derive(Clone)]
pub struct MinimapSnapshot {
    pub edge_segments: Vec<EdgeSegment>,
    pub node_rects: Vec<NodeRect>,
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl MinimapSnapshot {
    pub fn from_document(document: &DocumentData) -> Option<Self> {
        if document.nodes.is_empty() {
            return None;
        }

        let mut edge_segments = Vec::new();
        let mut node_rects = Vec::new();
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for node in document.nodes.values() {
            let has_valid_geometry = node.x.0.is_finite()
                && node.y.0.is_finite()
                && node.width.0.is_finite()
                && node.height.0.is_finite();

            if !has_valid_geometry {
                continue;
            }

            min_x = min_x.min(node.x.0);
            min_y = min_y.min(node.y.0);
            max_x = max_x.max(node.x.0 + node.width.0);
            max_y = max_y.max(node.y.0 + node.height.0);

            let provider = node
                .tags
                .get(0)
                .map_or_else(|| String::from("generic"), Clone::clone);
            node_rects.push((
                node.kind == NodeKind::Subgraph,
                node.x.0,
                node.y.0,
                node.width.0,
                node.height.0,
                provider,
            ));
        }

        for edge in document.edges.values() {
            if let Some((source, target)) = document
                .nodes
                .get(&edge.source)
                .zip(document.nodes.get(&edge.target))
            {
                edge_segments.push((
                    source.x.0 + (source.width.0 / 2.0),
                    source.y.0 + (source.height.0 / 2.0),
                    target.x.0 + (target.width.0 / 2.0),
                    target.y.0 + (target.height.0 / 2.0),
                ));
            }
        }

        Some(Self {
            edge_segments,
            node_rects,
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }

    pub fn project(&self, min_x: f64, min_y: f64, scale: f64) -> MinimapProjection {
        let to_mini = |x: f64, y: f64| ((x - min_x) * scale, (y - min_y) * scale);
        let edge_segments = self
            .edge_segments
            .iter()
            .map(|(sxw, syw, txw, tyw)| {
                let (sx, sy) = to_mini(*sxw, *syw);
                let (tx, ty) = to_mini(*txw, *tyw);
                (sx, sy, tx, ty)
            })
            .collect();
        let node_rects = self
            .node_rects
            .iter()
            .map(|(is_subgraph, node_x, node_y, node_w, node_h, provider)| {
                let (x, y) = to_mini(*node_x, *node_y);
                let w = (*node_w * scale).max(2.0);
                let h = (*node_h * scale).max(2.0);
                (*is_subgraph, x, y, w, h, provider_color(provider))
            })
            .collect();

        MinimapProjection {
            edge_segments,
            node_rects,
        }
    }
}

fn provider_color(provider: &str) -> &'static str {
    match provider {
        "aws" => "#FF9900",
        "gcp" => "#4285F4",
        "azure" => "#0078D4",
        "k8s" => "#326CE5",
        _ => "#6B7280",
    }
}
