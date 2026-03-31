pub mod canvas_event;
pub mod input;
pub mod interaction_state;
pub mod transition;
pub mod types;

#[cfg(test)]
#[allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
pub mod test_utils;
#[cfg(test)]
#[allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
pub mod tests;

pub use canvas_event::{parse_event, CanvasEvent};
pub use interaction_state::{apply_drag_delta, DragState, InteractionState};
pub use transition::transition;
pub use transition::transition as reduce;
pub use types::{CanvasError, CanvasPoint, CanvasVector, RawEvent, SelectionBounds, SelectionMode};
