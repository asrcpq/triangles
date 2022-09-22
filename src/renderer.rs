use vulkano::sync::{self, GpuFuture, FlushError};
use vulkano::image::ImageAccess;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::swapchain::{
	self, AcquireError, SwapchainCreateInfo, SwapchainCreationError,
};
use winit::event_loop::EventLoopWindowTarget;

use crate::helper::*;
use crate::base::Base;
use crate::rmod::Rmod;
use crate::model::Model;
use crate::M4;
use crate::camera::Camera;
use crate::TexImage;

pub struct Renderer {
	base: Base,
	rmod: Rmod,
	viewport: Viewport,
	dirty: bool,

	prev: Option<VkwFuture>,
}

impl Renderer {
	pub fn new<E>(
		images: Vec<TexImage>,
		el: &EventLoopWindowTarget<E>,
		window_size: [u32; 2],
	) -> Self {
		let base = Base::new(el, window_size);
		let prev = Some(sync::now(base.device.clone()).boxed());
		let rmod = Rmod::new(base.clone(), images);
		let viewport = Viewport {
			origin: [0.0, 0.0],
			dimensions: [window_size[0] as f32, window_size[1] as f32],
			depth_range: 0.0..1.0,
		};

		Self {
			base,
			rmod,
			prev,
			viewport,
			dirty: false,
		}
	}

	pub fn damage(&mut self) { self.dirty = true; }

	pub fn render(&mut self, model: &Model, camera: M4) {
		self.prev.as_mut().unwrap().cleanup_finished();
		if self.dirty {
			self.create_swapchain();
			self.dirty = false;
		}
		let (image_num, _, acquire_future) = match swapchain::acquire_next_image(
			self.base.swapchain.clone(),
			None,
		) {
			Ok(r) => r,
			Err(AcquireError::OutOfDate) => {
				self.dirty = true;
				return;
			}
			Err(e) => panic!("{:?}", e),
		};
		let mut builder = AutoCommandBufferBuilder::primary(
			self.base.device.clone(),
			self.base.queue.family(),
			CommandBufferUsage::OneTimeSubmit,
		).unwrap();
		self.rmod.build_command(
			&mut builder,
			image_num,
			model,
			Camera {data: camera.into()},
			self.viewport.clone(),
		);
		let command_buffer = Box::new(builder.build().unwrap());

		let future = self
			.prev
			.take()
			.unwrap()
			.join(acquire_future)
			.then_execute(self.base.queue.clone(), command_buffer)
			.unwrap()
			.then_swapchain_present(
				self.base.queue.clone(),
				self.base.swapchain.clone(),
				image_num,
			)
			.then_signal_fence_and_flush();
		match future {
			Ok(future) => {
				self.prev = Some(future.boxed());
			}
			Err(FlushError::OutOfDate) => {
				self.dirty = true;
				self.prev =
					Some(sync::now(self.base.device.clone()).boxed());
			}
			Err(e) => {
				println!("Failed to flush future: {:?}", e);
				self.prev = Some(sync::now(self.base.device.clone()).boxed());
			}
		}
	}

	fn create_swapchain(&mut self) {
		eprintln!("Recreate swapchain");
		let dimensions: [u32; 2] =
			self.base.surface.window().inner_size().into();
		let swapchain = self.base.swapchain.clone();
		let (new_swapchain, new_images) =
			match swapchain.recreate(SwapchainCreateInfo {
				image_extent: dimensions,
				..swapchain.create_info()
			}) {
				Ok(r) => r,
				Err(SwapchainCreationError::ImageExtentNotSupported {
					..
				}) => {
					eprintln!("Error: unsupported dimensions");
					return;
				}
				Err(e) => {
					panic!("Failed to recreate swapchain: {:?}", e)
				}
			};
		self.base.swapchain = new_swapchain;

		let dimensions = new_images[0].dimensions().width_height();
		self.viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];
		self.rmod.update_framebuffers(&new_images);
		self.base.images = new_images;
	}
}
