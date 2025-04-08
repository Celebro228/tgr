use crate::node2d;
pub use crate::object::*;
pub use crate::physic::*;
pub use crate::render::*;

pub use crate::info;

static mut WINDOW: Vec2 = Vec2::new(1280., 720.);
static mut WINDOW_UPDATE: bool = false;

static mut CANVAS: Vec2 = Vec2::new(1280., 720.);
static mut CANVAS_UPDATE: bool = false;

static mut RESIZABLE: bool = true;
static mut FULLSCREEN: bool = false;
static mut HIGH_DPI: bool = true;

static mut SCRIPT: Option<&'static dyn Module> = None;
static mut NODE2D: Option<Node2d> = None;

static mut DELTA: f64 = 0.;
static mut LAST_FRAME_TIME: f64 = 0.;

pub struct Engine;

impl Engine {
    pub fn start(self, name: &str) -> Self {
        unsafe {
            if let Some(node) = &mut NODE2D {
                node.start();
            }
        }

        Render::start(name);

        self
    }

    pub(crate) fn update() {
        unsafe {
            if let Some(node) = &mut NODE2D {
                node.update();
            }
        }
    }

    pub(crate) fn draw2d() {
        unsafe {
            if let Some(node) = &mut NODE2D {
                node.draw();
            }
        }
    }

    pub fn node2d(self, node: Node2d) -> Self {
        unsafe {
            NODE2D = Some(node);
        }
        self
    }

    pub fn script(self, script: &'static dyn Module) -> Self {
        unsafe {
            SCRIPT = Some(script);
        }
        self
    }

    pub fn window(self, x: f32, y: f32) -> Self {
        unsafe {
            WINDOW = vec2(x, y);
        }
        self
    }

    pub fn canvas(self, x: f32, y: f32) -> Self {
        unsafe {
            CANVAS = vec2(x, y);
        }
        self
    }

    pub fn resizable(self, sel: bool) -> Self {
        unsafe {
            RESIZABLE = sel;
        }
        self
    }

    pub fn fullscreen(self, sel: bool) -> Self {
        unsafe {
            FULLSCREEN = sel;
        }
        self
    }

    pub fn high_dpi(self, sel: bool) -> Self {
        unsafe {
            HIGH_DPI = sel;
        }
        self
    }
}

#[inline(always)]
pub fn get_window() -> Vec2 {
    unsafe { WINDOW }
}
#[inline(always)]
pub fn get_window_update() -> bool {
    unsafe {
        if WINDOW_UPDATE == true {
            WINDOW_UPDATE = false;
            true
        } else {
            false
        }
    }
}
#[inline(always)]
pub fn set_window(x: f32, y: f32) {
    unsafe {
        WINDOW = vec2(x, y);
        WINDOW_UPDATE = true;
    }
}
#[inline(always)]
pub(crate) fn set_window_2(x: f32, y: f32) {
    unsafe {
        WINDOW = vec2(x, y);
    }
}
#[inline(always)]
pub fn get_canvas() -> Vec2 {
    unsafe { CANVAS }
}

#[inline(always)]
pub fn get_window_resizable() -> bool {
    unsafe { RESIZABLE }
}
#[inline(always)]
pub fn get_fullscreen() -> bool {
    unsafe { FULLSCREEN }
}
#[inline(always)]
pub fn get_high_dpi() -> bool {
    unsafe { HIGH_DPI }
}

#[inline(always)]
pub(crate) fn get_delta() -> f64 {
    unsafe { DELTA }
}
#[inline(always)]
pub(crate) fn set_delta(delta: f64) {
    unsafe {
        DELTA = delta;
    }
}
#[inline(always)]
pub(crate) fn get_last_frame_time() -> f64 {
    unsafe { LAST_FRAME_TIME }
}
#[inline(always)]
pub(crate) fn set_last_frame_time(last_frame_time: f64) {
    unsafe {
        LAST_FRAME_TIME = last_frame_time;
    }
}
