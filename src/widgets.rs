use crate::object::Touch::*;
use crate::prelude::{circle, rect, text, Font, Module, Node2d, Obj2d, Rgba, Touch}; // Key};

use glam::{vec2, Vec2};

struct Button;
impl Module for Button {
    fn touch(&self, obj: &mut Node2d, _id: u64, touch: &Touch, _pos: Vec2) {
        let mut color = match touch {
            Relese => {
                obj.set_hash("button", false);
                0.1
            }
            _ => {
                obj.set_hash("button", true);
                0.0
            }
        };

        obj.color.r = color;
        obj.color.g = color;
        obj.color.b = color;

        if let Some(text) = obj.get_node("text") {
            color = 0.9 + color;

            text.color.r = color;
            text.color.g = color;
            text.color.b = color;
        }
    }
}

pub fn button(name: &str, tex: &str, size: f32, font: &Font) -> Node2d {
    let tex = text("text", &tex, size, &font);
    let mut size = vec2(0., 0.);

    if let Obj2d::Text(_, _, _, ref t) = tex.obj {
        size = vec2(t.width, t.height)
    }

    rect(&name, size.x + size.y, size.y * 2., size.x.min(size.y) / 4.)
        .color(Rgba::new(0.1, 0.1, 0.1, 1.))
        .node(vec![tex])
        .script(&Button)
        .hash("button", false)
}

struct Check;
impl Module for Check {
    fn update(&self, obj: &mut Node2d, d: f32) {
        let data = *obj.get_hash::<bool>("check").unwrap();

        if let Some(button) = obj.get_node("button") {
            if let Obj2d::Circle(size) = button.obj {
                let check = if data { 1. } else { -1. };
                let point = check * size;

                if button.position.x != point {
                    button.position.x = (button.position.x + (point * d * 20.)).clamp(-size, size);
                    let color = 0.5 + button.position.x / size * 0.4;
                    obj.color.r = color;
                }
                //button.position.x
            }
        }
    }

    fn touch(&self, obj: &mut Node2d, _id: u64, touch: &Touch, pos: Vec2) {
        match touch {
            Press => {
                obj.set_hash("pos", pos);
            }
            Move => {}
            Relese => {
                if pos == *obj.get_hash::<Vec2>("pos").unwrap() {
                    let data = !obj.get_hash::<bool>("check").unwrap();
                    obj.set_hash("check", data);
                }
            }
        }
    }
}

pub fn check(name: &str, size: f32) -> Node2d {
    let size2 = size / 2.;

    rect(&name, size * 2. + size2, size + size2, size2 + size / 4.)
        .color(Rgba::new(0.1, 0.1, 0.1, 1.))
        .node(vec![circle("button", size / 2.)])
        .script(&Check)
        .hash("check", false) //.hash("posx", 0.)
}

/*struct EditText;
impl Module for EditText {
    fn touch(&self, obj: &mut Node2d, _id: u64, touch: &crate::engine::Touch, _pos: glam::Vec2) {
        match touch {
            Relese => {
                obj.set_hash("online", true);
            }
            _ => {
                obj.set_hash("online", false);
            }
        };
    }

    fn key(&self, obj: &mut Node2d, key: &Key, _keymod: miniquad::KeyMods, _touch: &Touch) {
        if *obj.get_hash::<bool>("online").unwrap() {
            if let Key::Char(code) = key {
                let text = if let Some(text) = obj.get_hash_mut::<String>("text") {
                    match code {
                        '\u{8}' => {
                            text.pop();
                        }
                        '\r' => {
                            text.push('\n')
                        }
                        c => {
                            text.push(*c);
                        },
                    }
                    text.clone()
                } else {
                    String::from("")
                };

                let node_obj = obj.get_node("text").unwrap();

                node_obj.obj.set_text(&text);

                if let Obj2d::Text(_, _, _, ref t) = node_obj.obj {
                    obj.obj = Obj2d::Rect(t.width + t.height, t.height * 2., t.width.min(t.height) / 4.);
                }
            }
        }
    }
}

pub fn edittext(name: &str, tex: &str, size: f32, font: &Font) -> Node2d {
    let node_text = text("text", &tex, size, &font);
    let mut size = vec2(0., 0.);

    if let Obj2d::Text(_, _, _, ref t) = node_text.obj {
        size = vec2(t.width, t.height)
    }

    rect(&name, size.x + size.y, size.y * 2., size.x.min(size.y) / 4.)
        .color(Rgba::new(0.1, 0.1, 0.1, 1.))
        .node(vec![node_text])
        .script(&EditText)
        .hash("text", String::from(tex))
        .hash("online", false)
}*/
