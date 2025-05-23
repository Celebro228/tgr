use crate::{
    object::{
        d2::{Node2d, ON_TOUCH},
        Touch,
    },
    prelude::CreateNode2d,
    render::{
        d2::{
            upd_proj, CANVAS, CANVAS_UPDATE, RENDERS,
        },
        Rgba, FPS, FPS_BUFFER, LAST_FPS_TIME, LAST_FRAME_TIME, WINDOW, WINDOW_UPDATE,
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
            node.draw(1.);
        }
    }
}

pub(crate) fn touch(id: u64, touch: &Touch, pos: Vec2) {
    unsafe {
        if let Some(node) = &mut NODE2D {
            ON_TOUCH = true;
            node.touch(id, touch, pos);
        }
    }
}

pub struct Engine;
impl Engine {
    pub fn start(&self, name: &str) {
        render(name);
    }

    /*pub(crate) fn key(&mut self, key: &Key, keymod: KeyMods, touch: &Touch) {
        unsafe {
            if let Some(node) = &mut NODE2D {
                node.key(&key, keymod, touch);
            }
        }
    }*/

    pub fn node2d(self, node: CreateNode2d) -> Self {
        unsafe {
            NODE2D = Some(node.get_node());
            //RENDERS.clear();

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
            WINDOW_UPDATE = true;
            CANVAS_UPDATE = true;
        }
        self
    }

    pub fn canvas(self, x: f32, y: f32) -> Self {
        unsafe {
            CANVAS = vec2(x, y);
            CANVAS_UPDATE = true;
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

    pub fn backgraund(self, color: Rgba) -> Self {
        unsafe {
            BACKGRAUND = color;
        }
        self
    }
}

