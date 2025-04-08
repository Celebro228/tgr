/*use inventory;

pub use tgr_macro::module;

inventory::collect!(&'static dyn Module);

pub fn start_all() {
    for module in inventory::iter::<&'static dyn Module> {
        module.start();
    }
}*/

pub use glam::{vec2, Vec2};

use crate::engine::{draw2d, get_delta};

pub trait Module: Sync {
    fn start(&self, obj: &mut Node2d) {}
    fn update(&self, obj: &mut Node2d, d: f64) {}
    fn input(&self) {}
}

pub enum Obj2d {
    None,
    Rect(f32, f32),
    Circle(f32),
}

pub struct Node2d {
    pub name: String,
    pub obj: Obj2d,
    pub(crate) parent_position: Vec2,
    pub(crate) global_position: Vec2,
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub visible: bool,
    pub node: Vec<Node2d>,
    pub script: Option<&'static dyn Module>,
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
            visible: true,
            node: Vec::new(),
            script: None,
        }
    }

    pub fn node(mut self, node: Vec<Node2d>) -> Self {
        self.node.extend(node);
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

    pub fn set_global_position(&mut self, x: f32, y: f32) {
        self.position = vec2(x, y) - self.parent_position;
    }

    pub fn get_global_position(&mut self) -> Vec2 {
        self.parent_position + self.position
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

    pub fn start(&mut self) {
        if let Some(s) = self.script {
            s.start(self);
        }

        self.global_position = self.parent_position + self.position;

        for obj in &mut self.node {
            obj.parent_position = self.global_position;
            obj.start();
        }
    }

    pub fn update(&mut self) {
        if let Some(s) = self.script {
            s.update(self, get_delta());
        }

        self.global_position = self.parent_position + self.position;

        for obj in &mut self.node {
            obj.parent_position = self.global_position;
            obj.update();
        }
    }

    pub(crate) fn draw(&mut self) {
        draw2d(self.global_position, &self.obj);

        for obj in &mut self.node {
            obj.draw();
        }
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

#[inline(always)]
pub fn circle(name: &str, r: f32) -> Node2d {
    Node2d::new(name, Obj2d::Circle(r))
}

#[inline(always)]
pub fn rect(name: &str, w: f32, h: f32) -> Node2d {
    Node2d::new(name, Obj2d::Rect(w, h))
}
