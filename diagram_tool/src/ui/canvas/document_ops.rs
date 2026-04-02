//(clippy::panic)]
//(clippy::cast_precision_loss)]
//(unsafe_code)]

// Re-export functions for testing
pub use super::canvas_view::{
    touch_handle_hit_test, touch_hit_radius, RESIZE_HANDLE_SIZE_PX, TOUCH_HIT_RADIUS_PX,
};

pub mod drag_commit;
pub mod mutations;
pub mod queries;

pub use drag_commit::*;
pub use mutations::*;
pub use queries::*;
