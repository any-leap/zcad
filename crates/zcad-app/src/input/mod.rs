//! 输入处理模块

mod mouse;
pub mod keyboard;
mod snap;

pub use mouse::{handle_left_click, handle_right_click};
pub use keyboard::handle_keyboard_shortcuts;
pub use snap::{update_snap, get_effective_draw_point};
