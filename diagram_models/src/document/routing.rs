use super::{DiagramDocument, EdgeId};
use crate::document::edge::SerializedPoint;
use std::fmt;

pub const MAX_BEND_POINTS: usize = 10_000;

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeRoutingError {
    InvalidCoordinate(f64),
    EdgeNotFound(String),
    BendPointLimitExceeded(usize),
    NoOpClearRouting,
}

impl fmt::Display for EdgeRoutingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCoordinate(val) => write!(f, "Invalid coordinate value: {val}"),
            Self::EdgeNotFound(id) => write!(f, "Edge not found: {id}"),
            Self::BendPointLimitExceeded(len) => write!(f, "Bend point limit exceeded: {len}"),
            Self::NoOpClearRouting => {
                write!(f, "No operation clear routing: edge already has no routing")
            }
        }
    }
}

impl std::error::Error for EdgeRoutingError {}

pub trait EdgeRoutingMutator {
    /// Update the routing points for an edge.
    ///
    /// # Errors
    /// Returns `EdgeRoutingError` if the edge does not exist, coordinate is non-finite,
    /// or if the bend point limit is exceeded.
    fn update_edge_routing(
        &self,
        edge_id: &EdgeId,
        bend_points: im::Vector<SerializedPoint>,
    ) -> Result<Self, EdgeRoutingError>
    where
        Self: Sized;

    /// Clears the routing points for an edge.
    ///
    /// # Errors
    /// Returns `EdgeRoutingError` if the edge does not exist or if the routing is already empty.
    fn clear_edge_routing(&self, edge_id: &EdgeId) -> Result<Self, EdgeRoutingError>
    where
        Self: Sized;
}

fn validate_bend_points(points: &im::Vector<SerializedPoint>) -> Result<(), EdgeRoutingError> {
    if points.len() > MAX_BEND_POINTS {
        return Err(EdgeRoutingError::BendPointLimitExceeded(points.len()));
    }
    for pt in points {
        if !pt.x.0.is_finite() {
            return Err(EdgeRoutingError::InvalidCoordinate(pt.x.0));
        }
        if !pt.y.0.is_finite() {
            return Err(EdgeRoutingError::InvalidCoordinate(pt.y.0));
        }
    }
    Ok(())
}

fn get_mut_edge_for_routing<'a>(
    doc: &'a mut DiagramDocument,
    edge_id: &EdgeId,
) -> Result<&'a mut crate::document::edge::Edge, EdgeRoutingError> {
    doc.document
        .edges
        .get_mut(edge_id)
        .ok_or_else(|| EdgeRoutingError::EdgeNotFound(edge_id.to_string()))
}

impl EdgeRoutingMutator for DiagramDocument {
    fn update_edge_routing(
        &self,
        edge_id: &EdgeId,
        bend_points: im::Vector<SerializedPoint>,
    ) -> Result<Self, EdgeRoutingError> {
        validate_bend_points(&bend_points)?;
        let mut doc = self.clone();
        get_mut_edge_for_routing(&mut doc, edge_id)?.bend_points = bend_points;
        Ok(doc)
    }

    fn clear_edge_routing(&self, edge_id: &EdgeId) -> Result<Self, EdgeRoutingError> {
        let mut doc = self.clone();

        let edge = doc
            .document
            .edges
            .get_mut(edge_id)
            .ok_or_else(|| EdgeRoutingError::EdgeNotFound(edge_id.to_string()))?;

        if edge.bend_points.is_empty() {
            return Err(EdgeRoutingError::NoOpClearRouting);
        }

        edge.bend_points = im::Vector::new();

        Ok(doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::edge::SerializedPoint;
    use crate::document::OrderedFloat;
    use crate::document::{ArrowType, DiagramDocument, Edge, EdgeId, EdgeStyle, NodeId};
    #[test]
    fn r_test() {
        let mut doc = DiagramDocument::default();
        let e = Edge {
            source: NodeId::new("n1".into()),
            target: NodeId::new("n2".into()),
            label: "".into(),
            style: EdgeStyle::default(),
            arrow_type: ArrowType::default(),
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.0),
            directed: false,
            bend_points: im::Vector::new(),
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            font_size: None,
            source_port: None,
            target_port: None,
        };
        doc.document.edges.insert(EdgeId::new("e1".into()), e);
        let pts = im::vector![SerializedPoint {
            x: OrderedFloat(10.0),
            y: OrderedFloat(20.0)
        }];
        assert!(doc
            .update_edge_routing(&EdgeId::new("e1".into()), pts.clone())
            .is_ok());
        let mut d2 = doc.clone();
        d2.document
            .edges
            .get_mut(&EdgeId::new("e1".into()))
            .unwrap()
            .bend_points = pts;
        assert!(d2.clear_edge_routing(&EdgeId::new("e1".into())).is_ok());
    }
}
