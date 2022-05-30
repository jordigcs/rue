use std::{rc::Rc};

use wgpu::{Surface, Queue, SurfaceConfiguration, Device, RenderPipeline};
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

const VERTICES: &[Vertex] = &[
    
];

pub struct Renderer {
    pub window: Rc<Window>,
    surface: Surface,
    rendering_device: Device,
    render_queue: Queue,
    surface_config: SurfaceConfiguration,
    pub render_config: RenderConfig,
    window_size: PhysicalSize<u32>,
    render_pipeline: RenderPipeline,
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

        let pipeline_shader = device.create_shader_module(&wgpu::include_wgsl!("base_shader.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &pipeline_shader,
                entry_point: "vertex_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &pipeline_shader,
                entry_point: "fragment_main",
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }]
            }),
            primitive: wgpu::PrimitiveState { 
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None, front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Renderer {
            window,
            surface,
            rendering_device: device,
            render_queue: queue,
            surface_config: config,
            render_config,
            window_size,
             render_pipeline,
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
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }
        self.render_queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}