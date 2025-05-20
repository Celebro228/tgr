use super::{Keep, Touch};
use crate::{prelude::new_render, render::{
    add_text,
    d2::{draw, CAMERA2D, CANVAS_PROJ, CANVAS_UPDATE, RENDERS},
    rgb, Font, Rgba, Texture, DELTA,
}};

use glam::{vec2, Vec2};
use std::{any::Any, collections::HashMap};

pub(crate) static mut ON_TOUCH: bool = false;

#[derive(Clone, PartialEq)]
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

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum DrawUpdate {
    None,
    Update,
    Create,
}

struct Chache {
    offset: Vec2,
}

struct Hidden {
    obj: Obj2d,
    position: Vec2,
    scale: Vec2,
    offset: Vec2,
    visible: bool,
}

pub struct Node2d {
    pub name: String,
    pub obj: Obj2d,
    pub parent_position: Vec2,
    pub global_position: Vec2,
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub color: Rgba,
    pub keep: Keep,
    pub offset: Vec2,
    pub visible: bool,
    pub node: Vec<Node2d>,
    pub script: Option<&'static dyn Module>,
    pub hash: HashMap<&'static str, Box<dyn Any + Send + Sync>>,
    touch_id: Option<u64>,
    render_id: usize,
    draw_update: DrawUpdate,
    chache: Chache,
    hidden: Hidden,
}
impl Node2d {

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

    pub fn add_node(&mut self, node: Vec<CreateNode2d>) {
        let mut node: Vec<Node2d> = node.into_iter().map(|n| {
            let mut n = n.get_node();
            if n.visible {
                n.render_id = new_render();
            }
            n
        }).collect();

        self.node.extend(node);
        
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

        for obj in &mut self.node {
            obj.start();
        }
    }

    pub(crate) fn update(&mut self) {
        self.upd_pos();

        if let Some(s) = self.script {
            s.update(self, unsafe { DELTA });
        }

        self.upd_pos();

        if self.obj != Obj2d::None && (self.obj != self.hidden.obj
            || self.offset != self.hidden.offset
            || self.scale != self.hidden.scale)
        {
            self.hidden.obj = self.obj.clone();
            self.hidden.offset = self.offset;
            self.hidden.scale = self.scale;

            self.chache.offset = match &self.obj {
                Obj2d::Rect(w, h, _) => self.offset * vec2(*w, *h) * self.scale,
                Obj2d::Circle(r) => self.offset * r * self.scale,
                Obj2d::Texture(t) | Obj2d::Text(_, _, _, t) => {
                    self.offset * vec2(t.width, t.height) * self.scale
                }
                Obj2d::None => vec2(0., 0.),
            };

            self.draw_update = DrawUpdate::Update;
            self.upd_img();
        }

        let parrent_pos = self.global_position + self.chache.offset / 2.;

        for obj in &mut self.node {
            obj.parent_position = parrent_pos;
            obj.update();
        }
    }

    pub(crate) fn draw(&mut self, a: f32) {
        if self.visible != self.hidden.visible && self.obj != Obj2d::None {
            self.hidden.visible = self.visible;
            self.upd_img();
        }

        if self.visible {
            let mut color = self.color.get();
            color[3] *= a;

            if self.draw_update != DrawUpdate::None {
                draw(
                    self.render_id,
                    self.global_position,
                    &self.obj,
                    self.scale,
                    self.rotation,
                    self.offset,
                    color,
                );
            }

            unsafe {
                if RENDERS[self.render_id].3 != DrawUpdate::Create {
                    RENDERS[self.render_id].3 = self.draw_update;
                }
            }

            self.draw_update = DrawUpdate::None;

            for obj in &mut self.node {
                obj.draw(color[3]);
            }
        }
    }

    #[inline(always)]
    fn upd_pos(&mut self) {
        if self.global_position != self.hidden.position {
            self.position = self.global_position - self.parent_position;
            self.draw_update = DrawUpdate::Update;
        }

        self.global_position = unsafe {
            match self.keep {
                Keep::Canvas => self.parent_position,
                Keep::Center => CAMERA2D,
                Keep::Up => CAMERA2D + vec2(0., -CANVAS_PROJ.y),
                Keep::Down => CAMERA2D + vec2(0., CANVAS_PROJ.y),
                Keep::Left => CAMERA2D + vec2(-CANVAS_PROJ.x, 0.),
                Keep::Right => CAMERA2D + vec2(CANVAS_PROJ.x, 0.),
                Keep::LeftUp => CAMERA2D - CANVAS_PROJ,
                Keep::LeftDown => CAMERA2D + vec2(-CANVAS_PROJ.x, CANVAS_PROJ.y),
                Keep::RightUp => CAMERA2D + vec2(CANVAS_PROJ.x, -CANVAS_PROJ.y),
                Keep::RightDown => CAMERA2D + CANVAS_PROJ,
            } } + self.position;
        
        self.hidden.position = self.global_position;
    }

