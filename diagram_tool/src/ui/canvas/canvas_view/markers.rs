#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

#[must_use]
pub const fn edge_marker_ref(selected: bool) -> &'static str {
    if selected {
        "url(#arrowhead-selected)"
    } else {
        "url(#arrowhead)"
    }
}
