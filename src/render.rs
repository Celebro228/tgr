use std::vec;

use crate::{
    engine::{
        add_fps_buffer, get_add_buffer, get_backgraund, get_camera, get_canvas, get_canvas_update,
        get_fps_buffer, get_fullscreen, get_high_dpi, get_last_fps_time, get_last_frame_time,
        get_view_height, get_view_width, get_window, get_window_resizable, get_window_update,
        get_zoom, set_canvas_proj, set_delta, set_fps, set_fps_buffer, set_last_fps_time,
        set_last_frame_time, set_mouse, set_mouse_d, set_mouse_wheel_d, set_window_2, Engine, Key,
        Obj2d, View,
    },
    info::DEVICE,
    object::Touch,
};
use glam::{mat4, vec2, Mat4, Vec2, Vec3};
use image::{DynamicImage, GenericImageView, ImageBuffer};
use miniquad::{window::set_window_size, *};
use rusttype::{point, Font, Point, Scale};

static mut RENDERS: Vec<(Vec<Vertex>, Vec<u16>, Option<usize>)> = Vec::new();

static mut TEXUTRES: Vec<Option<TextureId>> = Vec::new();
static mut TEXUTRES_ID: usize = 0;
static mut TEXUTRES_BUFFER: Vec<(Vec<u8>, u16, u16)> = Vec::new();
static mut TEXUTRES_UPDATE: Vec<(usize, Vec<u8>, u16, u16)> = Vec::new();

static mut FONTS: Vec<Font> = Vec::new();

static mut PROJ: Mat4 = Mat4::IDENTITY;
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
    white: TextureId,
}
impl QuadRender {
    pub fn new() -> Self {
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        if get_fullscreen() || DEVICE != 0 {
            let (x, y) = window::screen_size();
            set_window_2(x, y);
        }

        set_proj();

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

        set_last_frame_time(date::now());
        set_last_fps_time(get_last_frame_time() + 1.);

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
        Engine::update();
    }

    fn draw(&mut self) {
        if get_window_update() {
            set_window_size(get_window().x as u32, get_window().y as u32);
            set_proj();
        } else if get_canvas_update() {
            set_proj();
        }

        set_mouse_d(0., 0.);
        set_mouse_wheel_d(0., 0.);

        clear_render();
        Engine::draw2d();

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

        if get_add_buffer() {
            self.bindings.clear();

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
            }
        } else {
            for i in get_render().iter().enumerate() {
                self.ctx.buffer_update(
                    self.bindings[i.0].vertex_buffers[0],
                    BufferSource::slice(&i.1 .0),
                );
                self.ctx.buffer_update(
                    self.bindings[i.0].index_buffer,
                    BufferSource::slice(&i.1 .1),
                );
            }
        }

        let backgraund = get_backgraund();

        //self.ctx.clear(Some((backgraund.r, backgraund.g, backgraund.b, backgraund.a)), None, None);
        self.ctx.begin_default_pass(PassAction::clear_color(
            backgraund.r,
            backgraund.g,
            backgraund.b,
            backgraund.a,
        ));

        self.ctx.apply_pipeline(&self.pipeline);
        self.ctx
            .apply_uniforms(UniformsSource::table(&shader::Uniforms {
                mvp: unsafe { PROJ },
            }));

        for i in get_render().iter().enumerate() {
            self.ctx.apply_bindings(&self.bindings[i.0]);
            self.ctx.draw(0, i.1 .1.len() as i32, 1);
        }

        self.ctx.end_render_pass();

        self.ctx.commit_frame();

        set_delta((date::now() - get_last_frame_time()) as f32);
        set_last_frame_time(date::now());

