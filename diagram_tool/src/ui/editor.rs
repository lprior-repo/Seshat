#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolMode {
    Select,
    Pan,
    Edge,
    Subgraph,
    Text,
    Draw,
}

impl ToolMode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Pan => "Pan",
            Self::Edge => "Edge",
            Self::Subgraph => "Subgraph",
            Self::Text => "Text",
            Self::Draw => "Draw",
        }
    }

    #[cfg(target_arch = "wasm32")]
    #[must_use]
    pub const fn persisted_key(self) -> &'static str {
        match self {
            Self::Select => "select",
            Self::Pan => "pan",
            Self::Edge => "edge",
            Self::Subgraph => "subgraph",
            Self::Text => "text",
            Self::Draw => "draw",
        }
    }

    #[cfg(target_arch = "wasm32")]
    #[allow(dead_code)]
    #[must_use]
    pub fn from_persisted_key(raw: &str) -> Option<Self> {
        match raw {
            "select" => Some(Self::Select),
            "pan" => Some(Self::Pan),
            "edge" => Some(Self::Edge),
            "subgraph" => Some(Self::Subgraph),
            "text" => Some(Self::Text),
            "draw" => Some(Self::Draw),
            _ => None,
        }
    }
}
