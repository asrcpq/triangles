use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano_win::VkSurfaceBuild;
use vulkano::VulkanLibrary;
use winit::dpi::{LogicalSize, Size};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder};

use crate::helper::*;

#[derive(Clone)]
pub struct Base {
	pub device: VkwDevice,
	pub queue: VkwQueue,
	pub surface: VkwSurface<Window>,
	pub swapchain: VkwSwapchain<Window>,
	pub images: VkwImages,
}

fn winit_size(size: [u32; 2]) -> Size {
	Size::new(LogicalSize::new(size[0], size[1]))
}

impl Base {
	pub fn new<E>(
		el: &EventLoopWindowTarget<E>,
	) -> Self {
		let library = VulkanLibrary::new().unwrap();
		let required_extensions = vulkano_win::required_extensions(&library);

		let instance = Instance::new(
			library,
			InstanceCreateInfo {
				enabled_extensions: required_extensions,
				// Enable enumerating devices that use non-conformant vulkan implementations. (ex. MoltenVK)
				enumerate_portability: true,
				..Default::default()
			},
		).unwrap();
		let surface = WindowBuilder::new()
			.with_inner_size(winit_size([800, 600]))
			//.with_resizable(false)
			.build_vk_surface(el, instance.clone())
			.unwrap();

		let (physical_device, device, queue) =
			get_device_and_queue(&instance, surface.clone());

		let (swapchain, images) = get_swapchain_and_images(
			physical_device,
			device.clone(),
			surface.clone(),
		);
		Self {
			device,
			queue,
			surface,
			swapchain,
			images,
		}
	}
}
