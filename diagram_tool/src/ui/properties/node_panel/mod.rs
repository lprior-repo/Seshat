#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

mod core_props;
mod layout_props;
mod meta_props;
mod update;

use diagram_models::document::{Node, NodeId};
use dioxus::prelude::*;

use self::core_props::CorePropsPanel;
use self::layout_props::LayoutPropsPanel;
use self::meta_props::MetaPropsPanel;

#[derive(PartialEq, Clone, Props)]
pub struct NodePanelProps {
    pub id: NodeId,
    pub node: Node,
    pub connection_rows: Vec<(String, String, String)>,
}

#[component]
#[allow(clippy::approx_constant, clippy::float_cmp)]
pub fn NodePanel(props: NodePanelProps) -> Element {
    let id = props.id.clone();
    let node = props.node.clone();
    let connection_rows = props.connection_rows;

    rsx! {
        div {
            key: "{id}",
            style: "display: flex; flex-direction: column; gap: 10px;",
            CorePropsPanel { id: id.clone(), node: node.clone() }
            LayoutPropsPanel { id: id.clone(), node: node.clone() }
            MetaPropsPanel { id: id, node: node, connection_rows: connection_rows }
        }
    }
}
