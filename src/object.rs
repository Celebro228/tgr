use macroquad::{prelude::*, input::*};
use crate::{color::*, draw::*};

use std::collections::HashMap;

pub enum OT {
    Circle(f32),
    Rect(f32, f32),
    RoundedRect(f32, f32, f32),
    Text(String, Option<Font>, f32),
    Texture(Texture2D, f32, f32),
}

pub struct Obj {
    pub object_type: OT,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub color: Color,
    pub visible: bool,
    pub sx: f32,
    pub sy: f32,
    pub(crate) chache: [f32; 5],
    pub(crate) draw_update: bool,
}

impl Obj {
    pub fn new(t: OT, x: f32, y: f32) -> Self {
        Self {
            object_type: t,
            x: x,
            y: y,
            z: 0.,
            color: WHITE1,
            visible: true,
            sx: 0.,
            sy: 0.,
            chache: [0.; 5],
            draw_update: true,
        }
    }
}

pub struct D2 {
    pub objects: HashMap<String, Obj>,
    sorted_objects: Vec<String>,
    object_z: f32,
    touched: HashMap<u64, (String, Vec2, TouchPhase)>,
}

impl D2 {
    pub fn new() -> Self {
        simulate_mouse_with_touch(false);
        Self {
            objects: HashMap::new(),
            sorted_objects: Vec::new(),
            object_z: 0.,
            touched: HashMap::new(),
        }
    }

    pub fn update(&mut self) {
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
        for s in &self.sorted_objects {
            if let Some(obj) = self.objects.get_mut(s) {
                if screen_wb() || screen_hb() {
                    obj.draw_update = true;
                }
                if obj.visible {
                    match &obj.object_type {
                        OT::Circle(r) => {
                            if obj.draw_update {
                                obj.draw_update = false;
                                obj.chache[0] = wx(obj.x + (r * obj.sx));
                                obj.chache[1] = wy(obj.y + (r * obj.sy));
                                obj.chache[2] = ws(*r);
                            }
                            draw_circle(
                                obj.chache[0],
                                obj.chache[1],
                                obj.chache[2],
                                obj.color,
                            )
                        }
                        OT::Rect(w, h) => {
                            if obj.draw_update {
                                obj.draw_update = false;
                                obj.chache[0] = wx((obj.x + ((w / 2.) * obj.sx)) - (w / 2.));
                                obj.chache[1] = wy((obj.y + ((h / 2.) * obj.sy)) - (h / 2.));
                                obj.chache[2] = ws(*w);
                                obj.chache[3] = ws(*h);
                            }
                            draw_rectangle(
                                obj.chache[0],
                                obj.chache[1],
                                obj.chache[2],
                                obj.chache[3],
                                obj.color,
                            )
                        }
                        OT::RoundedRect(w, h, r) => {
                            if obj.draw_update {
                                obj.draw_update = false;
                                obj.chache[0] = wx((obj.x + ((w / 2.) * obj.sx)) - (w / 2.));
                                obj.chache[1] = wy((obj.y + ((h / 2.) * obj.sy)) - (h / 2.));
                                obj.chache[2] = ws(*w);
                                obj.chache[3] = ws(*h);
                                obj.chache[4] = ws(*r);
                            }
                            draw_rounded_rect(
                                obj.chache[0],
                                obj.chache[1],
                                obj.chache[2],
                                obj.chache[3],
                                obj.chache[4],
                                obj.color,
                            )
                        }
                        OT::Text(t, f, s) => {
                            if obj.draw_update {
                                obj.draw_update = false;
                                obj.chache[2] = ws(*s);
                                let c = get_text_center(&t, f.as_ref(), obj.chache[2] as u16, 1.0, 0.0);
                                obj.chache[0] = (wx(obj.x) + (c.x * obj.sx)) - c.x;
                                obj.chache[1] = (wy(obj.y) + (c.y * obj.sy)) - c.y;
                            }
                            draw_text_ex(
                                &t,
                                obj.chache[0],
                                obj.chache[1],
                                TextParams {
                                    font: f.as_ref(),
                                    font_size: obj.chache[2] as u16,
                                    color: obj.color,
                                    ..Default::default()
                                },
                            );
                        }
                        OT::Texture(t, w, h) => {
                            if obj.draw_update {
                                println!("update");
                                obj.draw_update = false;
                                obj.chache[0] = wx((obj.x + ((w / 2.) * obj.sx)) - (w / 2.));
                                obj.chache[1] = wy((obj.y + ((h / 2.) * obj.sy)) - (h / 2.));
                                obj.chache[2] = ws(*w);
                                obj.chache[3] = ws(*h);
                            }
                            draw_texture_ex(t, 
                                obj.chache[0],
                                obj.chache[1],
                                obj.color,
                                DrawTextureParams {
                                    dest_size: Some(Vec2 { x: obj.chache[2], y: obj.chache[3] }),
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

    pub fn add(&mut self, name: &str, obj: Obj) {
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
                        OT::Circle(r) => {
                            ((x - wx(obj.x + (r * obj.sx))).powi(2)
                                + (y - wy(obj.y + (r * obj.sy))).powi(2))
                            .sqrt()
                                < ws(*r)
                        }
                        OT::Rect(w, h) => {
                            (x - wx(obj.x + ((w / 2.) * obj.sx))).abs() < ws(w / 2.)
                                && (y - wy(obj.y + ((h / 2.) * obj.sy))).abs() < ws(h / 2.)
                        }
                        OT::RoundedRect(w, h, _) => {
                            (x - wx(obj.x + ((w / 2.) * obj.sx))).abs() < ws(w / 2.)
                                && (y - wy(obj.y + ((h / 2.) * obj.sy))).abs() < ws(h / 2.)
                        }
                        OT::Text(t, f, s) => {
                            let c = get_text_center(&t, f.as_ref(), ws(*s) as u16, 1.0, 0.0);
                            (x - (wx(obj.x) + (c.x * obj.sx))).abs() < c.x.abs()
                                && (y - (wy(obj.y) + (c.y * obj.sy))).abs() < c.y.abs()
                        }
                        OT::Texture(_, w, h) => {
                            (x - wx(obj.x + ((w / 2.) * obj.sx))).abs() < ws(w / 2.)
                                && (y - wy(obj.y + ((h / 2.) * obj.sy))).abs() < ws(h / 2.)
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
}