use super::{
    d2::{upd_proj, CANVAS_UPDATE, RENDERS},
    WINDOW,
};
use crate::{
    engine::{draw, update, Engine, FULLSCREEN},
    info::DEVICE,
};

use glam::vec2;
use pollster::block_on;
use std::{iter::once, sync::Arc};
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{self, Window, WindowAttributes, WindowId},
};

struct WgpuRender {
    window_attributes: WindowAttributes,
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,
    pipeline: Option<wgpu::RenderPipeline>,
    vertex: Option<wgpu::Buffer>,
}
impl WgpuRender {
    pub fn new(title: &str) -> Self {
        Self {
            window_attributes: Window::default_attributes().with_title(title),
            window: None,
            surface: None,
            device: None,
            queue: None,
            config: None,
            pipeline: None,
            vertex: None,
        }
    }
}
impl ApplicationHandler for WgpuRender {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(Arc::new(
            event_loop
                .create_window(self.window_attributes.clone())
                .unwrap(),
        ));

        #[cfg(target_os = "windows")]
        let backends = wgpu::Backends::DX12;

        #[cfg(target_os = "linux")]
        let backends = wgpu::Backends::VULKAN;

        #[cfg(target_os = "android")]
        let backends = wgpu::Backends::GL;

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        let backends = wgpu::Backends::METAL;

        #[cfg(target_arch = "wasm32")]
        let backends = wgpu::Backends::BROWSER_WEBGPU;

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        });

        let window = self.window.as_ref().unwrap();

        let size = window.inner_size();

        unsafe {
            if FULLSCREEN || DEVICE != 0 {
                WINDOW = vec2(size.width as f32, size.height as f32);
            }
        }

        upd_proj();

        let surface = unsafe {
            std::mem::transmute::<wgpu::Surface<'_>, wgpu::Surface<'static>>(
                instance.create_surface(window).unwrap(),
            )
        };

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            // WebGL doesn't support all of wgpu's features, so if
            // we're building for the web, we'll have to disable some.
            required_limits: if cfg!(target_arch = "wasm32") {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            },
            label: None,
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }))
        .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[],                 // 2.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None,     // 6.
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(unsafe { &RENDERS[0].0 }),
            usage: wgpu::BufferUsages::VERTEX,
        });

        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
        self.pipeline = Some(render_pipeline);
        self.vertex = Some(());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match &event {
            WindowEvent::RedrawRequested => {
                self.window.as_mut().unwrap().request_redraw();

                let device = self.device.as_mut().unwrap();
                let queue = self.queue.as_mut().unwrap();

                update();
                draw();

                let output = self
                    .surface
                    .as_mut()
                    .unwrap()
                    .get_current_texture()
                    .unwrap();
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });

                    render_pass.set_pipeline(self.pipeline.as_ref().unwrap());
                    render_pass.draw(0..3, 0..1);
                }

                queue.submit(once(encoder.finish()));
                output.present();
            }
            WindowEvent::Resized(size) => {
                unsafe {
                    WINDOW = vec2(size.width as f32, size.height as f32);
                    CANVAS_UPDATE = true;
                }

                let config = self.config.as_mut().unwrap();
                let surface = self.surface.as_mut().unwrap();
                let device = self.device.as_ref().unwrap();

                config.width = size.width;
                config.height = size.height;

                surface.configure(&device, &config);
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }
}

pub(crate) fn render(name: &str) {
    let event_loop = EventLoop::new().unwrap();

    let mut window_state = WgpuRender::new(name);
    let _ = event_loop.run_app(&mut window_state);
}
