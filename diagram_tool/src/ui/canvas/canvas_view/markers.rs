#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use serde::Serialize;

/// Returns the marker URL for the end of an edge (pointing in direction of target).
#[must_use]
pub const fn edge_marker_ref(selected: bool) -> &'static str {
    if selected {
        "url(#arrowhead-selected)"
    } else {
        "url(#arrowhead)"
    }
}

/// Returns the marker URL for the start of a bidirectional edge (pointing toward source).
#[must_use]
pub const fn edge_marker_start_ref(selected: bool) -> &'static str {
    if selected {
        "url(#arrowhead-start-selected)"
    } else {
        "url(#arrowhead-start)"
    }
}

/// Marker references for rendering an edge.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct EdgeMarkers {
    pub marker_end: String,
    pub marker_start: Option<String>,
}

impl EdgeMarkers {
    /// Creates marker references based on edge direction.
    /// For bidirectional edges, includes both start and end markers.
    pub fn for_edge(directed: bool, bidirectional: bool, selected: bool) -> Self {
        if directed && bidirectional {
            Self {
                marker_end: edge_marker_ref(selected).to_string(),
                marker_start: Some(edge_marker_start_ref(selected).to_string()),
            }
        } else if directed {
            Self {
                marker_end: edge_marker_ref(selected).to_string(),
                marker_start: None,
            }
        } else {
            Self {
                marker_end: String::new(),
                marker_start: None,
            }
        }
    }
}
