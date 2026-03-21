use super::{scale_edge, scale_node, Error};
use diagram_models::document::{
    ArrowType, Edge, EdgeStyle, FontWeight, LockState, Node, NodeId, NodeKind, OrderedFloat,
};

fn create_valid_node(width: f64, height: f64, font_size: Option<f64>) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: String::new(),
        x: OrderedFloat::new_unchecked(0.0),
        y: OrderedFloat::new_unchecked(0.0),
        width: OrderedFloat::new_unchecked(width),
        height: OrderedFloat::new_unchecked(height),
        font_size: font_size.map(OrderedFloat::new_unchecked),
        font_weight: Some(FontWeight::Normal),
        lock_state: LockState::Unlocked,
        parent: None,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: im::HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    }
}

fn create_valid_edge(font_size: Option<f64>) -> Edge {
    Edge {
        source: NodeId::new("n1".into()),
        target: NodeId::new("n2".into()),
        label: String::new(),
        style: EdgeStyle::Solid,
        arrow_type: ArrowType::Default,
        label_offset_t: OrderedFloat::new_unchecked(0.5),
        color: None,
        thickness: OrderedFloat::new_unchecked(1.0),
        directed: true,
        bend_points: im::Vector::new(),
        tags: im::Vector::new(),
        metadata: im::HashMap::new(),
        font_size: font_size.map(OrderedFloat::new_unchecked),
        source_port: None,
        target_port: None,
    }
}

#[test]
fn given_valid_node_with_font_when_scaled_up_then_dimensions_and_font_size_increase() {
    let mut node = create_valid_node(100.0, 50.0, Some(12.0));
    scale_node(&mut node, 2.0).unwrap();
    assert_eq!(node.width.0, 200.0);
    assert_eq!(node.height.0, 100.0);
    assert_eq!(node.font_size.unwrap().0, 24.0);
}

#[test]
fn given_valid_node_with_font_when_scaled_down_then_dimensions_and_font_size_decrease() {
    let mut node = create_valid_node(100.0, 50.0, Some(24.0));
    scale_node(&mut node, 0.5).unwrap();
    assert_eq!(node.width.0, 50.0);
    assert_eq!(node.height.0, 25.0);
    assert_eq!(node.font_size.unwrap().0, 12.0);
}

#[test]
fn given_valid_node_without_font_when_scaled_then_dimensions_change_and_font_remains_none() {
    let mut node = create_valid_node(100.0, 50.0, None);
    scale_node(&mut node, 2.0).unwrap();
    assert_eq!(node.width.0, 200.0);
    assert_eq!(node.height.0, 100.0);
    assert!(node.font_size.is_none());
}

#[test]
fn given_valid_edge_with_font_when_scaled_up_then_font_size_increases() {
    let mut edge = create_valid_edge(Some(12.0));
    scale_edge(&mut edge, 2.0).unwrap();
    assert_eq!(edge.font_size.unwrap().0, 24.0);
}

#[test]
fn given_valid_edge_with_font_when_scaled_down_then_font_size_decreases() {
    let mut edge = create_valid_edge(Some(24.0));
    scale_edge(&mut edge, 0.5).unwrap();
    assert_eq!(edge.font_size.unwrap().0, 12.0);
}

#[test]
fn given_valid_edge_without_font_when_scaled_then_font_remains_none() {
    let mut edge = create_valid_edge(None);
    scale_edge(&mut edge, 2.0).unwrap();
    assert!(edge.font_size.is_none());
}

#[test]
fn given_node_when_scaled_by_zero_then_returns_invalid_scale_factor_error() {
    let mut node = create_valid_node(100.0, 50.0, None);
    assert_eq!(scale_node(&mut node, 0.0), Err(Error::InvalidScaleFactor));
}

#[test]
fn given_node_when_scaled_by_negative_factor_then_returns_invalid_scale_factor_error() {
    let mut node = create_valid_node(100.0, 50.0, None);
    assert_eq!(scale_node(&mut node, -1.0), Err(Error::InvalidScaleFactor));
}

