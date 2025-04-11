use std::vec;

use crate::{
    engine::{
        get_add_buffer, get_camera, get_canvas, get_canvas_update, get_fullscreen, get_high_dpi,
        get_last_frame_time, get_view_height, get_view_width, get_window, get_window_resizable,
        get_window_update, get_zoom, set_delta, set_last_frame_time, set_mouse, set_mouse_d,
        set_window_2, Engine, Obj2d, View,
    },
    info::DEVICE,
    object::Touch,
};
use glam::{mat4, vec2, Mat4, Vec2, Vec3};
use miniquad::{window::set_window_size, *};
use image::GenericImageView;

static mut RENDERS: Vec<(Vec<Vertex>, Vec<u16>, Option<usize>)> = Vec::new();
static mut TEXUTRES: Vec<TextureId> = Vec::new();
static mut TEXUTRES_ID: usize = 0;
static mut TEXUTRES_BUFFER: Vec<(Vec<u8>, u16, u16)> = Vec::new();

static mut MOUSE_PROJ: Vec2 = Vec2::new(0., 0.);

#[repr(C)]
struct Vertex {
    pos: Vec2,
    color: [f32; 4],
    uv: Vec2,
}

struct QuadRender {
    pipeline: Pipeline,
    bindings: Vec<Bindings>,
    ctx: Box<dyn RenderingBackend>,
    proj: Mat4,
    white: TextureId,
}
impl QuadRender {
    pub fn new() -> Self {
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        if get_fullscreen() {
            let (x, y) = window::screen_size();
            set_window_2(x, y);
        }

        let proj = get_proj();

        /*let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Dynamic,
            BufferSource::slice(get_vertices()),
        );

        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Dynamic,
            BufferSource::slice(get_indices()),
        );

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer: index_buffer,
            images: vec![],
        };*/

        let white = ctx.new_texture_from_rgba8(1, 1, &[0xFF, 0xFF, 0xFF, 0xFF]);

        let shader = ctx
            .new_shader(
                match ctx.info().backend {
                    Backend::OpenGl => ShaderSource::Glsl {
                        vertex: shader::VERTEX,
                        fragment: shader::FRAGMENT,
                    },
                    Backend::Metal => ShaderSource::Msl {
                        program: shader::METAL,
                    },
                },
                shader::meta(),
            )
            .unwrap();

        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("in_pos", VertexFormat::Float2),
                VertexAttribute::new("in_color", VertexFormat::Float4),
                VertexAttribute::new("in_uv", VertexFormat::Float2),
            ],
            shader,
            PipelineParams::default(),
        );

        set_last_frame_time(date::now());

        Self {
            pipeline,
            bindings: Vec::new(),
            ctx,
            proj,
            white,
        }
    }
}
impl EventHandler for QuadRender {
    fn update(&mut self) {
        Engine::update();
    }

    fn draw(&mut self) {
        if get_window_update() {
            set_window_size(get_window().x as u32, get_window().y as u32);
            self.proj = get_proj();
        } else if get_canvas_update() {
            self.proj = get_proj();
        }

        set_mouse_d(0., 0.);

        clear_render();
        Engine::draw2d();

        if get_add_buffer() {
            let o = get_render().len() - self.bindings.len();

            self.bindings.clear();

            unsafe {
                for i in &TEXUTRES_BUFFER {
                    TEXUTRES.push(self.ctx.new_texture_from_rgba8(i.1, i.2, &i.0));
                }
                TEXUTRES_BUFFER.clear();
            }

            for i in get_render() {
                let vertex_buffer = self.ctx.new_buffer(
                    BufferType::VertexBuffer,
                    BufferUsage::Dynamic,
                    BufferSource::slice(&i.0),
                );
        
                let index_buffer = self.ctx.new_buffer(
                    BufferType::IndexBuffer,
                    BufferUsage::Dynamic,
                    BufferSource::slice(&i.1),
                );

                let images = if let Some(id) = i.2 {
                    unsafe {
                        TEXUTRES[id]
                    }
                } else {
                    self.white
                };
        
                let bindings = Bindings {
                    vertex_buffers: vec![vertex_buffer],
                    index_buffer,
                    images: vec![images],
                };

                self.bindings.push(bindings);
            }
        } else {
            for i in get_render().iter().enumerate() {
                self.ctx.buffer_update(
                    self.bindings[i.0].vertex_buffers[0],
                    BufferSource::slice(&i.1.0),
                );
                self.ctx.buffer_update(
                    self.bindings[i.0].index_buffer,
                    BufferSource::slice(&i.1.1),
                );
            }
        }

        self.ctx.begin_default_pass(Default::default());

        self.ctx.apply_pipeline(&self.pipeline);
        self.ctx
            .apply_uniforms(UniformsSource::table(&shader::Uniforms { mvp: self.proj }));

        for i in get_render().iter().enumerate() {
            self.ctx.apply_bindings(&self.bindings[i.0]);
            self.ctx.draw(0, i.1.1.len() as i32, 1);
        }
        
        self.ctx.end_render_pass();
        self.ctx.commit_frame();

        set_delta(date::now() - get_last_frame_time());
        set_last_frame_time(date::now());
    }

