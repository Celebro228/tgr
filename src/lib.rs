use macroquad::{input::*, miniquad::TextureParams, prelude::*};
use std::collections::HashMap;

pub const WHITE1: Color = color_u8!(236, 236, 236, 255);
pub const WHITE2: Color = color_u8!(204, 204, 204, 255);
pub const BLACK1: Color = color_u8!(24, 24, 24, 255);
pub const BLACK2: Color = color_u8!(31, 31, 31, 255);

pub struct D2 {
    w: f32,
    h: f32,
    pub background: Color,
    screen_w: f32,
    screen_h: f32,
    pub objects: HashMap<String, GObject>,
    sorted_objects: Vec<String>,
    object_z: f32,
    touched: HashMap<u64, (String, Vec2, TouchPhase)>,
}

impl D2 {
    pub fn new(w: f32, h: f32) -> Self {
        simulate_mouse_with_touch(false);
        Self {
            w: w,
            h: h,
            background: BLACK1,
            screen_w: screen_width(),
            screen_h: screen_height(),
            objects: HashMap::new(),
            sorted_objects: Vec::new(),
            object_z: 0.,
            touched: HashMap::new(),
        }
    }

    pub fn update(&mut self) {
        self.screen_w = screen_width();
        self.screen_h = screen_height();

        self.touched
            .retain(|_, (_, _, phase)| *phase != TouchPhase::Ended);

        for touch in touches() {
            match touch.phase {
                TouchPhase::Started => {
                    self.in_touch(touch.id, touch.position.x, touch.position.y);
                }
                TouchPhase::Moved | TouchPhase::Stationary => {
                    if let Some(s) = self.touched.get_mut(&touch.id) {
                        s.1 = touch.position;
                        s.2 = touch.phase;
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if let Some(s) = self.touched.get_mut(&touch.id) {
                        s.2 = TouchPhase::Ended;
                    }
                }
            }
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            self.in_touch(0, mouse_position().0, mouse_position().1);
        } else if is_mouse_button_released(MouseButton::Left) {
            if let Some(s) = self.touched.get_mut(&0) {
                s.2 = TouchPhase::Ended;
            }
        } else if is_mouse_button_down(MouseButton::Left) {
            if let Some(s) = self.touched.get_mut(&0) {
                s.1 = vec2(mouse_position().0, mouse_position().1);
                s.2 = TouchPhase::Moved;
            }
        }
    }

    pub fn draw(&mut self) {
        clear_background(self.background);

        let t = get_time();

        for s in &self.sorted_objects {
            if let Some(obj) = self.objects.get_mut(s) {
                if obj.anim {
                    if t < obj.anim_time_start + obj.anim_time_end {
                        obj.x = obj.anim_start.x
                            + (obj.anim_end.x
                                * curve(
                                    ((t - obj.anim_time_start) / obj.anim_time_end) as f32,
                                    obj.anim_curve,
                                ));
                        obj.y = obj.anim_start.y
                            + (obj.anim_end.y
                                * curve(
                                    ((t - obj.anim_time_start) / obj.anim_time_end) as f32,
                                    obj.anim_curve,
                                ));
                    } else {
                        println!(
                            "{} {}",
                            obj.anim_start.x + obj.anim_end.x,
                            obj.anim_start.y + obj.anim_end.y
                        );
                        obj.anim = false;
                        obj.x = obj.anim_start.x + obj.anim_end.x;
                        obj.y = obj.anim_start.y + obj.anim_end.y;
                    }
                }
            }
            if let Some(obj) = self.objects.get(s) {
                if obj.visible {
                    match &obj.object_type {
                        GOType::Circle(r) => draw_circle(
                            self.x(obj.x + (r * obj.sx)),
                            self.y(obj.y + (r * obj.sy)),
                            self.s(*r),
                            obj.color,
                        ),
                        GOType::Rect(w, h) => draw_rectangle(
                            self.x((obj.x + ((w / 2.) * obj.sx)) - (w / 2.)),
                            self.y((obj.y + ((h / 2.) * obj.sy)) - (h / 2.)),
                            self.s(*w),
                            self.s(*h),
                            obj.color,
                        ),
                        GOType::RoundedRect(w, h, r) => draw_rounded_rect(
                            self.x((obj.x + ((w / 2.) * obj.sx)) - (w / 2.)),
                            self.y((obj.y + ((h / 2.) * obj.sy)) - (h / 2.)),
                            self.s(*w),
                            self.s(*h),
                            self.s(*r),
                            obj.color,
                        ),
                        GOType::Text(t, f, s) => {
                            let c = get_text_center(&t, f.as_ref(), self.s(*s) as u16, 1.0, 0.0);
                            draw_text_ex(
                                &t,
                                (self.x(obj.x) + (c.x * obj.sx)) - c.x,
                                (self.y(obj.y) + (c.y * obj.sy)) - c.y,
                                TextParams {
                                    font: f.as_ref(),
                                    font_size: self.s(*s) as u16,
                                    color: obj.color,
                                    ..Default::default()
                                },
                            );
                        }
                        GOType::Texture(t, w, h) => {
                            draw_texture_ex(t, 
                                self.x((obj.x + ((w / 2.) * obj.sx)) - (w / 2.)),
                                self.y((obj.y + ((h / 2.) * obj.sy)) - (h / 2.)),
                                obj.color,
                                DrawTextureParams {
                                    dest_size: Some(Vec2 { x: self.s(*w), y: self.s(*h) }),
                                    ..Default::default()
                                });
                        }
                    }
                }
            }
        }
    }

    pub fn is_touch_pressed(&self, name: &str) -> Option<Vec2> {
        for (_, (obj_name, position, phase)) in &self.touched {
            if obj_name == name && *phase == TouchPhase::Started {
                println!("preess {}", obj_name);
                return Some(*position);
            }
        }
        None
    }

    pub fn is_touch_released(&mut self, name: &str) -> bool {
        for (_, (obj_name, _, phase)) in &self.touched {
            if obj_name == name && *phase == TouchPhase::Ended {
                return true;
            }
        }
        false
    }

    pub fn is_touch_down(&self, name: &str) -> Option<Vec2> {
        for (_, (obj_name, position, phase)) in &self.touched {
            if obj_name == name && (*phase == TouchPhase::Moved || *phase == TouchPhase::Stationary)
            {
                return Some(*position);
            }
        }
        None
    }

    pub fn anim(&mut self, name: &str, curve: f32, x: f32, y: f32, time: f64) {
        if let Some(obj) = self.objects.get_mut(name) {
            if obj.x != x || obj.y != y {
                obj.anim = true;
                obj.anim_curve = curve;
                obj.anim_start = vec2(obj.x, obj.y);
                obj.anim_end = vec2(x - obj.anim_start.x, y - obj.anim_start.y);
                obj.anim_time_start = get_time();
                obj.anim_time_end = time;
            }
        }
    }

    pub fn add(&mut self, name: &str, obj: GObject) {
        self.objects.insert(name.to_string(), obj);
        self.sorted_objects.push(name.to_string());

        self.go_sort();

        if let Some(objs) = self.objects.get_mut(name) {
            objs.z = self.object_z;
            self.object_z += 0.001;
        }
    }

    pub fn del(&mut self, name: &str) {
        self.objects.remove(name);
        self.sorted_objects.retain(|obj_name| obj_name != name);
    }

    fn in_touch(&mut self, touch_id: u64, x: f32, y: f32) {
        for s in self.sorted_objects.iter().rev() {
            if let Some(obj) = self.objects.get(s) {
                if obj.visible {
                    if match &obj.object_type {
                        GOType::Circle(r) => {
                            ((x - self.x(obj.x + (r * obj.sx))).powi(2)
                                + (y - self.y(obj.y + (r * obj.sy))).powi(2))
                            .sqrt()
                                < self.s(*r)
                        }
                        GOType::Rect(w, h) => {
                            (x - self.x(obj.x + ((w / 2.) * obj.sx))).abs() < self.s(w / 2.)
                                && (y - self.y(obj.y + ((h / 2.) * obj.sy))).abs() < self.s(h / 2.)
                        }
                        GOType::RoundedRect(w, h, _) => {
                            (x - self.x(obj.x + ((w / 2.) * obj.sx))).abs() < self.s(w / 2.)
                                && (y - self.y(obj.y + ((h / 2.) * obj.sy))).abs() < self.s(h / 2.)
                        }
                        GOType::Text(t, f, s) => {
                            let c = get_text_center(&t, f.as_ref(), self.s(*s) as u16, 1.0, 0.0);
                            (x - (self.x(obj.x) + (c.x * obj.sx))).abs() < c.x.abs()
                                && (y - (self.y(obj.y) + (c.y * obj.sy))).abs() < c.y.abs()
                        }
                        GOType::Texture(t, w, h) => {
                            (x - self.x(obj.x + ((w / 2.) * obj.sx))).abs() < self.s(w / 2.)
                                && (y - self.y(obj.y + ((h / 2.) * obj.sy))).abs() < self.s(h / 2.)
                        }
                    } {
                        self.touched
                            .insert(touch_id, (s.to_string(), vec2(x, y), TouchPhase::Started));
                        break;
                    }
                }
            }
        }
    }

    fn go_sort(&mut self) {
        self.sorted_objects.sort_by(|a, b| {
            let obj_a = self.objects.get(a).unwrap(); // Извлекаем объект по имени
            let obj_b = self.objects.get(b).unwrap(); // Извлекаем объект по имени
            obj_a
                .z
                .partial_cmp(&obj_b.z)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    fn x(&self, x: f32) -> f32 {
        (self.screen_w / 2.)
            + ((x / (self.w / 2.)) * (self.screen_w.min((self.w / self.h) * self.screen_h) / 2.))
    }

    fn y(&self, y: f32) -> f32 {
        (self.screen_h / 2.)
            + ((y / (self.h / 2.)) * (self.screen_h.min((self.h / self.w) * self.screen_w) / 2.))
    }

    fn s(&self, s: f32) -> f32 {
        (s / self.w) * self.screen_w.min((self.w / self.h) * self.screen_h)
    }
}

pub struct GObject {
    pub object_type: GOType,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub color: Color,
    pub visible: bool,
    pub sx: f32,
    pub sy: f32,
    pub anim: bool,
    pub anim_curve: f32,
    pub anim_start: Vec2,
    pub anim_end: Vec2,
    pub anim_time_start: f64,
    pub anim_time_end: f64,
}

impl GObject {
    pub fn new(gt: GOType, x: f32, y: f32) -> Self {
        Self {
            object_type: gt,
            x: x,
            y: y,
            z: 0.,
            color: WHITE1,
            visible: true,
            sx: 0.,
            sy: 0.,
            anim: false,
            anim_curve: 0.,
            anim_start: Vec2 { x: 0., y: 0. },
            anim_end: Vec2 { x: 0., y: 0. },
            anim_time_start: 0.,
            anim_time_end: 0.,
        }
    }
}

pub enum GOType {
    Circle(f32),
    Rect(f32, f32),
    RoundedRect(f32, f32, f32),
    Text(String, Option<Font>, f32),
    Texture(Texture2D, f32, f32),
}

#[inline(always)]
fn curve(t: f32, tension: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;

    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;

    // Начальная и конечная скорость контролируются tension
    let m0 = tension;
    let m1 = tension;

    h00 + h10 * m0 + h01 + h11 * m1
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

    draw_circle(x, y, r/2., color);
    draw_circle(x2, y2, r/2., color);
}
