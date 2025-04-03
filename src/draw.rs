use macroquad::{
    prelude::{
        Color,
        screen_width,
        screen_height,
        get_frame_time,
        next_frame,
        clear_background,
    },
    shapes::*
};
use crate::color::*;

static mut WINDOW_W: f32 = 1280.;
static mut WINDOW_H: f32 = 720.;
static mut BACKGROUND: Color = BLACK1;

static mut SCREEN_W: f32 = 0.;
static mut SWB: bool = false;
static mut SCREEN_H: f32 = 0.;
static mut SHB: bool = false;
static mut DELTA: f32 = 0.;

static mut TOUCHS: [i32; 2] = [0, 2];

// Взять что-то
#[inline(always)]
pub fn window_w() -> f32 {
    unsafe { WINDOW_W }
}
#[inline(always)]
pub fn window_h() -> f32 {
    unsafe { WINDOW_H }
}
#[inline(always)]
pub fn background() -> Color {
    unsafe { BACKGROUND }
}
#[inline(always)]
pub fn screen_w() -> f32 {
    unsafe { SCREEN_W }
}
#[inline(always)]
pub(crate) fn screen_wb() -> bool {
    unsafe { SWB }
}
#[inline(always)]
pub fn screen_h() -> f32 {
    unsafe { SCREEN_H }
}
#[inline(always)]
pub(crate) fn screen_hb() -> bool {
    unsafe { SHB }
}
#[inline(always)]
pub fn delta() -> f32 {
    unsafe { DELTA }
}
#[inline(always)]
pub(crate) fn wx(x: f32) -> f32 {
    (screen_w() / 2.) + ((x / window_w()) * screen_w().min((window_w() / window_h()) * screen_h()))
}
#[inline(always)]
pub(crate) fn wy(y: f32) -> f32 {
    (screen_h() / 2.) + ((y / window_h()) * screen_h().min((window_h() / window_w()) * screen_w()))
}
#[inline(always)]
pub(crate) fn ws(s: f32) -> f32 {
    (s / window_w()) * screen_w().min((window_w() / window_h()) * screen_h())
}

// Сохранить что-то
pub fn set_window(w: f32, h: f32) {
    unsafe {
        WINDOW_W = w;
        WINDOW_H = h;
    }
}
#[inline(always)]
pub fn set_background(color: Color) {
    unsafe { BACKGROUND = color }
}

// Каждый кадр
pub async fn wait() {
    // Конец кадра

    next_frame().await;

    // Ноый кадр
    unsafe {
        DELTA = get_frame_time();

        let screen_w_now = screen_width();
        let screen_h_now = screen_height();

        if SCREEN_W != screen_w_now {
            SCREEN_W = screen_w_now;
            SWB = true;
        } else {
            SWB = false;
        }

        if SCREEN_H != screen_h_now {
            SCREEN_H = screen_h_now;
            SHB = true;
        } else {
            SHB = false;
        }
    }

    clear_background(background());
}

pub fn draw_rounded_rect(x: f32, y: f32, w: f32, h: f32, r: f32, color: Color) {
    draw_rectangle(x + r, y, w - 2.0 * r, h, color);
    draw_rectangle(x, y + r, w, h - 2.0 * r, color);

    draw_circle(x + r, y + r, r, color);
    draw_circle(x + w - r, y + r, r, color);
    draw_circle(x + r, y + h - r, r, color);
    draw_circle(x + w - r, y + h - r, r, color);
}

pub fn draw_rounded_line(x: f32, y: f32, x2: f32, y2: f32, r: f32, color: Color) {
    draw_line(x, y, x2, y2, r, color);

    draw_circle(x, y, r / 2., color);
    draw_circle(x2, y2, r / 2., color);
}
