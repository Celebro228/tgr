use crate::engine::{draw2d, get_delta, get_touch, set_add_buffer, set_touch};

pub use glam::{vec2, Vec2};

pub struct Rgba {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Rgba {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.,
            g: g as f32 / 255.,
            b: b as f32 / 255.,
            a: a as f32 / 255.,
        }
    }

    pub fn get(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

pub enum Touch {
    Down,
    Up,
    Move,
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
    pub color: Rgba,
    pub visible: bool,
    pub(crate) node: Vec<Node2d>,
    pub script: Option<&'static dyn Module>,
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
            color: rgb(255, 255, 255),
            visible: true,
            node: Vec::new(),
            script: None,
            touch_id: None,
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

    pub fn color(mut self, color: Rgba) -> Self {
        self.color = color;
        self
    }

    pub fn visible(mut self, sel: bool) -> Self {
        self.visible = sel;
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

    pub fn add_node(&mut self, node: Vec<Node2d>) {
        self.node.extend(node);
        set_add_buffer();
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
        if self.visible {
            draw2d(
                self.global_position,
                &self.obj,
                self.scale,
                self.color.get(),
            );
        }

        for obj in &mut self.node {
            obj.draw();
        }
    }

    pub(crate) fn touch(&mut self, id: u64, touch: &Touch, pos: Vec2) {
        for obj in &mut self.node {
            if get_touch() {
                obj.touch(id, touch, pos);
            } else {
                break;
            }
        }

        if get_touch() {
            if let Some(s) = self.script {
                if match touch {
                    Touch::Down => {
                        if match self.obj {
                            Obj2d::Rect(w, h) => {
                                ((pos.x - self.global_position.x).abs()) / self.scale.x
                                    < w / 2.
                                    && ((pos.y - self.global_position.y).abs()) / self.scale.y
                                        < h / 2.
                            }
                            Obj2d::Circle(r) => {
                                ((((pos.x - self.global_position.x).abs()) / self.scale.x).powi(2)
                                    + (((pos.y - self.global_position.y).abs()) / self.scale.y).powi(2))
                                        .sqrt() < r
                            }
                            Obj2d::None => {true}
                        } {
                            self.touch_id = Some(id);
                            true
                        } else {
                            false
                        }
                    }
                    Touch::Up => {
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
pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Rgba {
    Rgba {
        r: r as f32 / 255.,
        g: g as f32 / 255.,
        b: b as f32 / 255.,
        a: a as f32 / 255.,
    }
}

#[inline(always)]
pub fn circle(name: &str, r: f32) -> Node2d {
    Node2d::new(name, Obj2d::Circle(r))
}

#[inline(always)]
pub fn rect(name: &str, w: f32, h: f32) -> Node2d {
    Node2d::new(name, Obj2d::Rect(w, h))
}

pub trait Module: Sync {
    fn start(&self, obj: &mut Node2d) {}
    fn update(&self, obj: &mut Node2d, d: f64) {}
    fn touch(&self, obj: &mut Node2d, id: u64, touch: &Touch, pos: Vec2) {
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