    fn resize_event(&mut self, width: f32, height: f32) {
        set_window_2(width, height);
        self.proj = get_proj();
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        let half_window = get_window() / 2.;
        set_mouse(
            (x - half_window.x) * get_mouse_proj().x,
            (y - half_window.y) * get_mouse_proj().y,
        );
        Engine.touch(
            0,
            &Touch::Move,
            vec2(
                (x - half_window.x) * get_mouse_proj().x,
                (y - half_window.y) * get_mouse_proj().y,
            ),
        );
    }

    fn raw_mouse_motion(&mut self, dx: f32, dy: f32) {
        set_mouse_d(dx, dy);
    }

    fn mouse_button_down_event(&mut self, _button: MouseButton, x: f32, y: f32) {
        let half_window = get_window() / 2.;
        Engine.touch(
            0,
            &Touch::Down,
            vec2(
                (x - half_window.x) * get_mouse_proj().x,
                (y - half_window.y) * get_mouse_proj().y,
            ),
        );
    }

    fn mouse_button_up_event(&mut self, _button: MouseButton, x: f32, y: f32) {
        let half_window = get_window() / 2.;
        Engine.touch(
            0,
            &Touch::Up,
            vec2(
                (x - half_window.x) * get_mouse_proj().x,
                (y - half_window.y) * get_mouse_proj().y,
            ),
        );
    }

    fn touch_event(&mut self, phase: TouchPhase, id: u64, x: f32, y: f32) {
        let half_window = get_window() / 2.;
        match phase {
            TouchPhase::Started => {
                Engine.touch(id, &Touch::Down, vec2(
                    (x - half_window.x) * get_mouse_proj().x,
                    (y - half_window.y) * get_mouse_proj().y,
                ));
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                Engine.touch(id, &Touch::Up, vec2(
                    (x - half_window.x) * get_mouse_proj().x,
                    (y - half_window.y) * get_mouse_proj().y,
                ));
            }
            TouchPhase::Moved => {
                Engine.touch(id, &Touch::Move, vec2(
                    (x - half_window.x) * get_mouse_proj().x,
                    (y - half_window.y) * get_mouse_proj().y,
                ));
            }
        }
    }

    fn mouse_wheel_event(&mut self, _x: f32, _y: f32) {}

