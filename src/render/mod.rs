pub mod d2;
pub mod d3;

#[cfg(feature = "miniquad")]
pub mod miniquad;

#[cfg(feature = "wgpu")]
pub mod wgpu;

use crate::data::load_file;

use glam::{Vec2, Vec3};
use image::{load_from_memory, DynamicImage, GenericImageView};
use rusttype::{point, Font as RFont, Scale};

pub(crate) static mut WINDOW: Vec2 = Vec2::new(1280., 720.);
pub(crate) static mut WINDOW_UPDATE: bool = false;

pub(super) static mut DELTA: f32 = 0.;
pub(super) static mut LAST_FRAME_TIME: f64 = 0.;

pub(super) static mut FPS: u16 = 60;
pub(super) static mut FPS_BUFFER: u16 = 0;
pub(super) static mut LAST_FPS_TIME: f64 = 0.;

#[repr(C)]
pub(crate) struct Vertex {
    pos: Vec3,
    color: [f32; 4],
    uv: Vec2,
}

pub enum View {
    KeepWidth,
    KeepHeight,
    Scale,
    Window,
}

#[derive(Debug, Clone, Copy)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn get(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

#[inline(always)]
pub fn rgb(r: u8, g: u8, b: u8) -> Rgba {
    Rgba {
        r: r as f32 / 255.,
        g: g as f32 / 255.,
        b: b as f32 / 255.,
        a: 1.,
    }
}
#[inline(always)]
pub fn rgba(r: u8, g: u8, b: u8, a: f32) -> Rgba {
    Rgba {
        r: r as f32 / 255.,
        g: g as f32 / 255.,
        b: b as f32 / 255.,
        a,
    }
}

pub fn hsv(h: f32, s: f32, v: f32) -> Rgba {
    let c = v * s;
    let h = h / 60.;
    let x = c * (1. - ((h % 2.) - 1.).abs());
    let m = v - s;

    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.),
        1 => (x, c, 0.),
        2 => (0., c, x),
        3 => (0., x, c),
        4 => (x, 0., c),
        5 => (c, 0., x),
        _ => (0., 0., 0.), // fallback, например если h = NaN
    };

    Rgba {
        r: r + m,
        g: g + m,
        b: b + m,
        a: 1.,
    }
}

static mut FONTS: Vec<RFont> = Vec::new();

pub struct Font {
    pub(crate) id: usize,
}

#[inline(always)]
pub fn font(path: &str) -> Font {
    let file = load_file(path).expect("Error to loading font");
    let file: &'static [u8] = Box::leak(file.into_boxed_slice());
    let font = RFont::try_from_bytes(&file).expect("Error font bytes");

    unsafe {
        FONTS.push(font);
        Font {
            id: FONTS.len() - 1,
        }
    }
}

pub(crate) static mut TEXUTRES_ID: usize = 0;
pub(crate) static mut TEXUTRES_BUFFER: Vec<(Vec<u8>, u16, u16)> = Vec::new();
pub(crate) static mut TEXUTRES_UPDATE: Vec<(usize, Vec<u8>, u16, u16)> = Vec::new();

pub struct Texture {
    pub(crate) id: usize,
    pub width: f32,
    pub height: f32,
}

#[inline(always)]
pub fn texture(path: &str) -> Texture {
    let img = load_from_memory(&load_file(path).expect("Error to loading texture"))
        .expect("Error to convert bytes from image");
    //let img = image::open(path).expect("Error to load texture");

    let rgba = img.to_rgba8().to_vec();
    let (width, height) = img.dimensions();

    let (id, w, h) = add_texture_buffer(rgba, width, height);

    Texture {
        id,
        width: w,
        height: h,
    }
}

#[inline(always)]
fn add_texture_buffer(rgba: Vec<u8>, width: u32, height: u32) -> (usize, f32, f32) {
    unsafe {
        TEXUTRES_BUFFER.push((rgba, width as u16, height as u16));
        TEXUTRES_ID += 1;
        (TEXUTRES_ID - 1, width as f32, height as f32)
    }
}
#[inline(always)]
fn upd_texture_buffer(id: usize, rgba: Vec<u8>, width: u32, height: u32) -> (usize, f32, f32) {
    unsafe {
        TEXUTRES_UPDATE.push((id, rgba, width as u16, height as u16));
        (id, width as f32, height as f32)
    }
}

pub(crate) fn add_text(
    text: &str,
    size: f32,
    font_id: usize,
    upd: Option<usize>,
) -> (usize, f32, f32) {
    let scale = Scale::uniform(size);

    let font = unsafe { &FONTS[font_id] };

    let v_metrics = font.v_metrics(scale);

    let glyphs: Vec<_> = font
        .layout(text, scale, point(20.0, 20.0 + v_metrics.ascent))
        .collect();

    let (min_x, max_x, min_y, max_y) = {
        //let first = glyphs.first().and_then(|g| g.pixel_bounding_box()).unwrap();
        //let last = glyphs.last().and_then(|g| g.pixel_bounding_box()).unwrap();

        //let (mut min_x, mut max_x) = (first.min.x, last.max.x);
        //let (mut min_y, mut max_y) = (first.min.y, last.max.y);

        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;

        for g in &glyphs {
            if let Some(bb) = g.pixel_bounding_box() {
                min_x = min_x.min(bb.min.x);
                max_x = max_x.max(bb.max.x);
                min_y = min_y.min(bb.min.y);
                max_y = max_y.max(bb.max.y);
            }
        }

        if min_x > max_x || min_y > max_y {
            (0, 1, 0, 1)
        } else {
            (min_x, max_x, min_y, max_y)
        }
    };

    let width = (max_x - min_x) as u32;
    let height = (max_y - min_y) as u32;

    let mut image = DynamicImage::new_rgba8(width, height).to_rgba8();

    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                if v != 0. {
                    image.put_pixel(
                        x + (bounding_box.min.x - min_x) as u32,
                        y + (bounding_box.min.y - min_y) as u32,
                        image::Rgba([255, 255, 255, (v * 255.) as u8]),
                    )
                }
            });
        }
    }

    if let Some(id) = upd {
        upd_texture_buffer(id, image.to_vec(), width, height)
    } else {
        add_texture_buffer(image.to_vec(), width, height)
    }
}
