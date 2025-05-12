use super::{Keep, Touch};
use crate::render::{
    add_text,
    d2::{draw, CAMERA, CANVAS_PROJ, TOUCH, UPD_RENDER_BUFFER},
    rgb, Font, Rgba, Texture, DELTA,
};

use glam::{vec2, Vec2};
use std::{any::Any, collections::HashMap};

pub(crate) static mut ON_TOUCH: bool = false;

pub enum Obj2d {
    None,
    Rect(f32, f32, f32),
    Circle(f32),
    Texture(Texture),
    Text(String, f32, usize, Texture),
}

impl Obj2d {
    pub fn set_text(&mut self, new_text: &str) {
        if let Obj2d::Text(text, size, id, texture) = self {
            *text = new_text.to_string();

            let (tex_id, w, h) = add_text(text, *size, *id, Some(texture.id));

            texture.id = tex_id;
            texture.width = w;
            texture.height = h;
        } else {
            panic!("Not a Text object!")
        }
    }
}

pub struct Node2d {
    pub name: String,
    pub obj: Obj2d,
    pub(crate) parent_position: Vec2,
    pub(crate) global_position: Vec2,
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub color: Rgba,
    pub keep: Keep,
    pub offset: Vec2,
    visible: bool,
    pub(crate) node: Vec<Node2d>,
    pub script: Option<&'static dyn Module>,
    pub hash: HashMap<&'static str, Box<dyn Any + Send + Sync>>,
    touch_id: Option<u64>,
}

impl Node2d {
    pub fn new(name: &str, obj: Obj2d) -> Self {
        Self {
            name: name.to_string(),
            obj,
            parent_position: Vec2::new(0., 0.),
            global_position: Vec2::new(0., 0.),
            position: Vec2::new(0., 0.),
            rotation: 0.,
            scale: Vec2::new(1., 1.),
            color: rgb(234, 234, 234),
            visible: true,
            keep: Keep::Canvas,
            offset: Vec2::new(0., 0.),
            node: Vec::new(),
            script: None,
            hash: HashMap::new(),
            touch_id: None,
        }
    }

    pub fn node(mut self, node: Vec<Node2d>) -> Self {
        self.node.extend(node);
        self
    }

