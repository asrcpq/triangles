use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::image::ImageAccess;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::swapchain::PresentInfo;
use vulkano::swapchain::{
	self, AcquireError, SwapchainCreateInfo, SwapchainCreationError,
};
use vulkano::sync::{self, FlushError, GpuFuture};
use winit::event_loop::EventLoopWindowTarget;

use crate::base::Base;
use crate::camera::Camera;
use crate::helper::*;
use crate::model::Model;
use crate::rmod::Rmod;
use crate::teximg::Teximg;
use crate::M4;

pub struct Renderer {
	base: Base,
	rmod: Rmod,
	viewport: Viewport,
	dirty: bool,

	prev: Option<VkwFuture>,
}

impl Renderer {
	pub fn new<E>(el: &EventLoopWindowTarget<E>) -> Self {
		let base = Base::new(el);
		let prev = Some(sync::now(base.device.clone()).boxed());
		let rmod = Rmod::new(base.clone());
		let viewport = Viewport {
			origin: [0.0, 0.0],
			dimensions: [800.0, 600.0],
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

	pub fn get_size(&self) -> [u32; 2] {
		self.base.surface.window().inner_size().into()
	}

	pub fn redraw(&mut self) {
		self.base.surface.window().request_redraw();
	}

	pub fn upload_tex(&mut self, image: Teximg, id: u32) {
		self.rmod.texman.upload(image, id, self.base.queue.clone());
	}

	pub fn remove_tex(&mut self, outer: u32) {
		self.rmod.texman.remove(outer);
	}

	pub fn damage(&mut self) {
		self.dirty = true;
	}

	pub fn render2(&mut self, model: &Model) {
		let [w, h]: [u32; 2] = self.base.surface.window().inner_size().into();
		let [w, h] = [w as f32, h as f32];
		let camera = M4::new_orthographic(0., w, 0., h, 1.0, -1.0);
		self.render(model, camera);
	}

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
			self.base.queue.queue_family_index(),
			CommandBufferUsage::OneTimeSubmit,
		)
		.unwrap();
		self.rmod.build_command(
			&mut builder,
			image_num,
			model,
			Camera {
				data: camera.into(),
			},
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
				PresentInfo {
					index: image_num,
					..PresentInfo::swapchain(self.base.swapchain.clone())
				},
			)
			.then_signal_fence_and_flush();
		match future {
			Ok(future) => {
				self.prev = Some(future.boxed());
			}
			Err(FlushError::OutOfDate) => {
				self.dirty = true;
				self.prev = Some(sync::now(self.base.device.clone()).boxed());
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
