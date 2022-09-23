use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::command_buffer::{RenderPassBeginInfo, SubpassContents};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::{Pipeline, PipelineBindPoint};

use crate::base::Base;
use crate::vertex::{VertexTex, VertexSolid};
use crate::helper::*;
use crate::model::Model;
use crate::camera::Camera;
use crate::texman::Texman;

type VertexSolidBuffer = Arc<CpuAccessibleBuffer<[VertexSolid]>>;
type VertexTexBuffer = Arc<CpuAccessibleBuffer<[VertexTex]>>;

pub struct Rmod {
	base: Base,
	framebuffers_solid: Vec<VkwFramebuffer>,
	framebuffers_tex: Vec<VkwFramebuffer>,
	pipeline_solid: VkwPipeline,
	pipeline_tex: VkwPipeline,
	renderpass_solid: VkwRenderPass,
	renderpass_tex: VkwRenderPass,
	pub texman: Texman,
	texset: Option<VkwTextureSet>,
}


impl Rmod {
	pub fn new(
		base: Base,
	) -> Self {
		let renderpass_solid =
			get_render_pass_clear(base.device.clone(), base.swapchain.clone());
		let renderpass_tex =
			get_render_pass_load(base.device.clone(), base.swapchain.clone());
		let pipeline_solid = get_pipeline_solid(renderpass_solid.clone(), base.device.clone());
		let pipeline_tex = get_pipeline_tex(renderpass_tex.clone(), base.device.clone(), 0);

		let framebuffers_solid = window_size_dependent_setup(renderpass_solid.clone(), &base.images);
		let framebuffers_tex = window_size_dependent_setup(renderpass_tex.clone(), &base.images);
		Self {
			base,
			framebuffers_solid,
			framebuffers_tex,
			pipeline_solid,
			pipeline_tex,
			renderpass_solid,
			renderpass_tex,
			texman: Default::default(),
			texset: None,
		}
	}

	fn generate_solid_vertex_buffer(
		&self,
		model: &Model,
	) -> Option<VertexSolidBuffer> {
		let vertices = model.solid_faces
			.iter()
			.flat_map(|face| {
				(0..3).map(|i| VertexSolid {
					pos: model.vs[face.vid[i]],
					rgba: face.rgba,
				})
			}).collect::<Vec<_>>();
		if vertices.is_empty() {
			return None
		}
		let result = CpuAccessibleBuffer::from_iter(
			self.base.device.clone(),
			BufferUsage {
				vertex_buffer: true,
				..BufferUsage::empty()
			},
			false,
			vertices.into_iter(),
		).unwrap();
		Some(result)
	}

	fn generate_tex_vertex_buffer(
		&self,
		model: &Model,
	) -> Option<VertexTexBuffer> {
		let vertices = model.tex_faces
			.iter()
			.flat_map(|face| {
				(0..3).map(|i| VertexTex {
					pos: model.vs[face.vid[i]],
					tex_coord: model.uvs[face.uvid[i]],
					tex_layer: face.layer as i32,
				})
			}).collect::<Vec<_>>();
		if vertices.is_empty() {
			return None
		}
		let result = CpuAccessibleBuffer::from_iter(
			self.base.device.clone(),
			BufferUsage {
				vertex_buffer: true,
				..BufferUsage::empty()
			},
			false,
			vertices.into_iter(),
		).unwrap();
		Some(result)
	}

	pub fn build_command(
		&mut self,
		builder: &mut VkwCommandBuilder,
		image_num: usize,
		model: &Model,
		camera: Camera,
		viewport: Viewport,
	) {
		let uniform_buffer = CpuAccessibleBuffer::from_data(
			self.base.device.clone(),
			BufferUsage {
				uniform_buffer: true,
				..BufferUsage::empty()
			},
			false,
			camera,
		)
		.unwrap();

		let layout = self.pipeline_solid.layout().set_layouts().get(0).unwrap();
		let set = PersistentDescriptorSet::new(
			layout.clone(),
			[WriteDescriptorSet::buffer(0, uniform_buffer)],
		).unwrap();

		let vertex_buffer = self.generate_solid_vertex_buffer(&model);
		if let Some(vertex_buffer) = vertex_buffer {
			let clear_values = vec![Some([0.0, 0.0, 0.0, 1.0].into())];
			builder.begin_render_pass(
					RenderPassBeginInfo {
						clear_values,
						..RenderPassBeginInfo::framebuffer(self.framebuffers_solid[image_num].clone())
					},
					SubpassContents::Inline,
				)
				.unwrap()
				.set_viewport(0, [viewport.clone()]);
			builder.bind_pipeline_graphics(self.pipeline_solid.clone());
			builder.bind_descriptor_sets(
				PipelineBindPoint::Graphics,
				self.pipeline_solid.layout().clone(),
				0,
				vec![set.clone()],
			);
			let buflen = vertex_buffer.len();
			builder.bind_vertex_buffers(0, vertex_buffer)
				.draw(buflen as u32, 1, 0, 0)
				.unwrap();
			builder.end_render_pass().unwrap();
		}

		let vertex_buffer = self.generate_tex_vertex_buffer(&model);
		if let Some(vertex_buffer) = vertex_buffer {
			if self.texman.get_dirty() {
				let tex_len = self.texman.tex_len();
				self.pipeline_tex = get_pipeline_tex(
					self.renderpass_tex.clone(),
					self.base.device.clone(),
					tex_len as u32,
				);
				let layout = self.pipeline_tex.layout().set_layouts().get(1).unwrap();
				let texset = self.texman.compile_set(
					self.base.device.clone(),
					layout.clone(),
				);
				self.texset = texset;
			}
			if let Some(texset) = self.texset.clone() {
				let clear_values = vec![None];
				builder
					.begin_render_pass(
						RenderPassBeginInfo {
							clear_values,
							..RenderPassBeginInfo::framebuffer(self.framebuffers_tex[image_num].clone())
						},
						SubpassContents::Inline,
					)
					.unwrap()
					.set_viewport(0, [viewport]);
				builder.bind_descriptor_sets(
					PipelineBindPoint::Graphics,
					self.pipeline_tex.layout().clone(),
					0,
					vec![set, texset],
				);
				builder.bind_pipeline_graphics(self.pipeline_tex.clone());
				let buflen = vertex_buffer.len();
				builder
					.bind_vertex_buffers(0, vertex_buffer)
					.draw(buflen as u32, 1, 0, 0)
					.unwrap();
				builder.end_render_pass().unwrap();
			} else {
				eprintln!("ERROR: Texture set is empty, but vertex buffer is non-empty");
			};
		}
	}

	pub fn update_framebuffers(&mut self, images: &VkwImages) {
		self.framebuffers_solid =
			window_size_dependent_setup(self.renderpass_solid.clone(), images);
		self.framebuffers_tex =
			window_size_dependent_setup(self.renderpass_tex.clone(), images);
	}
}

