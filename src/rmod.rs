use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::command_buffer::{RenderPassBeginInfo, SubpassContents};
use vulkano::descriptor_set::layout::{
	DescriptorSetLayout, DescriptorSetLayoutCreateInfo,
	DescriptorSetLayoutCreationError, DescriptorType,
};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::image::view::ImageView;
use vulkano::pipeline::graphics::input_assembly::{
	InputAssemblyState, PrimitiveTopology,
};
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::layout::{PipelineLayout, PipelineLayoutCreateInfo};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::{Pipeline, PipelineBindPoint};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, Subpass};

use crate::base::Base;
use crate::camera::Camera;
use crate::helper::*;
use crate::model::Model;
use crate::shader;
use crate::texman::Texman;
use crate::vertex::VertexTex;

type VertexTexBuffer = Arc<CpuAccessibleBuffer<[VertexTex]>>;

pub struct Rmod {
	base: Base,
	framebuffers_tex: Vec<VkwFramebuffer>,
	pipeline_tex: VkwPipeline,
	renderpass_tex: VkwRenderPass,
	pub texman: Texman,
	texset: Option<VkwTextureSet>,
}

impl Rmod {
	pub fn new(base: Base) -> Self {
		let renderpass_tex =
			get_render_pass_clear(base.device.clone(), base.swapchain.clone());
		let pipeline_tex =
			get_pipeline_tex(renderpass_tex.clone(), base.device.clone(), 0);
		let framebuffers_tex =
			window_size_dependent_setup(renderpass_tex.clone(), &base.images);
		Self {
			base,
			framebuffers_tex,
			pipeline_tex,
			renderpass_tex,
			texman: Default::default(),
			texset: None,
		}
	}

	fn generate_tex_vertex_buffer(
		&self,
		model: &Model,
	) -> Option<VertexTexBuffer> {
		let vertices = model
			.tex_faces
			.iter()
			.flat_map(|face| {
				(0..3).map(|i| VertexTex {
					pos: model.vs[face.vid[i]],
					tex_coord: model.uvs[face.uvid[i]],
					tex_layer: *self.texman.mapper.get(&face.layer).unwrap() as i32,
				})
			})
			.collect::<Vec<_>>();
		if vertices.is_empty() {
			return None;
		}
		let result = CpuAccessibleBuffer::from_iter(
			self.base.device.clone(),
			BufferUsage {
				vertex_buffer: true,
				..BufferUsage::empty()
			},
			false,
			vertices.into_iter(),
		)
		.unwrap();
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

		let layout = self.pipeline_tex.layout().set_layouts().get(0).unwrap();
		let set = PersistentDescriptorSet::new(
			layout.clone(),
			[WriteDescriptorSet::buffer(0, uniform_buffer)],
		)
		.unwrap();

		let vertex_buffer = self.generate_tex_vertex_buffer(&model);
		if let Some(vertex_buffer) = vertex_buffer {
			if self.texman.get_dirty() {
				let tex_len = self.texman.tex_len();
				self.pipeline_tex = get_pipeline_tex(
					self.renderpass_tex.clone(),
					self.base.device.clone(),
					tex_len as u32,
				);
				let layout =
					self.pipeline_tex.layout().set_layouts().get(1).unwrap();
				let texset = self
					.texman
					.compile_set(self.base.device.clone(), layout.clone());
				self.texset = texset;
			}
			if let Some(texset) = self.texset.clone() {
				let clear_values = vec![Some([0.0; 4].into())];
				builder
					.begin_render_pass(
						RenderPassBeginInfo {
							clear_values,
							..RenderPassBeginInfo::framebuffer(
								self.framebuffers_tex[image_num].clone(),
							)
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
		self.framebuffers_tex =
			window_size_dependent_setup(self.renderpass_tex.clone(), images);
	}
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

pub fn get_pipeline_tex(
	render_pass: VkwRenderPass,
	device: VkwDevice,
	tex_len: u32,
) -> VkwPipeline {
	let vs = shader::vs::load(device.clone()).unwrap();
	let fs = shader::fs::load(device.clone()).unwrap();
	let mut layout_create_infos: Vec<_> =
		DescriptorSetLayoutCreateInfo::from_requirements(
			vs.entry_point("main")
				.unwrap()
				.descriptor_requirements()
				.chain(
					fs.entry_point("main").unwrap().descriptor_requirements(),
				),
		);
	let mut binding = layout_create_infos[0].bindings.get_mut(&0).unwrap();
	binding.descriptor_type = DescriptorType::UniformBuffer;
	let mut binding = layout_create_infos[1].bindings.get_mut(&0).unwrap();
	binding.variable_descriptor_count = true;
	binding.descriptor_count = tex_len;
	let set_layouts = layout_create_infos
		.into_iter()
		.map(|desc| DescriptorSetLayout::new(device.clone(), desc))
		.collect::<Result<Vec<_>, DescriptorSetLayoutCreationError>>()
		.unwrap();
	let pipeline_layout = PipelineLayout::new(
		device.clone(),
		PipelineLayoutCreateInfo {
			set_layouts,
			..Default::default()
		},
	)
	.unwrap();

	let pipeline = GraphicsPipeline::start()
		.vertex_input_state(BuffersDefinition::new().vertex::<VertexTex>())
		.vertex_shader(vs.entry_point("main").unwrap(), ())
		.input_assembly_state(
			InputAssemblyState::new().topology(PrimitiveTopology::TriangleList),
		)
		.viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
		.fragment_shader(fs.entry_point("main").unwrap(), ())
		.render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
		.with_pipeline_layout(device.clone(), pipeline_layout)
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