#[test]
fn given_node_when_scaled_by_nan_then_returns_invalid_scale_factor_error() {
    let mut node = create_valid_node(100.0, 50.0, None);
    assert_eq!(
        scale_node(&mut node, f64::NAN),
        Err(Error::InvalidScaleFactor)
    );
}

#[test]
fn given_node_when_scaled_by_infinity_then_returns_invalid_scale_factor_error() {
    let mut node = create_valid_node(100.0, 50.0, None);
    assert_eq!(
        scale_node(&mut node, f64::INFINITY),
        Err(Error::InvalidScaleFactor)
    );
}

#[test]
fn given_node_when_scaled_by_neg_infinity_then_returns_invalid_scale_factor_error() {
    let mut node = create_valid_node(100.0, 50.0, None);
    assert_eq!(
        scale_node(&mut node, f64::NEG_INFINITY),
        Err(Error::InvalidScaleFactor)
    );
}

#[test]
fn given_edge_when_scaled_by_zero_then_returns_invalid_scale_factor_error() {
    let mut edge = create_valid_edge(None);
    assert_eq!(scale_edge(&mut edge, 0.0), Err(Error::InvalidScaleFactor));
}

#[test]
fn given_edge_when_scaled_by_negative_factor_then_returns_invalid_scale_factor_error() {
    let mut edge = create_valid_edge(None);
    assert_eq!(scale_edge(&mut edge, -2.0), Err(Error::InvalidScaleFactor));
}

#[test]
fn given_edge_when_scaled_by_nan_then_returns_invalid_scale_factor_error() {
    let mut edge = create_valid_edge(None);
    assert_eq!(
        scale_edge(&mut edge, f64::NAN),
        Err(Error::InvalidScaleFactor)
    );
}

#[test]
fn given_edge_when_scaled_by_infinity_then_returns_invalid_scale_factor_error() {
    let mut edge = create_valid_edge(None);
    assert_eq!(
        scale_edge(&mut edge, f64::INFINITY),
        Err(Error::InvalidScaleFactor)
    );
}

#[test]
fn given_edge_when_scaled_by_neg_infinity_then_returns_invalid_scale_factor_error() {
    let mut edge = create_valid_edge(None);
    assert_eq!(
        scale_edge(&mut edge, f64::NEG_INFINITY),
        Err(Error::InvalidScaleFactor)
    );
}

#[test]
fn given_node_with_zero_width_when_scaled_then_returns_invalid_node_dimensions_error() {
    let mut node = create_valid_node(0.0, 50.0, None);
    assert_eq!(
        scale_node(&mut node, 2.0),
        Err(Error::InvalidNodeDimensions)
    );
}

#[test]
fn given_node_with_negative_width_when_scaled_then_returns_invalid_node_dimensions_error() {
    let mut node = create_valid_node(-10.0, 50.0, None);
    assert_eq!(
        scale_node(&mut node, 2.0),
        Err(Error::InvalidNodeDimensions)
    );
}

#[test]
fn given_node_with_nan_width_when_scaled_then_returns_invalid_node_dimensions_error() {
    let mut node = create_valid_node(f64::NAN, 50.0, None);
    assert_eq!(
        scale_node(&mut node, 2.0),
        Err(Error::InvalidNodeDimensions)
    );
}

#[test]
fn given_node_with_infinity_width_when_scaled_then_returns_invalid_node_dimensions_error() {
    let mut node = create_valid_node(f64::INFINITY, 50.0, None);
    assert_eq!(
        scale_node(&mut node, 2.0),
        Err(Error::InvalidNodeDimensions)
    );
}

#[test]
fn given_node_with_neg_infinity_width_when_scaled_then_returns_invalid_node_dimensions_error() {
    let mut node = create_valid_node(f64::NEG_INFINITY, 50.0, None);
    assert_eq!(
        scale_node(&mut node, 2.0),
        Err(Error::InvalidNodeDimensions)
    );
}

#[test]
fn given_node_with_zero_height_when_scaled_then_returns_invalid_node_dimensions_error() {
    let mut node = create_valid_node(100.0, 0.0, None);
    assert_eq!(
        scale_node(&mut node, 2.0),
        Err(Error::InvalidNodeDimensions)
    );
}

