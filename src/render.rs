use glam::{vec2, Vec2, Vec3};
use miniquad::{window::set_window_size, *};

use crate::engine::{
    get_fullscreen, get_high_dpi, get_last_frame_time, get_window, get_window_resizable, get_window_update, set_delta, set_last_frame_time, set_window, set_window_2, Engine
};

#[repr(C)]
struct Vertex {
    pos: Vec2,
    color: [f32; 4],
}

struct QuadRender {
    pipeline: Pipeline,
    bindings: Bindings,
    ctx: Box<dyn RenderingBackend>,
}
impl QuadRender {
    pub fn new() -> Self {
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        let vertices = [
            Vertex {
                pos: vec2(-1., -0.5),
                color: [1., 0., 0., 1.],
            },
            Vertex {
                pos: vec2(0.5, -0.5),
                color: [0., 1., 0., 1.],
            },
            Vertex {
                pos: vec2(0.5, 0.5),
                color: [0., 0., 1., 1.],
            },
            Vertex {
                pos: vec2(-0.5, 0.5),
                color: [0., 0., 1., 1.],
            },
        ];

        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
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
            pipeline: pipeline,
            bindings: bindings,
            ctx: ctx,
        }
    }
}
impl EventHandler for QuadRender {
    fn update(&mut self) {
        Engine::update();

        if get_window_update() {
            set_window_size(get_window().x as u32, get_window().y as u32);
        }
    }

    fn draw(&mut self) {
        self.ctx.begin_default_pass(Default::default());

        self.ctx.apply_pipeline(&self.pipeline);
        self.ctx.apply_bindings(&self.bindings);
        self.ctx.draw(0, 3, 1);
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

    void main() {
        gl_Position = vec4(in_pos, 0, 1);
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
            uniforms: UniformBlockLayout { uniforms: vec![] },
        }
    }
}