use std::sync::Arc;
use vulkano::command_buffer::{
	AutoCommandBufferBuilder, PrimaryAutoCommandBuffer,
};
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{
	Device, DeviceCreateInfo, DeviceExtensions, Features, Queue,
	QueueCreateInfo,
};
use vulkano::image::view::ImageView;
use vulkano::image::{ImageUsage, ImmutableImage, SwapchainImage};
use vulkano::instance::Instance;
use vulkano::pipeline::graphics::input_assembly::{
	InputAssemblyState, PrimitiveTopology,
};
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{
	Framebuffer, FramebufferCreateInfo, RenderPass, Subpass,
};
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo};
use vulkano::sync::GpuFuture;
use winit::window::Window;

use crate::shader;
use crate::vertex::{VertexTex, VertexSolid};

pub type VkwCommandBuilder = AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>;
pub type VkwDevice = Arc<Device>;
pub type VkwFramebuffer = Arc<Framebuffer>;
pub type VkwFuture = Box<dyn GpuFuture>;
pub type VkwImageView = Arc<ImageView<ImmutableImage>>;
pub type VkwImages = Vec<Arc<SwapchainImage<Window>>>;
pub type VkwInstance = Arc<Instance>;
pub type VkwPipeline = Arc<GraphicsPipeline>;
pub type VkwQueue = Arc<Queue>;
pub type VkwRenderPass = Arc<RenderPass>;
pub type VkwSurface<W> = Arc<Surface<W>>;
pub type VkwSwapchain<W> = Arc<Swapchain<W>>;
pub type VkwTextureSet = Arc<PersistentDescriptorSet>;
pub type VkwTexLayout = Arc<DescriptorSetLayout>;

pub fn get_device_and_queue<W>(
	instance: &VkwInstance,
	surface: VkwSurface<W>,
) -> (PhysicalDevice, VkwDevice, VkwQueue) {
	let device_extensions = DeviceExtensions {
		khr_swapchain: true,
		..DeviceExtensions::none()
	};
	let (physical_device, queue_family) = PhysicalDevice::enumerate(instance)
		.filter(|&p| {
			p.supported_extensions().is_superset_of(&device_extensions)
		})
		.filter_map(|p| {
			p.queue_families()
				.find(|&q| {
					q.supports_graphics()
						&& q.supports_surface(&surface).unwrap_or(false)
				})
				.map(|q| (p, q))
		})
		.min_by_key(|(p, _)| match p.properties().device_type {
			PhysicalDeviceType::DiscreteGpu => 0,
			PhysicalDeviceType::IntegratedGpu => 1,
			PhysicalDeviceType::VirtualGpu => 2,
			PhysicalDeviceType::Cpu => 3,
			PhysicalDeviceType::Other => 4,
		})
		.unwrap();

	println!(
		"Using device: {} (type: {:?})",
		physical_device.properties().device_name,
		physical_device.properties().device_type,
	);

	let (device, mut queues) = Device::new(
		physical_device,
		DeviceCreateInfo {
			enabled_extensions: device_extensions,
			enabled_features: Features {
				fill_mode_non_solid: true,
				..Features::none()
			},
			queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
			..Default::default()
		},
	)
	.unwrap();

	let queue = queues.next().unwrap();

	(physical_device, device, queue)
}

pub fn get_swapchain_and_images(
	physical_device: PhysicalDevice,
	device: VkwDevice,
	surface: VkwSurface<Window>,
) -> (VkwSwapchain<Window>, VkwImages) {
	let caps = physical_device
		.surface_capabilities(&surface, Default::default())
		.unwrap();
	let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
	let format = physical_device
		.surface_formats(&surface, Default::default())
		.unwrap()[0]
		.0;
	let format = Some(format);
	let dimensions: [u32; 2] = surface.window().inner_size().into();

	Swapchain::new(
		device,
		surface,
		SwapchainCreateInfo {
			min_image_count: caps.min_image_count,
			image_format: format,
			image_extent: dimensions,
			image_usage: ImageUsage::color_attachment(),
			composite_alpha,
			..Default::default()
		},
	)
	.unwrap()
}

pub fn get_render_pass_clear<W>(
	device: VkwDevice,
	swapchain: VkwSwapchain<W>,
) -> VkwRenderPass {
	vulkano::single_pass_renderpass!(
		device,
		attachments: {
			color: {
				load: Clear,
				store: Store,
				format: swapchain.image_format(),
				samples: 1,
			}
		},
		pass: {
			color: [color],
			depth_stencil: {}
		}
	)
	.unwrap()
}

pub fn get_render_pass_load<W>(
	device: VkwDevice,
	swapchain: VkwSwapchain<W>,
) -> VkwRenderPass {
	vulkano::single_pass_renderpass!(
		device,
		attachments: {
			color: {
				load: Load,
				store: Store,
				format: swapchain.image_format(),
				samples: 1,
			}
		},
		pass: {
			color: [color],
			depth_stencil: {}
		}
	)
	.unwrap()
}

pub fn get_pipeline_solid (
	render_pass: VkwRenderPass,
	device: VkwDevice,
) -> VkwPipeline {
	let vs = shader::vs_solid::load(device.clone()).unwrap();
	let fs = shader::fs_solid::load(device.clone()).unwrap();
	let pipeline = GraphicsPipeline::start()
		.vertex_input_state(BuffersDefinition::new().vertex::<VertexSolid>())
		.vertex_shader(vs.entry_point("main").unwrap(), ())
		.input_assembly_state(
			InputAssemblyState::new().topology(PrimitiveTopology::TriangleList),
		)
		.viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
		.fragment_shader(fs.entry_point("main").unwrap(), ())
		.render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
		.build(device.clone())
		.unwrap();
	pipeline
}

pub fn get_pipeline_tex (
	render_pass: VkwRenderPass,
	device: VkwDevice,
) -> VkwPipeline {
	let vs = shader::vs_tex::load(device.clone()).unwrap();
	let fs = shader::fs_tex::load(device.clone()).unwrap();
	let pipeline = GraphicsPipeline::start()
		.vertex_input_state(BuffersDefinition::new().vertex::<VertexTex>())
		.vertex_shader(vs.entry_point("main").unwrap(), ())
		.input_assembly_state(
			InputAssemblyState::new().topology(PrimitiveTopology::TriangleList),
		)
		.viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
		.fragment_shader(fs.entry_point("main").unwrap(), ())
		.render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
		.build(device.clone())
		.unwrap();
	pipeline
}

pub fn window_size_dependent_setup(
	render_pass: VkwRenderPass,
	images: &VkwImages,
) -> Vec<VkwFramebuffer> {
	images
		.iter()
		.map(|image| {
			let view = ImageView::new_default(image.clone()).unwrap();
			Framebuffer::new(
				render_pass.clone(),
				FramebufferCreateInfo {
					attachments: vec![view],
					..Default::default()
				},
			)
			.unwrap()
		})
		.collect::<Vec<_>>()
}