    pub fn hash<T: 'static + Send + Sync>(mut self, key: &'static str, value: T) -> Self {
        self.hash.insert(key, Box::new(value));
        self
    }

    pub fn script(mut self, script: &'static dyn Module) -> Self {
        self.script = Some(script);
        self
    }

    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.position = vec2(x, y);
        self
    }

    pub fn scale(mut self, x: f32, y: f32) -> Self {
        self.scale = vec2(x, y);
        self
    }

    pub fn rotation(mut self, r: f32) -> Self {
        self.rotation = r;
        self
    }

    pub fn color(mut self, color: Rgba) -> Self {
        self.color = color;
        self
    }

    pub fn visible(mut self, sel: bool) -> Self {
        self.visible = sel;
        self
    }

    pub fn keep(mut self, keep: Keep) -> Self {
        self.keep = keep;
        self
    }

    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.offset = vec2(x, y);
        self
    }

    pub fn get_parent_position(&self) -> Vec2 {
        self.parent_position
    }

    pub fn set_global_position(&mut self, x: f32, y: f32) {
        self.position = vec2(x, y) - self.parent_position;
    }

    pub fn get_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, sel: bool) {
        self.visible = sel;
        unsafe { UPD_RENDER_BUFFER = true }
    }

    #[inline(always)]
    pub fn get_global_position(&mut self) -> Vec2 {
        unsafe {
            match self.keep {
                Keep::Canvas => self.parent_position + self.position,
                Keep::Center => CAMERA + self.position,
                Keep::Up => CAMERA + self.position + vec2(0., -CANVAS_PROJ.y),
                Keep::Down => CAMERA + self.position + vec2(0., CANVAS_PROJ.y),
                Keep::Left => CAMERA + self.position + vec2(-CANVAS_PROJ.x, 0.),
                Keep::Right => CAMERA + self.position + vec2(CANVAS_PROJ.x, 0.),
                Keep::LeftUp => CAMERA + self.position - CANVAS_PROJ,
                Keep::LeftDown => CAMERA + self.position + vec2(-CANVAS_PROJ.x, CANVAS_PROJ.y),
                Keep::RightUp => CAMERA + self.position + vec2(CANVAS_PROJ.x, -CANVAS_PROJ.y),
                Keep::RightDown => CAMERA + self.position + CANVAS_PROJ,
            }
        }
    }

    pub fn get_node(&mut self, name: &str) -> Option<&mut Node2d> {
        let name = name.to_string();
        for obj in &mut self.node {
            if obj.name == name {
                return Some(obj);
            }
        }
        None
    }

    pub fn del_node(&mut self, name: &str) -> Result<Node2d, String> {
        let name = name.to_string();
        let mut del_id: Option<usize> = None;

        for (i, obj) in self.node.iter().enumerate() {
            if obj.name == name {
                del_id = Some(i);
                break;
            }
        }

        if let Some(id) = del_id {
            Ok(self.node.remove(id))
        } else {
            Err(format!("Not found object ({})", name))
        }
    }

    pub fn add_node(&mut self, node: Vec<Node2d>) {
        self.node.extend(node);
        unsafe { UPD_RENDER_BUFFER = true }
    }

    pub fn set_hash<T: 'static + Send + Sync>(&mut self, key: &'static str, value: T) {
        self.hash.insert(key, Box::new(value));
    }

    pub fn get_hash<T: 'static>(&self, key: &'static str) -> Option<&T> {
        self.hash.get(key)?.downcast_ref::<T>()
    }

    pub fn get_hash_mut<T: 'static>(&mut self, key: &'static str) -> Option<&mut T> {
        self.hash.get_mut(key)?.downcast_mut::<T>()
    }

    pub(crate) fn start(&mut self) {
        if let Some(s) = self.script {
            s.start(self);
        }

        self.global_position = self.get_global_position();

        for obj in &mut self.node {
            obj.parent_position = self.global_position;
            obj.start();
        }
    }

    pub(crate) fn update(&mut self) {
        if let Some(s) = self.script {
            s.update(self, unsafe { DELTA });
        }

        self.global_position = self.get_global_position();

        let parrent_pos = self.global_position
            + match &self.obj {
                Obj2d::Rect(w, h, _) => self.offset * vec2(*w, *h) * self.scale / 2.,
                Obj2d::Circle(r) => self.offset * r * self.scale / 2.,
                Obj2d::Texture(t) | Obj2d::Text(_, _, _, t) => {
                    self.offset * vec2(t.width, t.height) * self.scale / 2.
                }
                Obj2d::None => vec2(0., 0.),
            };

        for obj in &mut self.node {
            obj.parent_position = parrent_pos;
            obj.update();
        }
    }

    pub(crate) fn draw(&mut self, a: f32) {
        if self.visible {
            let mut color = self.color.get();
            color[3] *= a;

            draw(
                self.global_position,
                &self.obj,
                self.scale,
                self.rotation,
                self.offset,
                color,
            );

            for obj in &mut self.node {
                obj.draw(color[3]);
            }
        }
    }

    /*pub(crate) fn key(&mut self, key: &Key, keymod: KeyMods, touch: &Touch) {
        if let Some(s) = self.script {
            s.key(self, &key, keymod, touch);
        }

        for obj in &mut self.node {
            obj.key(&key, keymod, touch);
        }
    }*/

    pub(crate) fn touch(&mut self, id: u64, touch: &Touch, pos: Vec2) {
        for obj in &mut self.node.iter_mut().rev() {
            if unsafe { TOUCH } {
                obj.touch(id, touch, pos);
            } else {
                break;
            }
        }

        if unsafe { TOUCH } {
            if let Some(s) = self.script {
                if match touch {
                    Touch::Press => {
                        let dx = pos.x - self.global_position.x;
                        let dy = pos.y - self.global_position.y;

                        let sin = self.rotation.sin();
                        let cos = self.rotation.cos();

                        let local_x = cos * dx + sin * dy;
                        let local_y = -sin * dx + cos * dy;

                        if match &self.obj {
                            Obj2d::Rect(w, h, _) => {
                                let offset = self.offset * vec2(*w, *h) * self.scale;

                                ((local_x - offset.x).abs()) / self.scale.x + offset.x < w / 2.
                                    && ((local_y - offset.y).abs()) / self.scale.y + offset.y
                                        < h / 2.
                            }
                            Obj2d::Circle(r) => {
                                let offset = self.offset * r * self.scale;

                                println!("{}", (((local_x - offset.x) / self.scale.x).powi(2)
                                    + ((local_y - offset.y) / self.scale.y).powi(2))
                                .sqrt());

                                (((local_x - offset.x) / self.scale.x).powi(2)
                                    + ((local_y - offset.y) / self.scale.y).powi(2))
                                .sqrt()
                                    < *r
                            }
                            Obj2d::Texture(t) | Obj2d::Text(_, _, _, t) => {
                                let offset = self.offset * vec2(t.width, t.height) * self.scale;

                                ((local_x - offset.x).abs()) / self.scale.x < t.width / 2.
                                    && ((local_y - offset.y).abs()) / self.scale.y < t.height / 2.
                            }
                            Obj2d::None => true,
                        } {
                            self.touch_id = Some(id);
                            true
                        } else {
                            false
                        }
                    }
                    Touch::Relese => {
                        if self.touch_id == Some(id) {
                            self.touch_id = None;
                            true
                        } else {
                            false
                        }
                    }
                    Touch::Move => self.touch_id == Some(id),
                } {
                    unsafe {
                        TOUCH = false;
                    }
                    s.touch(self, id, touch, pos);
                }
            }
        }
    }
}