    fn key_down_event(&mut self, _keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {}
}

pub(crate) struct Render;

impl Render {
    pub(crate) fn start(name: &str) {
        let conf = conf::Conf {
            window_title: name.to_string(),
            window_width: get_window().x as i32,
            window_height: get_window().y as i32,
            high_dpi: get_high_dpi(),
            fullscreen: get_fullscreen(),
            window_resizable: get_window_resizable(),
            sample_count: match DEVICE {
                0 => 4,
                1 => 2,
                _ => 1,
            },
            ..Default::default()
        };

        start(conf, || Box::new(QuadRender::new()));
    }
}

fn get_proj() -> Mat4 {
    let aspect_window = get_window().x / get_window().y;
    let aspect_canvas = get_canvas().x / get_canvas().y;

    let canvas = get_canvas() / 2. * get_zoom();
    let window = get_window() / 2.;

    let view: &View = if aspect_window > aspect_canvas {
        get_view_width()
    } else {
        get_view_height()
    };

    match view {
        View::KeepWidth => {
            let scale = canvas.y / (aspect_window / aspect_canvas);
            set_mouse_proj(canvas.x / window.x, scale / window.y);
            Mat4::orthographic_rh_gl(-canvas.x, canvas.x, scale, -scale, -1.0, 1.0)
        }
        View::KeepHeight => {
            let scale = canvas.x / (aspect_canvas / aspect_window);
            set_mouse_proj(scale / window.x, canvas.y / window.y);
            Mat4::orthographic_rh_gl(-scale, scale, canvas.y, -canvas.y, -1.0, 1.0)
        }
        View::Scale => {
            set_mouse_proj(canvas.x / window.x, canvas.y / window.y);
            Mat4::orthographic_rh_gl(-canvas.x, canvas.x, canvas.y, -canvas.y, -1.0, 1.0)
        }
    }
}

pub(crate) fn draw2d(pos: Vec2, obj: &Obj2d, scale: Vec2, color: [f32; 4]) {
    let pos = pos - get_camera();
    match obj {
        Obj2d::None => {}
        Obj2d::Circle(r) => {
            let r = r;

            let mut vertices = Vec::new();
            let mut indices = Vec::new();

            let segments = 50;

            vertices.push(Vertex {
                pos: vec2(pos.x, pos.y),
                color: color,
                uv: Vec2::new(0., 0.),
            });

            for i in 0..=segments {
                let theta = i as f32 / segments as f32 * std::f32::consts::TAU;
                let x = pos.x + r * theta.cos() * scale.x;
                let y = pos.y + r * theta.sin() * scale.y;
                vertices.push(Vertex {
                    pos: vec2(x, y),
                    color: color,
                    uv: Vec2::new(0., 0.),
                });
            }

            for i in 1..=segments {
                indices.extend_from_slice(&[0, i as u16, (i + 1) as u16]);
            }

            add_render(vertices, indices, None);
        }
        Obj2d::Rect(w, h) => {
            let w = (w * scale.x) / 2.;
            let h = (h * scale.y) / 2.;
            add_render(
                vec![
                    Vertex {
                        pos: vec2(pos.x - w, pos.y - h),
                        color: color,
                        uv: Vec2::new(0., 0.),
                    },
                    Vertex {
                        pos: vec2(pos.x + w, pos.y - h),
                        color: color,
                        uv: Vec2::new(0., 0.),
                    },
                    Vertex {
                        pos: vec2(pos.x + w, pos.y + h),
                        color: color,
                        uv: Vec2::new(0., 0.),
                    },
                    Vertex {
                        pos: vec2(pos.x - w, pos.y + h),
                        color: color,
                        uv: Vec2::new(0., 0.),
                    },
                ],
                vec![0, 1, 3, 1, 2, 3],
                None
            );
        }
        Obj2d::Texture(img, w, h) => {
            let w2 = (*w as f32 * scale.x) / 2.;
            let h2 = (*h as f32 * scale.y) / 2.;
            add_render(
                vec![
                    Vertex {
                        pos: vec2(pos.x - w2, pos.y - h2),
                        color: color,
                        uv: vec2(0., 0.),
                    },
                    Vertex {
                        pos: vec2(pos.x + w2, pos.y - h2),
                        color: color,
                        uv: vec2(1., 0.),
                    },
                    Vertex {
                        pos: vec2(pos.x + w2, pos.y + h2),
                        color: color,
                        uv: vec2(1., 1.),
                    },
                    Vertex {
                        pos: vec2(pos.x - w2, pos.y + h2),
                        color: color,
                        uv: vec2(0., 1.),
                    },
                ],
                vec![0, 1, 3, 1, 2, 3],
                Some(*img),
            );
        }
    }
}

#[inline(always)]
fn add_render(vert: Vec<Vertex>, indi: Vec<u16>, img: Option<usize>) {
    unsafe {
        RENDERS.push((vert, indi, img));
    }
}

pub(crate) fn clear_render() {
    unsafe {
        RENDERS.clear();
    }
}

#[inline(always)]
fn get_render() -> &'static Vec<(Vec<Vertex>, Vec<u16>, Option<usize>)> {
    unsafe { &RENDERS }
}

#[inline(always)]
pub(crate) fn add_texture(path: &str) -> (usize, f32, f32) {
    let img = image::open(path).expect("Error to load texture");

    let rgba = img.to_rgba8().to_vec();
    let (width, height) = img.dimensions();

    unsafe {
        TEXUTRES_BUFFER.push((rgba, width as u16, height as u16));
        TEXUTRES_ID += 1;
        (TEXUTRES_ID - 1, width as f32, height as f32)
    }
}

pub(crate) fn clear_texture() {
    unsafe {
        TEXUTRES.clear();
    }
}

#[inline(always)]
fn get_texture(id: usize) -> &'static TextureId {
    unsafe { &TEXUTRES[id] }
}

#[inline(always)]
fn set_mouse_proj(x: f32, y: f32) {
    unsafe {
        MOUSE_PROJ = vec2(x, y);
    }
}
#[inline(always)]
fn get_mouse_proj() -> Vec2 {
    unsafe { MOUSE_PROJ }
}

mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 in_pos;
    attribute vec4 in_color;
    attribute vec2 in_uv;

    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform mat4 mvp;

    void main() {
        gl_Position = mvp * vec4(in_pos, 0, 1);
        color = in_color;
        uv = in_uv;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform sampler2D tex;

    void main() {
        gl_FragColor = texture2D(tex, uv) * color;
    }"#;

    pub const METAL: &str = r#"
    #include <metal_stdlib>

    using namespace metal;

    struct Vertex
    {
        float2 in_pos   [[attribute(0)]];
        float4 in_color [[attribute(1)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
        float4 color [[user(locn0)]];
    };

    vertex RasterizerData vertexShader(Vertex v [[stage_in]])
    {
        RasterizerData out;

        out.position = float4(v.in_pos.xy, 0.0, 1.0);
        out.color = v.in_color;

        return out;
    }

    fragment float4 fragmentShader(RasterizerData in [[stage_in]])
    {
        return in.color;
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![UniformDesc::new("mvp", UniformType::Mat4)],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub mvp: glam::Mat4,
    }
}
