use std::collections::HashMap;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::format::Format;
use vulkano::image::view::{ImageView, ImageViewCreateInfo, ImageViewType};
use vulkano::image::{ImageDimensions, ImmutableImage, MipmapsCount};
use vulkano::sampler::{Sampler, SamplerCreateInfo};
use vulkano::sync::GpuFuture;

use crate::helper::*;
use crate::teximg::Teximg;

#[derive(Default)]
pub struct Texman {
	pub mapper: HashMap<u32, usize>,

	// pending remove_list record inner id only,
	// when an inner get pushed, it must have already been deleted from mapper
	// the removal is executed in compile_set
	remove_list: Vec<usize>,

	image_views: Vec<VkwImageView>,
	future: Option<VkwFuture>,
	dirty: bool,
}

impl Texman {
	// removed when TexHandle dropped
	pub fn upload(&mut self, image: Teximg, id: u32, memalloc: VkwMemAlloc, builder: &mut VkwCommandBuilder) {
		if let Some(id_inner) = self.mapper.get(&id) {
			self.remove_list.push(*id_inner);
		}
		let dimensions = ImageDimensions::Dim2d {
			width: image.dim[0],
			height: image.dim[1],
			array_layers: 1,
		};
		let format = Format::R8G8B8A8_SRGB;
		let image = ImmutableImage::from_iter(
			&memalloc,
			image.data.into_iter(),
			dimensions,
			MipmapsCount::One,
			format,
			builder,
		).unwrap();
		let image_view = ImageView::new(
			image.clone(),
			ImageViewCreateInfo {
				view_type: ImageViewType::Dim2d,
				..ImageViewCreateInfo::from_image(&image)
			},
		).unwrap();
		self.mapper.insert(id, self.image_views.len());
		self.image_views.push(image_view);
		self.dirty = true;
	}

	pub fn tex_len(&mut self) -> usize {
		self.gc();
		self.image_views.len()
	}

	pub fn remove(&mut self, outer: u32) {
		let inner = self.mapper.remove(&outer).unwrap();
		self.dirty = true;
		self.remove_list.push(inner);
	}

	pub fn get_dirty(&mut self) -> bool {
		self.dirty
	}

	fn gc(&mut self) {
		let mut new_mapper = HashMap::new();
		let mut new_views = Vec::new();
		for (outer, inner) in self.mapper.iter() {
			eprintln!("{} -> {}", outer, inner);
			if self.remove_list.iter().any(|x| x == inner) {
				continue;
			}
			new_mapper.insert(*outer, new_views.len());
			new_views.push(self.image_views[*inner].clone());
		}
		self.remove_list.clear();
		self.mapper = new_mapper;
		self.image_views = new_views;
		self.dirty = false;
	}

	// NOTE: gc is called in tex_len, not called here!
	pub fn compile_set(
		&mut self,
		device: VkwDevice,
		dstalloc: VkwDstAlloc,
		layout: VkwTexLayout,
	) -> Option<VkwTextureSet> {
		if let Some(future) = self.future.take() {
			future.flush().unwrap();
		}
		let iter: Vec<_> = self
			.image_views
			.iter()
			.cloned()
			.map(|view| {
				let sampler = Sampler::new(
					device.clone(),
					SamplerCreateInfo::default(),
				)
				.unwrap();
				(view as _, sampler)
			})
			.collect();
		if iter.is_empty() {
			return None;
		}

		Some(
			PersistentDescriptorSet::new_variable(
				&dstalloc,
				layout.clone(),
				iter.len() as u32,
				[WriteDescriptorSet::image_view_sampler_array(0, 0, iter)],
			)
			.unwrap(),
		)
	}
}
