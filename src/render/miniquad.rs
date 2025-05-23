use super::{
    d2::{upd_proj, CAMERA2D, CANVAS_UPDATE, MOUSE_PROJ, PROJ, RENDERS},
    DELTA, LAST_FPS_TIME, LAST_FRAME_TIME, TEXUTRES_BUFFER, TEXUTRES_UPDATE, WINDOW, WINDOW_UPDATE, Texture,
};
use crate::{
    engine::{
        draw, touch, update, BACKGRAUND, FULLSCREEN, HIGH_DPI, MOUSE, MOUSE_DELTA,
        MOUSE_WHEEL_DELTA, RESIZABLE,
    },
    object::d2::DrawUpdate,
    info::DEVICE,
    object::Touch, render::Vertex,
};

use glam::{vec2, Vec2};
use miniquad::{window::set_window_size, *};
use std::{usize, vec};

static mut TEXUTRES: Vec<Option<TextureId>> = Vec::new();

struct QuadRender {
    pipeline: Pipeline,
    bindings: Vec<Bindings>,
    ctx: Box<dyn RenderingBackend>,
    white: TextureId,
}
impl QuadRender {
    pub fn new() -> Self {
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        unsafe {
            if FULLSCREEN || DEVICE != 0 {
                let (x, y) = window::screen_size();
                WINDOW = vec2(x, y);
            }
        }

        upd_proj();

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
            .expect("Error to load shaders");

        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("in_pos", VertexFormat::Float3),
                VertexAttribute::new("in_color", VertexFormat::Float4),
                VertexAttribute::new("in_uv", VertexFormat::Float2),
            ],
            shader,
            PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                alpha_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Zero,
                    BlendFactor::One,
                )),
                ..Default::default()
            },
        );

        unsafe {
            LAST_FRAME_TIME = date::now();
            LAST_FPS_TIME = LAST_FRAME_TIME + 1.
        }

        Self {
            pipeline,
            bindings: Vec::new(),
            ctx,
            white,
        }
    }
}
impl EventHandler for QuadRender {
    fn update(&mut self) {
        let u = std::time::Instant::now();
        update();
        println!("update: {:?}", u.elapsed());
    }