        add_fps_buffer(1);
        if get_last_fps_time() <= get_last_frame_time() {
            set_fps(get_fps_buffer());
            set_fps_buffer(0);
            set_last_fps_time(get_last_frame_time() + 1.);
        }
    }

    fn resize_event(&mut self, width: f32, height: f32) {
        set_window_2(width, height);
        set_proj();
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        let mouse = get_mouse_proj(x, y);

        set_mouse(mouse.x, mouse.y);

        Engine.touch(0, &Touch::Move, vec2(mouse.x, mouse.y));
    }

    fn raw_mouse_motion(&mut self, dx: f32, dy: f32) {
        set_mouse_d(dx, dy);
    }

    fn mouse_button_down_event(&mut self, button: MouseButton, x: f32, y: f32) {
        Engine.touch(
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
        Engine.touch(
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
        set_mouse_wheel_d(x, y);
    }

    fn touch_event(&mut self, phase: TouchPhase, id: u64, x: f32, y: f32) {
        match phase {
            TouchPhase::Started => {
                Engine.touch(id, &Touch::Press, get_mouse_proj(x, y));
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                Engine.touch(id, &Touch::Relese, get_mouse_proj(x, y));
            }
            TouchPhase::Moved => {
                Engine.touch(id, &Touch::Move, get_mouse_proj(x, y));
            }
        }
    }

    fn char_event(&mut self, character: char, keymods: KeyMods, repeat: bool) {
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

fn set_proj() {
    let aspect_window = get_window().x / get_window().y;
    let aspect_canvas = get_canvas().x / get_canvas().y;

    let canvas = get_canvas() / 2. * get_zoom();
    let window = get_window() / 2.;

    let view: &View = if aspect_window > aspect_canvas {
        get_view_width()
    } else {
        get_view_height()
    };

    let proj = match view {
        View::KeepWidth => {
            let scale = canvas.y / (aspect_window / aspect_canvas);
            set_mouse_proj(canvas.x / window.x, scale / window.y);
            set_canvas_proj(canvas.x, scale);
            Mat4::orthographic_rh_gl(-canvas.x, canvas.x, scale, -scale, -1.0, 1.0)
        }
        View::KeepHeight => {
            let scale = canvas.x / (aspect_canvas / aspect_window);
            set_mouse_proj(scale / window.x, canvas.y / window.y);
            set_canvas_proj(scale, canvas.y);
            Mat4::orthographic_rh_gl(-scale, scale, canvas.y, -canvas.y, -1.0, 1.0)
        }
        View::Scale => {
            set_mouse_proj(canvas.x / window.x, canvas.y / window.y);
            set_canvas_proj(canvas.x, canvas.y);
            Mat4::orthographic_rh_gl(-canvas.x, canvas.x, canvas.y, -canvas.y, -1.0, 1.0)
        }
        View::Window => {
            let window = window * get_zoom();
            set_mouse_proj(get_zoom(), get_zoom());
            set_canvas_proj(window.x, window.y);
            Mat4::orthographic_rh_gl(-window.x, window.x, window.y, -window.y, -1.0, 1.0)
        }
    };

    unsafe { PROJ = proj }
}

pub(crate) fn draw2d(pos: Vec2, obj: &Obj2d, scale: Vec2, rotation: f32, color: [f32; 4]) {
    let pos = pos - get_camera();
    match obj {
        Obj2d::None => {}
        Obj2d::Circle(r) => {
            let mut vertices: Vec<Vertex> = Vec::new();
            let mut indices: Vec<u16> = Vec::new();

            let segments = 40;

            vertices.push(Vertex {
                pos: vec2(pos.x, pos.y),
                color,
                uv: Vec2::new(0., 0.),
            });

            for i in 0..segments {
                let theta = i as f32 / segments as f32 * std::f32::consts::TAU;
                let x = r * theta.cos() * scale.x;
                let y = r * theta.sin() * scale.y;
                let p = rotate(vec2(x, y), pos, rotation);
                vertices.push(Vertex {
                    pos: p,
                    color: color,
                    uv: Vec2::new(0., 0.),
                });
            }

            for i in 1..segments {
                indices.extend([0, i as u16, (i + 1) as u16]);
            }
            indices.extend([0, segments, 1]);

            add_render(vertices, indices, None);
        }
        Obj2d::Rect(w, h, r) => {
            let w = (w * scale.x) / 2.;
            let h = (h * scale.y) / 2.;

            let mut vertices: Vec<Vertex> = Vec::new();
            let mut indices: Vec<u16> = Vec::new();

            if *r <= 1. {
                vertices.extend([
                    Vertex {
                        pos: rotate(vec2(-w, -h), pos, rotation),
                        color: color,
                        uv: Vec2::new(0., 0.),
                    },
                    Vertex {
                        pos: rotate(vec2(w, -h), pos, rotation),
                        color: color,
                        uv: Vec2::new(0., 0.),
                    },
                    Vertex {
                        pos: rotate(vec2(w, h), pos, rotation),
                        color: color,
                        uv: Vec2::new(0., 0.),
                    },
                    Vertex {
                        pos: rotate(vec2(-w, h), pos, rotation),
                        color: color,
                        uv: Vec2::new(0., 0.),
                    },
                ]);
                indices.extend(vec![0, 1, 3, 1, 2, 3]);
            } else {
                let segments = 40;

                let half_segments = segments / 4;

                let corner_centers = [
                    vec2(w - r, h - r),   // bottom-right
                    vec2(-w + r, h - r),  // bottom-left
                    vec2(-w + r, -h + r), // top-left
                    vec2(w - r, -h + r),  // top-right
                ];

                vertices.push(Vertex {
                    pos: vec2(pos.x, pos.y),
                    color,
                    uv: Vec2::new(0., 0.),
                });

                for (corner_index, &center) in corner_centers.iter().enumerate() {
                    for i in 0..half_segments {
                        let theta = (corner_index * half_segments + i) as f32 / segments as f32
                            * std::f32::consts::TAU;
                        let x = center.x + r * theta.cos() * scale.x;
                        let y = center.y + r * theta.sin() * scale.y;
                        let p = rotate(vec2(x, y), pos, rotation);
                        vertices.push(Vertex {
                            pos: p,
                            color: color,
                            uv: Vec2::new(0., 0.),
                        });
                    }
                }

                for i in 1..segments {
                    indices.extend([0, i as u16, (i + 1) as u16]);
                }
                indices.extend([0, segments as u16, 1]);
            }

            add_render(vertices, indices, None);
        }
        Obj2d::Texture(t) | Obj2d::Text(_, _, _, t) => {
            let w = (t.width as f32 * scale.x) / 2.;
            let h = (t.height as f32 * scale.y) / 2.;
            add_render(
                vec![
                    Vertex {
                        pos: rotate(vec2(-w, -h), pos, rotation),
                        color: color,
                        uv: Vec2::new(0., 0.),
                    },
                    Vertex {
                        pos: rotate(vec2(w, -h), pos, rotation),
                        color: color,
                        uv: Vec2::new(1., 0.),
                    },
                    Vertex {
                        pos: rotate(vec2(w, h), pos, rotation),
                        color: color,
                        uv: Vec2::new(1., 1.),
                    },
                    Vertex {
                        pos: rotate(vec2(-w, h), pos, rotation),
                        color: color,
                        uv: Vec2::new(0., 1.),
                    },
                ],
                vec![0, 1, 3, 1, 2, 3],
                Some(t.id),
            );
        }
    }
}

#[inline(always)]
fn rotate(p: Vec2, center: Vec2, rotation: f32) -> Vec2 {
    if rotation != 0. {
        let s = rotation.sin();
        let c = rotation.cos();

        vec2(p.x * c - p.y * s, p.x * s + p.y * c) + center
    } else {
        p + center
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

pub(crate) fn get_font(path: &str) -> usize {
    let file = std::fs::read(path).unwrap();
    let file: &'static [u8] = Box::leak(file.into_boxed_slice());
    let font = Font::try_from_bytes(&file).expect("Error font bytes");

    unsafe {
        FONTS.push(font);
        FONTS.len() - 1
    }
}

#[inline(always)]
fn add_texture_buffer(rgba: Vec<u8>, width: u32, height: u32) -> (usize, f32, f32) {
    unsafe {
        TEXUTRES_BUFFER.push((rgba, width as u16, height as u16));
        TEXUTRES_ID += 1;
        (TEXUTRES_ID - 1, width as f32, height as f32)
    }
}
#[inline(always)]
fn upd_texture_buffer(id: usize, rgba: Vec<u8>, width: u32, height: u32) -> (usize, f32, f32) {
    unsafe {
        TEXUTRES_UPDATE.push((id, rgba, width as u16, height as u16));
        (id, width as f32, height as f32)
    }
}

#[inline(always)]
pub(crate) fn add_texture(path: &str) -> (usize, f32, f32) {
    let img = image::open(path).expect("Error to load texture");

    let rgba = img.to_rgba8().to_vec();
    let (width, height) = img.dimensions();

    add_texture_buffer(rgba, width, height)

    /*unsafe {
        TEXUTRES_BUFFER.push((rgba, width as u16, height as u16));
        TEXUTRES_ID += 1;
        (TEXUTRES_ID - 1, width as f32, height as f32)
    }*/
}

pub(crate) fn add_text(
    text: &str,
    size: f32,
    font_id: usize,
    upd: Option<usize>,
) -> (usize, f32, f32) {
    let scale = Scale::uniform(size);

    let font = unsafe { &FONTS[font_id] };

    let v_metrics = font.v_metrics(scale);

    let glyphs: Vec<_> = font
        .layout(text, scale, point(20.0, 20.0 + v_metrics.ascent))
        .collect();

    let (min_x, max_x, min_y, max_y) = {
        let first = glyphs.first().and_then(|g| g.pixel_bounding_box()).unwrap();
        let last = glyphs.last().and_then(|g| g.pixel_bounding_box()).unwrap();

        let (mut min_x, mut max_x) = (first.min.x, last.max.x);
        let (mut min_y, mut max_y) = (first.min.y, last.max.y);

        for g in &glyphs {
            if let Some(bb) = g.pixel_bounding_box() {
                min_x = min_x.min(bb.min.x);
                max_x = max_x.max(bb.max.x);
                min_y = min_y.min(bb.min.y);
                max_y = max_y.max(bb.max.y);
            }
        }

        (min_x, max_x, min_y, max_y)
    };

    let width = (max_x - min_x) as u32;
    let height = (max_y - min_y) as u32;

    let mut image = DynamicImage::new_rgba8(width, height).to_rgba8();

    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                if v != 0. {
                    image.put_pixel(
                        x + (bounding_box.min.x - min_x) as u32,
                        y + (bounding_box.min.y - min_y) as u32,
                        image::Rgba([255, 255, 255, (v * 255.) as u8]),
                    )
                }
            });
        }
    }

    if let Some(id) = upd {
        upd_texture_buffer(id, image.to_vec(), width, height)
    } else {
        add_texture_buffer(image.to_vec(), width, height)
    }
}

#[inline(always)]
fn set_mouse_proj(x: f32, y: f32) {
    unsafe {
        MOUSE_PROJ = vec2(x, y);
    }
}
#[inline(always)]
fn get_mouse_proj(x: f32, y: f32) -> Vec2 {
    //y - half_window.y) * get_mouse_proj().y - get_camera().y,
    unsafe { (vec2(x, y) - get_window() / 2.) * MOUSE_PROJ + get_camera() }
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