#[test]
fn given_node_with_negative_height_when_scaled_then_returns_invalid_node_dimensions_error() {
    let mut node = create_valid_node(100.0, -10.0, None);
    assert_eq!(
        scale_node(&mut node, 2.0),
        Err(Error::InvalidNodeDimensions)
    );
}

#[test]
fn given_node_with_nan_height_when_scaled_then_returns_invalid_node_dimensions_error() {
    let mut node = create_valid_node(100.0, f64::NAN, None);
    assert_eq!(
        scale_node(&mut node, 2.0),
        Err(Error::InvalidNodeDimensions)
    );
}

#[test]
fn given_node_with_infinity_height_when_scaled_then_returns_invalid_node_dimensions_error() {
    let mut node = create_valid_node(100.0, f64::INFINITY, None);
    assert_eq!(
        scale_node(&mut node, 2.0),
        Err(Error::InvalidNodeDimensions)
    );
}

#[test]
fn given_node_with_neg_infinity_height_when_scaled_then_returns_invalid_node_dimensions_error() {
    let mut node = create_valid_node(100.0, f64::NEG_INFINITY, None);
    assert_eq!(
        scale_node(&mut node, 2.0),
        Err(Error::InvalidNodeDimensions)
    );
}

#[test]
fn given_node_when_scaled_by_massive_factor_causing_overflow_then_returns_dimension_overflow_error()
{
    let mut node = create_valid_node(f64::MAX, f64::MAX, None);
    assert_eq!(scale_node(&mut node, 2.0), Err(Error::DimensionOverflow));
}

#[test]
fn given_node_with_exact_8_0_font_size_when_scaled_down_then_remains_exactly_8_0() {
    let mut node = create_valid_node(100.0, 50.0, Some(8.0));
    scale_node(&mut node, 0.5).unwrap();
    assert_eq!(node.font_size.unwrap().0, 8.0);
}

#[test]
fn given_node_with_exact_72_0_font_size_when_scaled_up_then_remains_exactly_72_0() {
    let mut node = create_valid_node(100.0, 50.0, Some(72.0));
    scale_node(&mut node, 2.0).unwrap();
    assert_eq!(node.font_size.unwrap().0, 72.0);
}

#[test]
fn given_edge_with_exact_8_0_font_size_when_scaled_down_then_remains_exactly_8_0() {
    let mut edge = create_valid_edge(Some(8.0));
    scale_edge(&mut edge, 0.5).unwrap();
    assert_eq!(edge.font_size.unwrap().0, 8.0);
}

#[test]
fn given_edge_with_exact_72_0_font_size_when_scaled_up_then_remains_exactly_72_0() {
    let mut edge = create_valid_edge(Some(72.0));
    scale_edge(&mut edge, 2.0).unwrap();
    assert_eq!(edge.font_size.unwrap().0, 72.0);
}

#[test]
fn given_node_when_scaled_down_massively_then_clamps_font_size_to_minimum() {
    let mut node = create_valid_node(100.0, 50.0, Some(12.0));
    scale_node(&mut node, 0.0001).unwrap();
    assert_eq!(node.font_size.unwrap().0, 8.0);
}

#[test]
fn given_node_when_scaled_up_massively_then_clamps_font_size_to_maximum() {
    let mut node = create_valid_node(100.0, 50.0, Some(12.0));
    scale_node(&mut node, 100.0).unwrap();
    assert_eq!(node.font_size.unwrap().0, 72.0);
}

#[test]
fn given_edge_when_scaled_down_massively_then_clamps_font_size_to_minimum() {
    let mut edge = create_valid_edge(Some(12.0));
    scale_edge(&mut edge, 0.0001).unwrap();
    assert_eq!(edge.font_size.unwrap().0, 8.0);
}

#[test]
fn given_edge_when_scaled_up_massively_then_clamps_font_size_to_maximum() {
    let mut edge = create_valid_edge(Some(12.0));
    scale_edge(&mut edge, 100.0).unwrap();
    assert_eq!(edge.font_size.unwrap().0, 72.0);
}
