use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::{
	Device, DeviceCreateInfo, DeviceExtensions, Features, QueueCreateInfo,
};
use vulkano::image::ImageUsage;
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceExtensions};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::swapchain::{Swapchain, SwapchainCreateInfo};
use vulkano::VulkanLibrary;
use vulkano_win::VkSurfaceBuild;
use winit::dpi::{LogicalSize, Size};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder};

use crate::helper::*;

#[derive(Clone)]
pub struct Base {
	pub instance: VkwInstance,
	pub device: VkwDevice,
	pub queue: VkwQueue,
	pub surface: VkwSurface,
	pub swapchain: VkwSwapchain,
	pub images: VkwImages,
	pub memalloc: VkwMemAlloc,
	pub dstalloc: VkwDstAlloc,
	pub comalloc: VkwComAlloc,
}

fn winit_size(size: [u32; 2]) -> Size {
	Size::new(LogicalSize::new(size[0], size[1]))
}

impl Base {
	pub fn new<E>(el: &EventLoopWindowTarget<E>) -> Self {
		let library = VulkanLibrary::new().unwrap();
		let required_extensions = vulkano_win::required_extensions(&library);
		let extensions = InstanceExtensions {
			ext_debug_utils: true,
			..InstanceExtensions::empty()
		};

		let layers = vec!["VK_LAYER_KHRONOS_validation".to_owned()];
		let instance = Instance::new(
			library,
			InstanceCreateInfo {
				enabled_extensions: required_extensions & extensions,
				enabled_layers: layers,
				..Default::default()
			},
		)
		.unwrap();
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
		let (memalloc, dstalloc, comalloc) = get_allocators(device.clone());
		Self {
			instance,
			device,
			queue,
			surface,
			swapchain,
			images,
			memalloc,
			dstalloc,
			comalloc,
		}
	}
}

pub fn get_allocators(
	device: VkwDevice,
) -> (VkwMemAlloc, VkwDstAlloc, VkwComAlloc) {
	let memalloc = StandardMemoryAllocator::new_default(device.clone());
	let dstalloc = StandardDescriptorSetAllocator::new(device.clone());
	let comalloc =
		StandardCommandBufferAllocator::new(device, Default::default());
	(Arc::new(memalloc), Arc::new(dstalloc), Arc::new(comalloc))
}

pub fn get_device_and_queue(
	instance: &VkwInstance,
	surface: VkwSurface,
) -> (VkwPhysicalDevice, VkwDevice, VkwQueue) {
	let device_extensions = DeviceExtensions {
		khr_swapchain: true,
		..DeviceExtensions::empty()
	};

	let features = Features {
		descriptor_binding_variable_descriptor_count: true,
		descriptor_indexing: true,
		shader_uniform_buffer_array_non_uniform_indexing: true,
		runtime_descriptor_array: true,
		..Features::empty()
	};

	let (physical_device, queue_family_index) = instance
		.enumerate_physical_devices()
		.unwrap()
		.filter(|p| p.supported_extensions().contains(&device_extensions))
		.filter(|p| p.supported_features().contains(&features))
		.filter_map(|p| {
			p.queue_family_properties()
				.iter()
				.enumerate()
				.position(|(i, q)| {
					q.queue_flags.graphics
						&& p.surface_support(i as u32, &surface)
							.unwrap_or(false)
				})
				.map(|i| (p, i as u32))
		})
		.min_by_key(|(p, _)| match p.properties().device_type {
			PhysicalDeviceType::DiscreteGpu => 0,
			PhysicalDeviceType::IntegratedGpu => 1,
			PhysicalDeviceType::VirtualGpu => 2,
			PhysicalDeviceType::Cpu => 3,
			PhysicalDeviceType::Other => 4,
			_ => 5,
		})
		.expect("No suitable physical device found");

	println!(
		"Using device: {} (type: {:?})",
		physical_device.properties().device_name,
		physical_device.properties().device_type,
	);

	let (device, mut queues) = Device::new(
		physical_device.clone(),
		DeviceCreateInfo {
			enabled_extensions: device_extensions,
			enabled_features: features,
			queue_create_infos: vec![QueueCreateInfo {
				queue_family_index,
				..Default::default()
			}],

			..Default::default()
		},
	)
	.unwrap();
	let queue = queues.next().unwrap();

	(physical_device, device, queue)
}

pub fn get_swapchain_and_images(
	physical_device: VkwPhysicalDevice,
	device: VkwDevice,
	surface: VkwSurface,
) -> (VkwSwapchain, VkwImages) {
	let caps = physical_device
		.surface_capabilities(&surface, Default::default())
		.unwrap();
	let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
	let format = physical_device
		.surface_formats(&surface, Default::default())
		.unwrap()[0]
		.0;
	let format = Some(format);
	let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();
	let dimensions: [u32; 2] = window.inner_size().into();

	Swapchain::new(
		device,
		surface,
		SwapchainCreateInfo {
			min_image_count: caps.min_image_count,
			image_format: format,
			image_extent: dimensions,
			image_usage: ImageUsage {
				color_attachment: true,
				..ImageUsage::empty()
			},
			composite_alpha,
			..Default::default()
		},
	)
	.unwrap()
}
