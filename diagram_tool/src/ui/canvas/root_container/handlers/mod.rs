pub mod double_click;
pub mod mouse_down;
pub mod mouse_move;
pub mod mouse_up;
pub mod wheel;

pub use double_click::handle_double_click;
pub use mouse_down::handle_mouse_down;
pub use mouse_move::handle_mouse_move;
pub use mouse_up::handle_mouse_up;
pub use wheel::handle_wheel;

#[cfg(test)]
pub mod tests;
