use std::sync::Arc;

use vulkano::{device::{physical::PhysicalDevice, DeviceExtensions, Device, QueueCreateInfo, Queue, DeviceCreateInfo}, swapchain::Surface};
use winit::window::Window;

pub struct Renderer {
    rendering_device: Arc<Device>,
    render_queue: Arc<Queue>,
}

impl Renderer {
    pub fn new(surface: Arc<Surface<Window>>) -> Self {
        let physical = PhysicalDevice::enumerate(surface.instance()).next().expect("No rendering devices available.");
        let queue_family = physical.queue_families().find(|&p| {
            p.supports_graphics()
        }).expect("No rendering devices connected support the Vulkan graphics pipeline.");
        let mut device_extensions = DeviceExtensions::none();
        device_extensions.khr_portability_subset = true;
        let (rendering_device, mut rendering_queues) = Device::new(physical, DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
            enabled_extensions: device_extensions,
            ..Default::default()
        }).expect("Failed to create rendering device.");
        Renderer {
            rendering_device,
            render_queue: rendering_queues.next().unwrap(),
        }
    }
}