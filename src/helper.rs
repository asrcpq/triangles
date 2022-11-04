use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
	AutoCommandBufferBuilder, PrimaryAutoCommandBuffer,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, Queue};
use vulkano::image::view::ImageView;
use vulkano::image::{ImmutableImage, SwapchainImage};
use vulkano::instance::Instance;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{Framebuffer, RenderPass};
use vulkano::swapchain::{Surface, Swapchain};
use vulkano::sync::GpuFuture;

pub type VkwCommandBuilder = AutoCommandBufferBuilder<
	PrimaryAutoCommandBuffer,
	Arc<StandardCommandBufferAllocator>,
>;
pub type VkwPhysicalDevice = Arc<PhysicalDevice>;
pub type VkwDevice = Arc<Device>;
pub type VkwFramebuffer = Arc<Framebuffer>;
pub type VkwFuture = Box<dyn GpuFuture>;
pub type VkwImageView = Arc<ImageView<ImmutableImage>>;
pub type VkwImages = Vec<Arc<SwapchainImage>>;
pub type VkwInstance = Arc<Instance>;
pub type VkwPipeline = Arc<GraphicsPipeline>;
pub type VkwQueue = Arc<Queue>;
pub type VkwRenderPass = Arc<RenderPass>;
pub type VkwSurface = Arc<Surface>;
pub type VkwSwapchain = Arc<Swapchain>;
pub type VkwTextureSet = Arc<PersistentDescriptorSet>;
pub type VkwTexLayout = Arc<DescriptorSetLayout>;
pub type VkwMemAlloc = Arc<StandardMemoryAllocator>;
pub type VkwDstAlloc = Arc<StandardDescriptorSetAllocator>;
pub type VkwComAlloc = Arc<StandardCommandBufferAllocator>;
