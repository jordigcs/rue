use std::sync::Arc;

use vulkano::{device::{physical::PhysicalDevice, DeviceExtensions, Device, QueueCreateInfo, Queue, DeviceCreateInfo, self}, swapchain::Surface, pipeline::{graphics::{viewport::{Viewport, ViewportState}, vertex_input::{BuffersDefinition}, input_assembly::InputAssemblyState}, GraphicsPipeline}, render_pass::{Subpass, self, Framebuffer, RenderPass, FramebufferCreateInfo}, format::Format, image::view::ImageView};
use winit::window::Window;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Default, Copy, Clone, Zeroable, Pod)]
struct Vertex {
    position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position);

mod vertex_shader {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec2 position;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}"
    }
}

mod fragment_shader {
    vulkano_shaders::shader!{
        ty: "fragment",
        src: "
#version 450

layout(location = 0) out vec4 f_color;

void main() {
    f_color = vec4(1.0, 0.0, 0.0, 1.0);
}"
    }
}

pub struct Renderer {
    rendering_device: Arc<Device>,
    render_queue: Arc<Queue>,
    render_pass: Arc<RenderPass>,
    render_pipeline: Arc<GraphicsPipeline>,
}

impl Renderer {
    pub fn new(surface: Arc<Surface<Window>>) -> Self {
        let physical = PhysicalDevice::enumerate(surface.instance()).next().expect("No rendering devices available.");
        let queue_family = physical.queue_families().find(|&p| {
            p.supports_graphics()
        }).expect("No rendering devices connected support the Vulkan graphics pipeline.");
        
        let mut device_extensions = DeviceExtensions::none();
        //device_extensions.khr_portability_subset = true;
        let (rendering_device, mut rendering_queues) = Device::new(physical, DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
            enabled_extensions: device_extensions,
            ..Default::default()
        }).expect("Failed to create rendering device.");

        let render_pass = vulkano::single_pass_renderpass!(rendering_device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: Format::R8G8B8A8_UNORM,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).unwrap();

        let vert_shader = vertex_shader::load(rendering_device.clone()).expect("Failed to create vertex shader.");
        let frag_shader = fragment_shader::load(rendering_device.clone()).expect("Failed to create fragment shader.");

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [surface.window().inner_size().width as f32, surface.window().inner_size().height as f32],
            depth_range: 0.0..1.0
        };

        let pipeline = GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            .vertex_shader(vert_shader.entry_point("main").unwrap(), ())
            .input_assembly_state(InputAssemblyState::new())
            .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
            .fragment_shader(frag_shader.entry_point("main").unwrap(), ())
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(rendering_device.clone())
            .unwrap();
        
        Renderer {
            rendering_device,
            render_queue: rendering_queues.next().unwrap(),
            render_pass,
            render_pipeline: pipeline
        }
    }
    
    pub fn render(&self) {
        let view = ImageView::new_default(ddd);
        let framebuffer = Framebuffer::new(
            self.render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![self.render_pipeline.viewport_state().unwrap().]
            }
        )
    }
}