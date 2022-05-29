use std::{rc::Rc};

use wgpu::{Surface, Queue, SurfaceConfiguration, Device};
use winit::{window::Window, dpi::PhysicalSize};

use crate::helpers::{colors::Color, self};

pub struct RenderConfig {
    pub clear_color: Color,
}

impl Default for RenderConfig {
    fn default() -> Self {
        RenderConfig {
            clear_color: Color::default()
        }
    }
}

pub struct Renderer {
    pub window: Rc<Window>,
    surface: Surface,
    rendering_device: Device,
    render_queue: Queue,
    surface_config: SurfaceConfiguration,
    pub render_config: RenderConfig,
    window_size: PhysicalSize<u32>,
}

impl Renderer {
    pub async fn new(window: Rc<Window>, render_config:RenderConfig) -> Self {
        let window_size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&*window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            }, None).await.unwrap();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        surface.configure(&device, &config);
        Renderer {
            window,
            surface,
            rendering_device: device,
            render_queue: queue,
            surface_config: config,
            render_config,
            window_size
        }
    }
    
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.window_size = self.window.inner_size();
            self.surface_config.width = self.window_size.width;
            self.surface_config.height = self.window_size.height;
            self.surface.configure(&self.rendering_device, &self.surface_config);
        }
    }
    
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.rendering_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        { // Define render pass in new scope because begin_render_pass borrows encoder, which we need later to submit the encoder info to render_queue
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(helpers::colors::color_to_wgpu_color(self.render_config.clear_color)),
                            store: true,
                        },
                    }
                ],
                depth_stencil_attachment: None,
            });
        }
        self.render_queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}