#[inline(always)]
pub fn circle(name: &str, r: f32) -> Node2d {
    Node2d::new(name, Obj2d::Circle(r))
}

#[inline(always)]
pub fn rect(name: &str, w: f32, h: f32, r: f32) -> Node2d {
    Node2d::new(name, Obj2d::Rect(w, h, r))
}

pub fn line(name: &str, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, r: f32) -> Node2d {
    let dx = x2 - x1;
    let dy = y2 - y1;

    let length = (dx * dx + dy * dy).sqrt();
    let angle = dy.atan2(dx); // угол между точками

    let center_x = (x1 + x2) / 2.0;
    let center_y = (y1 + y2) / 2.0;

    Node2d::new(name, Obj2d::Rect(length, thickness, r))
        .position(center_x, center_y)
        .rotation(angle)
}

#[inline(always)]
pub fn image(name: &str, texture: &Texture) -> Node2d {
    Node2d::new(
        name,
        Obj2d::Texture(Texture {
            id: texture.id,
            width: texture.width,
            height: texture.height,
        }),
    )
}

#[inline(always)]
pub fn text(name: &str, text: &str, size: f32, font: &Font) -> Node2d {
    let (id, w, h) = add_text(text, size, font.id, None);
    Node2d::new(
        name,
        Obj2d::Text(
            text.to_string(),
            size,
            font.id,
            Texture {
                id,
                width: w,
                height: h,
            },
        ),
    )
}

pub trait Module {
    fn start(&self, _obj: &mut Node2d) {}
    fn update(&self, _obj: &mut Node2d, _d: f32) {}
    //fn key(&self, _obj: &mut Node2d, _key: &Key, _keymod: KeyMods, _touch: &Touch) {}
    fn touch(&self, _obj: &mut Node2d, _id: u64, _touch: &Touch, _pos: Vec2) {
        unsafe {
            TOUCH = true;
        }
    }
}

#[macro_export]
macro_rules! node2d {
    ( $( $x:expr ),* $(,)? ) => {
        {
            let children = vec![$($x),*];
            $crate::object::d2::Node2d::new("", $crate::object::d2::Obj2d::None).node(children)
        }
    };
}
