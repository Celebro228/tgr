use super::{WINDOW, d2::{upd_proj, CANVAS_UPDATE}};
use crate::{engine::{FULLSCREEN}, info::DEVICE};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{self, Window, WindowAttributes, WindowId},
};
use wgpu::{
    Backends,
    Instance,
    RequestAdapterOptions,
    PowerPreference,
    MemoryHints,
    Trace,
    Surface,
    SurfaceConfiguration,
    TextureUsages,
    Device,
    Queue,
};
use pollster::block_on;
use std::sync::Arc;
use glam::vec2;


struct WgpuRender {
    window_attributes: WindowAttributes,
    window: Option<Arc<Window>>,
    surface: Option<Surface<'static>>,
    device: Option<Device>,
    queue: Option<Queue>,
    config: Option<SurfaceConfiguration>,
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
        }
    }
}
impl ApplicationHandler for WgpuRender {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            self.window = Some(Arc::new(event_loop.create_window(self.window_attributes.clone()).unwrap()));

            #[cfg(target_os = "windows")]
            let backends = Backends::DX12;

            #[cfg(target_os = "linux")]
            let backends = Backends::VULKAN;

            #[cfg(target_os = "android")]
            let backends = Backends::GL;

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            let backends = Backends::METAL;

            #[cfg(target_arch = "wasm32")]
            let backends = Backends::BROWSER_WEBGPU;

            let instance = Instance::new(&wgpu::InstanceDescriptor {
                backends,
                ..Default::default()
            });

            if let Some(ref window) = &self.window {
                let size = window.inner_size();

                unsafe {
                    if FULLSCREEN || DEVICE != 0 {
                        WINDOW = vec2(size.width as f32, size.height as f32);
                    }
                }

                upd_proj();

                let surface = unsafe {
                    std::mem::transmute::<wgpu::Surface<'_>, wgpu::Surface<'static>>(
                        instance.create_surface(window).unwrap()
                    )
                };
                
                let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
                    power_preference: PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })).unwrap();

                let (device, queue) = block_on(adapter.request_device(
                    &wgpu::DeviceDescriptor {
                        required_features: wgpu::Features::empty(),
                        // WebGL doesn't support all of wgpu's features, so if
                        // we're building for the web, we'll have to disable some.
                        required_limits: if cfg!(target_arch = "wasm32") {
                            wgpu::Limits::downlevel_webgl2_defaults()
                        } else {
                            wgpu::Limits::default()
                        },
                        label: None,
                        memory_hints: MemoryHints::Performance,
                        trace: Trace::Off,
                    },
                )).unwrap();

                let surface_caps = surface.get_capabilities(&adapter);

                let surface_format = surface_caps.formats.iter()
                    .find(|f| f.is_srgb())
                    .copied()
                    .unwrap_or(surface_caps.formats[0]);

                let config = SurfaceConfiguration {
                    usage: TextureUsages::RENDER_ATTACHMENT,
                    format: surface_format,
                    width: size.width,
                    height: size.height,
                    present_mode: surface_caps.present_modes[0],
                    alpha_mode: surface_caps.alpha_modes[0],
                    view_formats: vec![],
                    desired_maximum_frame_latency: 2,
                };

                self.surface = Some(surface);
                self.device = Some(device);
                self.queue = Some(queue);
                self.config = Some(config)
            }
    }
    
    fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            _: WindowId,
            event: WindowEvent,
        ) {
        match &event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                unsafe {
                    WINDOW = vec2(size.width as f32, size.height as f32);
                    CANVAS_UPDATE = true;
                }

                if let Some(config) = &mut self.config {
                    config.width = size.width;
                    config.height = size.height;
                    if let Some(device) = &self.device {
                        if let Some(surface) = &mut self.surface {
                            surface.configure(&device, &config);
                        }
                    }
                }

                
            }
            WindowEvent::RedrawRequested => {
                //TODO: Add flag for checking if surface has been configured
                if let Some(ref window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn render(name: &str) {
    let event_loop = EventLoop::new().unwrap();

    let mut window_state = WgpuRender::new(name);
    let _ = event_loop.run_app(&mut window_state);
}