use crate::{
    object::{
        d2::{Node2d, ON_TOUCH},
        Touch,
    },
    render::{
        d2::{
            upd_proj, CAMERA, CANVAS, CANVAS_UPDATE, RENDERS, UPD_RENDER_BUFFER, VIEW_HEIGHT,
            VIEW_WIDTH, ZOOM,
        },
        Rgba, View, FPS, FPS_BUFFER, LAST_FPS_TIME, LAST_FRAME_TIME, WINDOW, WINDOW_UPDATE,
    },
};

#[cfg(feature = "miniquad")]
use crate::render::miniquad::render;

#[cfg(feature = "wgpu")]
use crate::render::wgpu::render;

use glam::{vec2, Vec2};

pub const BLACK: Rgba = Rgba::new(0., 0., 0., 1.0);

pub(crate) static mut BACKGRAUND: Rgba = BLACK;

pub(crate) static mut RESIZABLE: bool = true;
pub(crate) static mut FULLSCREEN: bool = false;
pub(crate) static mut HIGH_DPI: bool = true;

//static mut SCRIPT: Option<&'static dyn Module> = None;
static mut NODE2D: Option<Node2d> = None;

pub(crate) static mut MOUSE: Vec2 = Vec2::new(0., 0.);
pub(crate) static mut MOUSE_DELTA: Vec2 = Vec2::new(0., 0.);
pub(crate) static mut MOUSE_WHEEL_DELTA: Vec2 = Vec2::new(0., 0.);

pub struct Engine;

impl Engine {
    pub fn start(self, name: &str) -> Self {
        render(name);
        self
    }

    pub(crate) fn update() {
        unsafe {
            if let Some(node) = &mut NODE2D {
                node.update();
            }
        }
    }

    pub(crate) fn draw() {
        unsafe {
            MOUSE_DELTA = Vec2::ZERO;
            MOUSE_WHEEL_DELTA = Vec2::ZERO;

            RENDERS.clear();

            if CANVAS_UPDATE {
                CANVAS_UPDATE = false;
                upd_proj();
            }

            FPS_BUFFER += 1;

            if LAST_FPS_TIME <= LAST_FRAME_TIME {
                FPS = FPS_BUFFER;
                FPS_BUFFER = 0;
                LAST_FPS_TIME = LAST_FRAME_TIME + 1.;
            }

            if let Some(node) = &mut NODE2D {
                node.global_position = -CAMERA;
                node.draw(1.);
            }
        }
    }

    /*pub(crate) fn key(&mut self, key: &Key, keymod: KeyMods, touch: &Touch) {
        unsafe {
            if let Some(node) = &mut NODE2D {
                node.key(&key, keymod, touch);
            }
        }
    }*/

    pub(crate) fn touch(&mut self, id: u64, touch: &Touch, pos: Vec2) {
        unsafe {
            if let Some(node) = &mut NODE2D {
                ON_TOUCH = true;
                node.touch(id, touch, pos);
            }
        }
    }

    pub fn node2d(self, node: Node2d) -> Self {
        unsafe {
            NODE2D = Some(node);

            if let Some(node) = &mut NODE2D {
                node.start();
            }
        }
        self
    }

    /*pub fn script<T: Module + 'static>(self, script: &'static T) -> Self {
        unsafe {
            SCRIPT = Some(script);
        }
        self
    }*/

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

    pub fn backgraund(self, color: Rgba) -> Self {
        unsafe {
            BACKGRAUND = color;
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
        CANVAS_UPDATE = true;
    }
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
pub(crate) fn get_add_buffer() -> bool {
    unsafe {
        if UPD_RENDER_BUFFER {
            UPD_RENDER_BUFFER = false;
            true
        } else {
            false
        }
    }
}

#[inline(always)]
pub fn set_camera(x: f32, y: f32) {
    unsafe {
        CAMERA = vec2(x, y);
        CANVAS_UPDATE = true;
    }
}
#[inline(always)]
pub fn get_camera() -> Vec2 {
    unsafe {
        CAMERA
    }
}
#[inline(always)]
pub fn set_zoom(n: f32) {
    unsafe {
        ZOOM = n;
        CANVAS_UPDATE = true;
    }
}