    fn draw(&mut self) {
        unsafe {
            if WINDOW_UPDATE {
                WINDOW_UPDATE = false;
                set_window_size(WINDOW.x as u32, WINDOW.y as u32);
            };
        };

        let u = std::time::Instant::now();
        draw();
        println!("vertex: {:?}", u.elapsed());

        unsafe {
            for i in &TEXUTRES_BUFFER {
                TEXUTRES.push(Some(self.ctx.new_texture_from_rgba8(i.1, i.2, &i.0)));
            }

            TEXUTRES_BUFFER.clear();

            for i in &TEXUTRES_UPDATE {
                if let Some(tex) = TEXUTRES[i.0] {
                    self.ctx
                        .texture_resize(tex, i.2 as u32, i.3 as u32, Some(&i.1));
                    //self.ctx.texture_update_part(tex, 0, 0, i.2 as i32, i.3 as i32, &i.1);
                }
            }
            TEXUTRES_UPDATE.clear();
        }

        let u = std::time::Instant::now();

        //self.ctx.clear(Some((backgraund.r, backgraund.g, backgraund.b, backgraund.a)), None, None);
        unsafe {
            self.ctx.begin_default_pass(PassAction::clear_color(
                BACKGRAUND.r,
                BACKGRAUND.g,
                BACKGRAUND.b,
                BACKGRAUND.a,
            ));
        }

        self.ctx.apply_pipeline(&self.pipeline);
        self.ctx
            .apply_uniforms(UniformsSource::table(&shader::Uniforms {
                mvp: unsafe { PROJ },
            }));
        
        let mut verts: Vec<Vertex> = vec![];
        let mut indis: Vec<u16> = vec![];
        let mut last_texture: Option<usize> = Some(usize::MAX);
        let mut new_texture: Option<usize> = last_texture;
        let mut last_new_render = DrawUpdate::None;

        let mut bind_num: usize = 0;
        
        for i in unsafe { RENDERS.iter_mut().enumerate() } {
            if let None = i.1 {
                continue;
            }

            let obj = i.1.as_mut().unwrap();

            last_new_render = if obj.3 == DrawUpdate::Create || last_new_render == DrawUpdate::Create {
                DrawUpdate::Create
            } else if obj.3 == DrawUpdate::Update || last_new_render == DrawUpdate::Update {
                DrawUpdate::Update
            } else {
                DrawUpdate::None
            };
            obj.3 = DrawUpdate::None;

            new_texture = obj.2.clone();

            if (new_texture != last_texture && i.0 != 0 ) || i.0 == unsafe { RENDERS.len() - 1 } {
                if last_new_render == DrawUpdate::Create {
                    let vertex_buffer = self.ctx.new_buffer(
                        BufferType::VertexBuffer,
                        BufferUsage::Dynamic,
                        BufferSource::slice(&verts),
                    );

                    let index_buffer = self.ctx.new_buffer(
                        BufferType::IndexBuffer,
                        BufferUsage::Dynamic,
                        BufferSource::slice(&indis),
                    );

                    let images = if let Some(id) = last_texture {
                        unsafe {
                            if let Some(tex) = TEXUTRES[id] {
                                tex
                            } else {
                                self.white
                            }
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
                } else if last_new_render == DrawUpdate::Update {
                    self.ctx.buffer_update(
                        self.bindings[bind_num].vertex_buffers[0],
                        BufferSource::slice(&verts),
                    );
                    self.ctx.buffer_update(
                        self.bindings[bind_num].index_buffer,
                        BufferSource::slice(&indis),
                    );
                }

                self.ctx.apply_bindings(&self.bindings[bind_num]);
                self.ctx.draw(0, indis.len() as i32, 1);

                bind_num += 1;

                last_new_render = DrawUpdate::None;
            }

            if new_texture != last_texture || i.0 == unsafe { RENDERS.len() - 1 } || i.0 == 0 {
                verts = obj.0.clone();
                indis = obj.1.clone();

                last_texture = new_texture;
            } else {
                let base_index = verts.len() as u16;
                let vert: Vec<Vertex> = obj.0.clone();
                let mut indi = obj.1.clone();

                // Смещаем индексы на количество уже имеющихся вершин
                for index in &mut indi {
                    *index += base_index;
                }

                verts.extend(vert);
                indis.extend(indi);
            }
        }

        self.ctx.end_render_pass();

        self.ctx.commit_frame();
        println!("frame: {:?}", u.elapsed());

        unsafe {
            DELTA = (date::now() - LAST_FRAME_TIME) as f32;
            LAST_FRAME_TIME = date::now();
            println!("fps: {}", 1. / DELTA);
        }
    }

    fn resize_event(&mut self, width: f32, height: f32) {
        unsafe {
            WINDOW = vec2(width, height);
            CANVAS_UPDATE = true;
        }
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        unsafe {
            MOUSE = get_mouse_proj(x, y);
            touch(0, &Touch::Move, vec2(MOUSE.x, MOUSE.y));
        }
    }

    fn raw_mouse_motion(&mut self, dx: f32, dy: f32) {
        unsafe { MOUSE_DELTA = vec2(dx, dy) }
    }

    fn mouse_button_down_event(&mut self, button: MouseButton, x: f32, y: f32) {
        touch(
            match button {
                MouseButton::Left => 0,
                MouseButton::Right => 1,
                MouseButton::Middle => 2,
                MouseButton::Unknown => 3,
            },
            &Touch::Press,
            get_mouse_proj(x, y),
        );
    }

    fn mouse_button_up_event(&mut self, button: MouseButton, x: f32, y: f32) {
        touch(
            match button {
                MouseButton::Left => 0,
                MouseButton::Right => 1,
                MouseButton::Middle => 2,
                MouseButton::Unknown => 3,
            },
            &Touch::Relese,
            get_mouse_proj(x, y),
        );
    }

    fn mouse_wheel_event(&mut self, x: f32, y: f32) {
        unsafe { MOUSE_WHEEL_DELTA = vec2(x, y) }
    }

    fn touch_event(&mut self, phase: TouchPhase, id: u64, x: f32, y: f32) {
        match phase {
            TouchPhase::Started => {
                touch(id, &Touch::Press, get_mouse_proj(x, y));
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                touch(id, &Touch::Relese, get_mouse_proj(x, y));
            }
            TouchPhase::Moved => {
                touch(id, &Touch::Move, get_mouse_proj(x, y));
            }
        }
    }

    /*fn char_event(&mut self, character: char, keymods: KeyMods, repeat: bool) {
        if !repeat {
            Engine.key(&Key::Char(character), keymods, &Touch::Press);
        }
    }

    fn key_down_event(&mut self, keycode: KeyCode, keymods: KeyMods, repeat: bool) {
        if !repeat {
            Engine.key(&Key::Code(keycode), keymods, &Touch::Press);
        }
    }

    fn key_up_event(&mut self, keycode: KeyCode, keymods: KeyMods) {
        Engine.key(&Key::Code(keycode), keymods, &Touch::Relese);
    }*/
}

pub(crate) fn render(name: &str) {
    unsafe {
        let conf = conf::Conf {
            window_title: name.to_string(),
            window_width: WINDOW.x as i32,
            window_height: WINDOW.y as i32,
            high_dpi: HIGH_DPI,
            fullscreen: FULLSCREEN,
            window_resizable: RESIZABLE,
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

#[inline(always)]
fn get_mouse_proj(x: f32, y: f32) -> Vec2 {
    //y - half_window.y) * get_mouse_proj().y - get_camera().y,
    unsafe { (vec2(x, y) - WINDOW / 2.) * MOUSE_PROJ + CAMERA2D }
}

mod shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec3 in_pos;
    attribute vec4 in_color;
    attribute vec2 in_uv;

    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform mat4 mvp;

    void main() {
        gl_Position = mvp * vec4(in_pos, 1);
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
        float3 in_pos   [[attribute(0)]];
        float4 in_color [[attribute(1)]];
        float2 in_uv    [[attribute(2)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
        float4 color [[user(locn0)]];
        float2 uv [[user(locn1)]];
    };

    vertex RasterizerData vertexShader(Vertex v [[stage_in]], constant Uniforms& uniforms [[buffer(0)]])
    {
        RasterizerData out;

        out.position = uniforms.mvp * float4(v.in_pos, 1.0);
        out.color = v.in_color;
        out.uv = v.in_uv;

        return out;
    }

    fragment float4 fragmentShader(RasterizerData in [[stage_in]], texture2d<float> tex [[texture(0)]], sampler texSmplr [[sampler(0)]])
    {
        return in.color * tex.sample(texSmplr, in.uv);
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
