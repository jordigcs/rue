use std::{rc::Rc, cell::RefCell};

use cgmath::{Vector2, Vector3};
use wgpu::{Surface, Queue, SurfaceConfiguration, Device, RenderPipeline, util::DeviceExt, Buffer, BufferUsages};
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
    position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    pub const fn new(pos:[f32; 2], color: [f32; 3]) -> Self {
        Vertex { position: [pos[0], pos[1], 0.0], color }
    }

    pub const fn new_with_rue_color(pos: [f32; 2], color: crate::helpers::colors::Color) -> Self {
        Vertex {
            position: [pos[0], pos[1], 0.0],
            color: [color.r as f32, color.g as f32, color.b as f32],
        }
    }

    pub fn get_position(&self) -> [f32; 2] {
        [self.position[0], self.position[1]]
    }

    pub fn buffer_descriptor<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, // Shader will skip this amount of bytes to get to the next vertex when iterating
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ],
        }
    }
}

pub const SQUARE_VERTICES: &[Vertex] = &[
    Vertex::new([-0.5, -0.5], [0.0, 0.0, 0.0]), // 0
    Vertex::new([0.5, -0.5], [0.0, 5.0, 0.0]), // 1
    Vertex::new([-0.5, 0.5], [0.0, 0.0, 1.0]), // 2
    Vertex::new([0.5, 0.5], [1.0, 0.2, 0.5]), // 3
];

pub const SQUARE_INDICES: &[u16] = &[
    0, 1, 2, 2, 1, 3
];


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderableInstanceRaw {
    model: [[f32; 4]; 4]
}

impl RenderableInstanceRaw {
    fn buffer_descriptor<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<RenderableInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                }
            ],
        }
    }
}

pub struct RenderableInstance {
    pub position: Vector2<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl RenderableInstance {
    pub fn to_raw(&self) -> RenderableInstanceRaw {
        RenderableInstanceRaw {
            model: (cgmath::Matrix4::from_translation(Vector3::new(self.position.x, self.position.y, 0.0)) * cgmath::Matrix4::from(self.rotation)).into(),
        }
    }
}

pub struct Renderer {
    pub window: Rc<Window>,
    surface: Surface,
    rendering_device: Device,
    render_queue: Queue,
    surface_config: SurfaceConfiguration,
    pub render_config: Rc<RefCell<RenderConfig>>,
    window_size: PhysicalSize<u32>,
    render_pipeline: RenderPipeline,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub num_vertices: u32,
    pub num_indices: u32,
}

impl Renderer {
    pub async fn new(window: Rc<Window>, render_config:Option<Rc<RefCell<RenderConfig>>>) -> Self {
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
                buffers: &[
                    Vertex::buffer_descriptor(),
                    RenderableInstanceRaw::buffer_descriptor(),
                ],
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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(SQUARE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(SQUARE_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let r_config = render_config.unwrap_or_else(|| Rc::new(RefCell::new(RenderConfig::default())));

        Renderer {
            window,
            surface,
            rendering_device: device,
            render_queue: queue,
            surface_config: config,
            render_config: r_config,
            window_size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_vertices: SQUARE_VERTICES.len() as u32,
            num_indices: SQUARE_INDICES.len() as u32,
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
    
    pub fn render(&mut self, renderable_instances:Vec<RenderableInstance>) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.rendering_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        let renderable_data = renderable_instances.iter().map(RenderableInstance::to_raw).collect::<Vec<_>>();
        let instance_buffer = self.rendering_device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instances"),
            contents: bytemuck::cast_slice(&renderable_data),
            usage: BufferUsages::VERTEX,
        });
        { // Define render pass in new scope because begin_render_pass borrows encoder, which we need later to submit the encoder info to render_queue
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(helpers::colors::color_to_wgpu_color(self.render_config.borrow().clear_color)),
                            store: true,
                        },
                    }
                ],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..renderable_instances.len() as _);
        }
        self.render_queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

// Renderable trait & impl
pub struct RenderData {
    pub vertex_buffer: Buffer,
    pub index_buffer: Option<Buffer>,
}

pub trait Renderable {
    fn prepare_for_render(&self) -> RenderData;
}
