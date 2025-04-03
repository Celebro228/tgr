pub mod color;
pub mod draw;
pub mod object;
pub mod prelude;
pub mod render;

pub use macroquad::main;

use macroquad::window::Conf;
use draw::{window_h, window_w};

pub fn tgr_conf() -> Conf {
    Conf {
        window_title: "TGR".to_string(),
        window_width: window_w() as i32,
        window_height: window_h() as i32,
        high_dpi: true,
        fullscreen: false,
        sample_count: 1,
        window_resizable: true,
        ..Default::default()
    }
}