use std::collections::HashMap;

use crate::engine::{
    add_text, add_texture, draw2d, get_camera, get_canvas_proj, get_delta, get_font, get_touch,
    set_add_buffer, set_touch,
};

pub use glam::{vec2, Vec2};
use miniquad::{KeyCode, KeyMods};
use std::any::Any;

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

pub struct Font {
    pub(crate) id: usize,
}
pub struct Texture {
    pub(crate) id: usize,
    pub width: f32,
    pub height: f32,
}

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

pub enum Keep {
    Canvas,
    Center,
    Up,
    Down,
    Left,
    Right,
    LeftUp,
    LeftDown,
    RightUp,
    RightDown,
}

pub enum Touch {
    Press,
    Relese,
    Move,
}

pub enum Key {
    Char(char),
    Code(KeyCode),
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
    pub visible: bool,
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

    pub fn get_parent_position(&mut self) -> Vec2 {
        self.parent_position
    }

    pub fn set_global_position(&mut self, x: f32, y: f32) {
        self.position = vec2(x, y) - self.parent_position;
    }

    #[inline(always)]
    pub fn get_global_position(&mut self) -> Vec2 {
        match self.keep {
            Keep::Canvas => self.parent_position + self.position,
            Keep::Center => get_camera() + self.position,
            Keep::Up => get_camera() + self.position + vec2(0., -get_canvas_proj().y),
            Keep::Down => get_camera() + self.position + vec2(0., get_canvas_proj().y),
            Keep::Left => get_camera() + self.position + vec2(-get_canvas_proj().x, 0.),
            Keep::Right => get_camera() + self.position + vec2(get_canvas_proj().x, 0.),
            Keep::LeftUp => {
                get_camera() + self.position + vec2(-get_canvas_proj().x, -get_canvas_proj().y)
            }
            Keep::LeftDown => {
                get_camera() + self.position + vec2(-get_canvas_proj().x, get_canvas_proj().y)
            }
            Keep::RightUp => {
                get_camera() + self.position + vec2(get_canvas_proj().x, -get_canvas_proj().y)
            }
            Keep::RightDown => {
                get_camera() + self.position + vec2(get_canvas_proj().x, get_canvas_proj().y)
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

    pub fn add_node(&mut self, node: Vec<Node2d>) {
        self.node.extend(node);
        set_add_buffer();
    }

    pub fn set_hash<T: 'static + Send + Sync>(&mut self, key: &'static str, value: T) {
        self.hash.insert(key, Box::new(value));
    }

    pub fn get_hash<T: 'static>(&self, key: &'static str) -> Option<&T> {
        self.hash.get(key)?.downcast_ref::<T>()
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
            s.update(self, get_delta());
        }

        self.global_position = self.get_global_position();

        for obj in &mut self.node {
            obj.parent_position = self.global_position;
            obj.update();
        }
    }

    pub(crate) fn draw(&mut self, a: f32) {
        let mut color = self.color.get();
        color[3] *= a;

        if self.visible {
            draw2d(
                self.global_position,
                &self.obj,
                self.scale,
                self.rotation,
                color,
            );
        }

        for obj in &mut self.node {
            obj.draw(color[3]);
        }
    }

    pub(crate) fn key(&mut self, key: &Key, keymod: KeyMods, touch: &Touch) {
        if let Some(s) = self.script {
            s.key(self, &key, keymod, touch);
        }

        for obj in &mut self.node {
            obj.key(&key, keymod, touch);
        }
    }

    pub(crate) fn touch(&mut self, id: u64, touch: &Touch, pos: Vec2) {
        for obj in &mut self.node.iter_mut().rev() {
            if get_touch() {
                obj.touch(id, touch, pos);
            } else {
                break;
            }
        }

        if get_touch() {
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
                                (local_x.abs()) / self.scale.x < w / 2.
                                    && (local_y.abs()) / self.scale.y < h / 2.
                            }
                            Obj2d::Circle(r) => {
                                ((local_x / self.scale.x).powi(2)
                                    + (local_y / self.scale.y).powi(2))
                                .sqrt()
                                    < *r
                            }
                            Obj2d::Texture(t) | Obj2d::Text(_, _, _, t) => {
                                (local_x.abs()) / self.scale.x < t.width / 2.
                                    && (local_y.abs()) / self.scale.y < t.height / 2.
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
                    set_touch(false);
                    s.touch(self, id, touch, pos);
                }
            }
        }
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

#[inline(always)]
pub fn font(path: &str) -> Font {
    let id = get_font(path);
    Font { id }
}

#[inline(always)]
pub fn texture(path: &str) -> Texture {
    let (id, w, h) = add_texture(path);
    Texture {
        id,
        width: w,
        height: h,
    }
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

#[inline(always)]
pub fn circle(name: &str, r: f32) -> Node2d {
    Node2d::new(name, Obj2d::Circle(r))
}

#[inline(always)]
pub fn rect(name: &str, w: f32, h: f32, r: f32) -> Node2d {
    Node2d::new(name, Obj2d::Rect(w, h, r))
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

pub trait Module {
    fn start(&self, _obj: &mut Node2d) {}
    fn update(&self, _obj: &mut Node2d, _d: f32) {}
    fn key(&self, _obj: &mut Node2d, _key: &Key, _keymod: KeyMods, _touch: &Touch) {}
    fn touch(&self, _obj: &mut Node2d, _id: u64, _touch: &Touch, _pos: Vec2) {
        set_touch(true);
    }
}

#[macro_export]
macro_rules! node2d {
    ( $( $x:expr ),* $(,)? ) => {
        {
            let children = vec![$($x),*];
            $crate::object::Node2d::new("", $crate::object::Obj2d::None).node(children)
        }
    };
}

/*use inventory;

pub use tgr_macro::module;

inventory::collect!(&'static dyn Module);

pub fn start_all() {
    for module in inventory::iter::<&'static dyn Module> {
        module.start();
    }
}*/