    #[inline(always)]
    fn upd_img(&mut self) {
        if self.visible {
            let c = match &self.obj {
                Obj2d::Rect(_, _, _) => None,
                Obj2d::Circle(_) => None,
                Obj2d::Texture(t) | Obj2d::Text(_, _, _, t) => Some(t.id),
                Obj2d::None => None,
            };

            unsafe {
                if c != RENDERS[self.render_id].2 {
                    RENDERS[self.render_id].2 = c;
                }
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
            if unsafe { ON_TOUCH } {
                obj.touch(id, touch, pos);
            } else {
                break;
            }
        }

        if unsafe { ON_TOUCH } {
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
                                ((local_x - self.chache.offset.x).abs()) / self.scale.x
                                    + self.chache.offset.x
                                    < w / 2.
                                    && ((local_y - self.chache.offset.y).abs()) / self.scale.y
                                        + self.chache.offset.y
                                        < h / 2.
                            }
                            Obj2d::Circle(r) => {
                                (((local_x - self.chache.offset.x) / self.scale.x).powi(2)
                                    + ((local_y - self.chache.offset.y) / self.scale.y).powi(2))
                                .sqrt()
                                    < *r
                            }
                            Obj2d::Texture(t) | Obj2d::Text(_, _, _, t) => {
                                ((local_x - self.chache.offset.x).abs()) / self.scale.x
                                    < t.width / 2.
                                    && ((local_y - self.chache.offset.y).abs()) / self.scale.y
                                        < t.height / 2.
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
                        ON_TOUCH = false;
                    }
                    s.touch(self, id, touch, pos);
                }
            }
        }
    }
}

pub struct CreateNode2d {
    pub node2d: Node2d,
}

impl CreateNode2d {
    pub fn new(name: &str, obj: Obj2d) -> Self {
        Self {
            node2d: Node2d {
                name: name.to_string(),
                obj: obj.clone(),
                parent_position: Vec2::ZERO,
                global_position: Vec2::ZERO,
                position: Vec2::ZERO,
                rotation: 0.,
                scale: Vec2::new(1., 1.),
                color: rgb(234, 234, 234),
                visible: true,
                keep: Keep::Canvas,
                offset: Vec2::ZERO,
                node: Vec::new(),
                script: None,
                hash: HashMap::new(),
                touch_id: None,
                render_id: 0,
                draw_update: DrawUpdate::Create,
                chache: Chache {
                    offset: Vec2::ZERO,
                },
                hidden: Hidden {
                    obj,
                    position: Vec2::ZERO,
                    scale: Vec2::ZERO,
                    offset: Vec2::ZERO,
                    visible: true,
                },
            },
        }
    }

    pub fn node(mut self, node: Vec<CreateNode2d>) -> Self {
        self.node2d.add_node(node);
        self
    }

    pub fn hash<T: 'static + Send + Sync>(mut self, key: &'static str, value: T) -> Self {
        self.node2d.hash.insert(key, Box::new(value));
        self
    }

    pub fn script(mut self, script: &'static dyn Module) -> Self {
        self.node2d.script = Some(script);
        self
    }

    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.node2d.position = vec2(x, y);
        self
    }

    pub fn scale(mut self, x: f32, y: f32) -> Self {
        self.node2d.scale = vec2(x, y);
        self
    }

    pub fn rotation(mut self, r: f32) -> Self {
        self.node2d.rotation = r;
        self
    }

    pub fn color(mut self, color: Rgba) -> Self {
        self.node2d.color = color;
        self
    }

    pub fn visible(mut self, sel: bool) -> Self {
        self.node2d.visible = sel;
        self
    }

    pub fn keep(mut self, keep: Keep) -> Self {
        self.node2d.keep = keep;
        self
    }

    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.node2d.offset = vec2(x, y);
        self
    }

    pub fn get_node(self) -> Node2d {
        self.node2d
    }
}

#[inline(always)]
pub fn circle(name: &str, r: f32) -> CreateNode2d {
    CreateNode2d::new(name, Obj2d::Circle(r))
}

#[inline(always)]
pub fn rect(name: &str, w: f32, h: f32, r: f32) -> CreateNode2d {
    CreateNode2d::new(name, Obj2d::Rect(w, h, r))
}

pub fn line(
    name: &str,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    thickness: f32,
    r: f32,
) -> CreateNode2d {
    let dx = x2 - x1;
    let dy = y2 - y1;

    let length = (dx * dx + dy * dy).sqrt();
    let angle = dy.atan2(dx); // угол между точками

    let center_x = (x1 + x2) / 2.0;
    let center_y = (y1 + y2) / 2.0;

    CreateNode2d::new(name, Obj2d::Rect(length, thickness, r))
        .position(center_x, center_y)
        .rotation(angle)
}

#[inline(always)]
pub fn image(name: &str, texture: &Texture) -> CreateNode2d {
    CreateNode2d::new(
        name,
        Obj2d::Texture(Texture {
            id: texture.id,
            width: texture.width,
            height: texture.height,
        }),
    )
}

#[inline(always)]
pub fn text(name: &str, text: &str, size: f32, font: &Font) -> CreateNode2d {
    let (id, w, h) = add_text(text, size, font.id, None);
    CreateNode2d::new(
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
            ON_TOUCH = true;
        }
    }
}

#[macro_export]
macro_rules! node2d {
    ( $( $x:expr ),* $(,)? ) => {
        {
            let children = vec![$($x),*];
            $crate::object::d2::CreateNode2d::new("", $crate::object::d2::Obj2d::None).node(children)
        }
    };
}
