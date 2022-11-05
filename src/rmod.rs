use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{RenderPassBeginInfo, SubpassContents};
use vulkano::descriptor_set::layout::{
	DescriptorSetLayout, DescriptorSetLayoutCreateInfo,
	DescriptorSetLayoutCreationError, DescriptorType,
};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{AttachmentImage, ImageAccess};
use vulkano::pipeline::graphics::color_blend::ColorBlendState;
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
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
use crate::model::modelman::Modelman;
use crate::shader;
use crate::texman::Texman;
use crate::vertex::VertexTex;

pub struct Rmod {
	base: Base,
	framebuffers_tex: Vec<VkwFramebuffer>,
	pipeline_tex: VkwPipeline,
	renderpass_tex: VkwRenderPass,
	pub texman: Texman,
	pub modelman: Modelman,
	texset: Option<VkwTextureSet>,
}

impl Rmod {
	pub fn new(base: Base) -> Self {
		let device = base.device.clone();
		let renderpass_tex =
			get_render_pass_clear(device.clone(), base.swapchain.clone());
		let pipeline_tex = get_pipeline_tex(renderpass_tex.clone(), device, 0);
		let framebuffers_tex = window_size_dependent_setup(
			renderpass_tex.clone(),
			&base.images,
			base.memalloc.clone(),
		);
		let memalloc = base.memalloc.clone();
		Self {
			base,
			framebuffers_tex,
			pipeline_tex,
			renderpass_tex,
			texman: Default::default(),
			modelman: Modelman::new(memalloc),
			texset: None,
		}
	}

	pub fn build_command(
		&mut self,
		builder: &mut VkwCommandBuilder,
		image_num: usize,
		camera: Camera,
		viewport: Viewport,
	) {
		if self.texman.get_dirty() {
			let (tex_len, update_mapper) = self.texman.tex_len();
			if tex_len == 0 { return }
			self.modelman.map_tex(update_mapper);
			self.pipeline_tex = get_pipeline_tex(
				self.renderpass_tex.clone(),
				self.base.device.clone(),
				tex_len as u32,
			);
			let layout =
				self.pipeline_tex.layout().set_layouts().get(1).unwrap();
			let texset = self.texman.compile_set(
				self.base.device.clone(),
				self.base.dstalloc.clone(),
				layout.clone(),
			);
			self.texset = texset;
		}
		// dirty workaround for gpulock
		let count = match self.modelman.write_buffer() {
			Some(count) => count,
			None => return,
		};

		let uniform_buffer = CpuAccessibleBuffer::from_data(
			&self.base.memalloc,
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
			&self.base.dstalloc,
			layout.clone(),
			[WriteDescriptorSet::buffer(0, uniform_buffer)],
		)
		.unwrap();

		let texset = self.texset.clone().unwrap();
		let clear_values = vec![Some([0.0; 4].into()), Some(1f32.into())];
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
		let buffer = self.modelman.buffer.clone();
		builder.bind_pipeline_graphics(self.pipeline_tex.clone());
		builder
			.bind_vertex_buffers(0, buffer)
			.draw(count as u32, 1, 0, 0)
			.unwrap();
		builder.end_render_pass().unwrap();
	}

	pub fn update_framebuffers(&mut self, images: &VkwImages) {
		self.framebuffers_tex = window_size_dependent_setup(
			self.renderpass_tex.clone(),
			images,
			self.base.memalloc.clone(),
		);
	}
}

pub fn get_render_pass_clear(
	device: VkwDevice,
	swapchain: VkwSwapchain,
) -> VkwRenderPass {
	vulkano::single_pass_renderpass!(
		device,
		attachments: {
			color: {
				load: Clear,
				store: Store,
				format: swapchain.image_format(),
				samples: 1,
			},
			depth: {
				load: Clear,
				store: Store,
				format: Format::D16_UNORM,
				samples: 1,
			}
		},
		pass: {
			color: [color],
			depth_stencil: {depth}
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

	let subpass = Subpass::from(render_pass, 0).unwrap();
	let pipeline = GraphicsPipeline::start()
		.vertex_input_state(BuffersDefinition::new().vertex::<VertexTex>())
		.vertex_shader(vs.entry_point("main").unwrap(), ())
		.input_assembly_state(
			InputAssemblyState::new().topology(PrimitiveTopology::TriangleList),
		)
		.viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
		.fragment_shader(fs.entry_point("main").unwrap(), ())
		.depth_stencil_state(DepthStencilState::simple_depth_test())
		.color_blend_state(
			ColorBlendState::new(subpass.num_color_attachments()).blend_alpha(),
		)
		.render_pass(subpass)
		.with_pipeline_layout(device, pipeline_layout)
		.unwrap();
	pipeline
}

pub fn window_size_dependent_setup(
	render_pass: VkwRenderPass,
	images: &VkwImages,
	memalloc: VkwMemAlloc,
) -> Vec<VkwFramebuffer> {
	let dimensions = images[0].dimensions().width_height();
	let depth_buffer = ImageView::new_default(
		AttachmentImage::transient(&memalloc, dimensions, Format::D16_UNORM)
			.unwrap(),
	)
	.unwrap();

	images
		.iter()
		.map(|image| {
			let view = ImageView::new_default(image.clone()).unwrap();
			Framebuffer::new(
				render_pass.clone(),
				FramebufferCreateInfo {
					attachments: vec![view, depth_buffer.clone()],
					..Default::default()
				},
			)
			.unwrap()
		})
		.collect::<Vec<_>>()
}
