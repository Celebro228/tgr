use std::vec;

pub use crate::info;
pub use crate::object::*;
pub use crate::physic::*;
pub use crate::render::*;

pub use Touch::*;
pub use View::*;

static mut WINDOW: Vec2 = Vec2::new(1280., 720.);
static mut WINDOW_UPDATE: bool = false;

static mut CANVAS: Vec2 = Vec2::new(1280., 720.);
static mut CANVAS_UPDATE: bool = false;

static mut VIEW_WIDTH: View = View::KeepHeight;
static mut VIEW_HEIGHT: View = View::KeepWidth;

static mut ADD_BUFFER: bool = true;

static mut CAMERA: Vec2 = Vec2::new(0., 0.);
static mut ZOOM: f32 = 1.;

static mut RESIZABLE: bool = true;
static mut FULLSCREEN: bool = false;
static mut HIGH_DPI: bool = true;

static mut SCRIPT: Option<&'static dyn Module> = None;
static mut NODE2D: Option<Node2d> = None;

static mut DELTA: f64 = 0.;
static mut LAST_FRAME_TIME: f64 = 0.;

static mut MOUSE: Vec2 = Vec2::new(0., 0.);
static mut MOUSE_DELTA: Vec2 = Vec2::new(0., 0.);

static mut TOUCH: bool = false;

pub enum View {
    KeepWidth,
    KeepHeight,
    Scale,
}

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

    pub(crate) fn touch(&mut self, id: u64, touch: &Touch, pos: Vec2) {
        unsafe {
            if let Some(node) = &mut NODE2D {
                set_touch(true);
                node.touch(id, touch, pos);
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

    pub fn view(self, width: View, height: View) -> Self {
        unsafe {
            VIEW_WIDTH = width;
            VIEW_HEIGHT = height;
        }
        self
    }

    pub fn camera(self, x: f32, y: f32) -> Self {
        set_camera(x, y);
        self
    }

    pub fn zoom(self, n: f32) -> Self {
        set_zoom(n);
        self
    }
}

#[inline(always)]
pub fn get_window() -> Vec2 {
    unsafe { WINDOW }
}
#[inline(always)]
pub(crate) fn get_window_update() -> bool {
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
pub fn set_canvas(x: f32, y: f32) {
    unsafe {
        CANVAS = vec2(x, y);
        CANVAS_UPDATE = true;
    }
}
#[inline(always)]
pub(crate) fn get_canvas_update() -> bool {
    unsafe {
        if CANVAS_UPDATE == true {
            CANVAS_UPDATE = false;
            true
        } else {
            false
        }
    }
}
#[inline(always)]
pub fn get_view_width() -> &'static View {
    unsafe { &VIEW_WIDTH }
}
#[inline(always)]
pub fn get_view_height() -> &'static View {
    unsafe { &VIEW_HEIGHT }
}
#[inline(always)]
pub(crate) fn set_add_buffer() {
    unsafe {
        ADD_BUFFER = true;
    }
}
#[inline(always)]
pub(crate) fn get_add_buffer() -> bool {
    unsafe {
        if ADD_BUFFER {
            ADD_BUFFER = false;
            true
        } else {
            false
        }
    }
}

#[inline(always)]
pub fn get_camera() -> Vec2 {
    unsafe { CAMERA }
}
#[inline(always)]
pub fn set_camera(x: f32, y: f32) {
    unsafe {
        CAMERA = vec2(x, y);
    }
}
#[inline(always)]
pub fn get_zoom() -> f32 {
    unsafe { ZOOM }
}
#[inline(always)]
pub fn set_zoom(n: f32) {
    unsafe {
        ZOOM = n;
    }
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
pub fn get_mouse() -> Vec2 {
    unsafe { MOUSE }
}
#[inline(always)]
pub(crate) fn set_mouse(x: f32, y: f32) {
    unsafe {
        MOUSE = vec2(x, y);
    }
}

#[inline(always)]
pub fn get_mouse_d() -> Vec2 {
    unsafe { MOUSE_DELTA }
}
#[inline(always)]
pub(crate) fn set_mouse_d(x: f32, y: f32) {
    unsafe {
        MOUSE_DELTA = vec2(x, y);
    }
}

#[inline(always)]
pub(crate) fn get_touch() -> bool {
    unsafe { TOUCH }
}
#[inline(always)]
pub(crate) fn set_touch(sel: bool) {
    unsafe {
        TOUCH = sel;
    }
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
