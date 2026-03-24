mod helpers;
mod operations;
mod types;

pub use crate::clipboard::ClipboardData;
pub use operations::{
    compute_selection_centroid, delete_selection, move_selection, resize_selection,
    scale_selection_around_centroid,
};
pub use types::{Error, NonEmptyVec, Rect, Vector2D};
