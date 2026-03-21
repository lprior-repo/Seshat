use diagram_models::document::{Edge, Node, OrderedFloat};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Error {
    #[error("invalid scale factor")]
    InvalidScaleFactor,
    #[error("invalid node dimensions")]
    InvalidNodeDimensions,
    #[error("dimension overflow")]
    DimensionOverflow,
}

pub fn scale_node(node: &mut Node, scale_factor: f64) -> Result<(), Error> {
    if !scale_factor.is_finite() || scale_factor <= 0.0 {
        return Err(Error::InvalidScaleFactor);
    }

    let width = node.width.0;
    let height = node.height.0;

    if !width.is_finite() || width <= 0.0 || !height.is_finite() || height <= 0.0 {
        return Err(Error::InvalidNodeDimensions);
    }

    let new_width = width * scale_factor;
    let new_height = height * scale_factor;

    if !new_width.is_finite() || !new_height.is_finite() {
        return Err(Error::DimensionOverflow);
    }

    node.width = OrderedFloat::new_unchecked(new_width);
    node.height = OrderedFloat::new_unchecked(new_height);

    if let Some(OrderedFloat(fs)) = node.font_size {
        let new_fs = (fs * scale_factor).clamp(8.0, 72.0);
        node.font_size = Some(OrderedFloat::new_unchecked(new_fs));
    }

    Ok(())
}

pub fn scale_edge(edge: &mut Edge, scale_factor: f64) -> Result<(), Error> {
    if !scale_factor.is_finite() || scale_factor <= 0.0 {
        return Err(Error::InvalidScaleFactor);
    }

    if let Some(OrderedFloat(fs)) = edge.font_size {
        let new_fs = (fs * scale_factor).clamp(8.0, 72.0);
        edge.font_size = Some(OrderedFloat::new_unchecked(new_fs));
    }

    Ok(())
}

#[cfg(test)]
#[path = "scale_tests.rs"]
mod tests;
