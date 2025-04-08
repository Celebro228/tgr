use std::vec;

use glam::{mat4, vec2, Mat4, Vec2, Vec3};
use miniquad::{window::set_window_size, *};

use crate::engine::{
    get_fullscreen, get_high_dpi, get_last_frame_time, get_window, get_window_resizable,
    get_window_update, set_delta, set_last_frame_time, set_window, set_window_2, Engine, Obj2d,
};

#[repr(C)]
#[derive(Debug)]
struct Vertex {
    pos: Vec2,
    color: [f32; 4],
}

static mut VERTICES: Vec<Vertex> = Vec::new();
static mut INDICES: Vec<u16> = Vec::new();
static mut INDICES_LEN: i32 = 0;
static mut INDEX_OFFSET: u16 = 0;

#[inline(always)]
fn get_vertices() -> &'static Vec<Vertex> {
    unsafe {
        &VERTICES
    }
}

#[inline(always)]
fn get_indices() -> &'static Vec<u16> {
    unsafe {
        &INDICES
    }
}
#[inline(always)]
fn get_indices_len() -> i32 {
    unsafe {
        INDICES_LEN
    }
}

#[inline(always)]
fn add_vertices(vert: Vec<Vertex>, indi: Vec<u16>) {
    unsafe {
        let indi: Vec<u16> = indi.iter().map(|x| x + INDEX_OFFSET).collect();

        INDEX_OFFSET += vert.len() as u16;
        VERTICES.extend(vert);
        INDICES.extend(indi);
        INDICES_LEN = INDICES.len() as i32;
    }
}

pub(crate) fn clear2d() {
    unsafe {
        VERTICES.clear();
        INDICES.clear();
        INDICES_LEN = 0;
        INDEX_OFFSET = 0;
    }
}

pub(crate) fn draw2d(pos: Vec2, obj: &Obj2d) {
    match obj {
        Obj2d::None => {}
        Obj2d::Circle(r) => {
            let mut vertices = Vec::new();
            let mut indices = Vec::new();

            let pixel_error = 5.;

            let segments = (std::f32::consts::PI * r / pixel_error).ceil() as usize;

            vertices.push(Vertex { pos: vec2(pos.x, pos.y), color: [0., 0., 1., 1.] });

            for i in 0..=segments {
                let theta = i as f32 / segments as f32 * std::f32::consts::TAU;
                let x = pos.x + r * theta.cos();
                let y = pos.y + r * theta.sin();
                vertices.push(Vertex { pos: vec2(x, y), color: [1., 0., 0., 1.] });
            }

            for i in 1..=segments {
                indices.extend_from_slice(&[0, i as u16, (i + 1) as u16]);
            }

            println!("{}", segments);

            add_vertices(vertices, indices);
        }
        Obj2d::Rect(w, h) => {
            add_vertices(vec![
                Vertex { pos: vec2(pos.x - w / 2., pos.y - h / 2.), color: [1., 0., 0., 1.] },
                Vertex { pos: vec2(pos.x + w / 2., pos.y - h / 2.), color: [1., 0., 0., 1.] },
                Vertex { pos: vec2(pos.x + w / 2., pos.y + h / 2.), color: [1., 0., 0., 1.] },
                Vertex { pos: vec2(pos.x - w / 2., pos.y + h / 2.), color: [1., 0., 0., 1.] },
            ],
            vec![
                0, 1, 3, 1, 2, 3,
            ]);
        }
    }
}

struct QuadRender {
    pipeline: Pipeline,
    bindings: Bindings,
    ctx: Box<dyn RenderingBackend>,
}
impl QuadRender {
    pub fn new() -> Self {
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        if get_fullscreen() {
            let (x, y) = window::screen_size();
            set_window_2(x, y);
        }

        let vertex_buffer = ctx.new_buffer(
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
        };

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
            ],
            shader,
            PipelineParams::default(),
        );

        Self {
            pipeline,
            bindings,
            ctx,
        }
    }

    pub fn update_buffer(&mut self) {
        self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(get_vertices()));
        self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(get_indices()));
    }

    pub fn add_buffer(&mut self) {
        let vertex_buffer = self.ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Dynamic,
            BufferSource::slice(get_vertices()),
        );

        let index_buffer = self.ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Dynamic,
            BufferSource::slice(get_indices()),
        );

        self.bindings.vertex_buffers[0] = vertex_buffer;
        self.bindings.index_buffer = index_buffer;
    }
}
impl EventHandler for QuadRender {
    fn update(&mut self) {
        Engine::update();
    }

    fn draw(&mut self) {
        if get_window_update() {
            set_window_size(get_window().x as u32, get_window().y as u32);
        }

        let window_half = get_window() / 2.;
        let proj = Mat4::orthographic_rh_gl(
            -window_half.x,
            window_half.x,
            -window_half.y,
            window_half.y,
            -1.0,
            1.0,
        );
        let vs_param = shader::Uniforms { mvp: proj };

        clear2d();
        Engine::draw2d();
        self.add_buffer();

        self.ctx.begin_default_pass(Default::default());

        self.ctx.apply_pipeline(&self.pipeline);
        self.ctx.apply_bindings(&self.bindings);
        self.ctx.apply_uniforms(UniformsSource::table(&vs_param));
        self.ctx.draw(0, get_indices_len(), 1);
        self.ctx.end_render_pass();
        self.ctx.commit_frame();

        set_delta(date::now() - get_last_frame_time());
        set_last_frame_time(date::now());
    }

    fn resize_event(&mut self, width: f32, height: f32) {
        set_window_2(width, height);
    }
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
            ..Default::default()
        };

        start(conf, || Box::new(QuadRender::new()));
    }
}

mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 in_pos;
    attribute vec4 in_color;

    varying lowp vec4 color;

    uniform mat4 mvp;

    void main() {
        gl_Position = mvp * vec4(in_pos, 0, 1);
        color = in_color;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;

    void main() {
        gl_FragColor = color;
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
            images: vec![],
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


/*fn projection_matrix(window_width: f32, window_height: f32) -> Mat4 {
    // Ожидаемая пропорция канваса (например, 2:2)
    let canvas_width = 2.0; // от -1.0 до 1.0
    let canvas_height = 2.0;

    // Пропорции экрана
    let aspect_window = window_width / window_height;
    let aspect_canvas = canvas_width / canvas_height;

    // Подгоняем масштаб так, чтобы всё влезло
    if aspect_window > aspect_canvas {
        // Окно шире — масштабируем по высоте
        let scale = aspect_canvas / aspect_window;
        Mat4::orthographic_rh_gl(-scale, scale, -1.0, 1.0, -1.0, 1.0)
    } else {
        // Окно выше — масштабируем по ширине
        let scale = aspect_window / aspect_canvas;
        Mat4::orthographic_rh_gl(-1.0, 1.0, -1.0 / scale, 1.0 / scale, -1.0, 1.0)
    }
}*/