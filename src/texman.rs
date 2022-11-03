use std::collections::HashMap;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::format::Format;
use vulkano::image::view::{ImageView, ImageViewCreateInfo, ImageViewType};
use vulkano::image::{ImageDimensions, ImmutableImage, MipmapsCount};
use vulkano::sampler::{Sampler, SamplerCreateInfo};

use crate::helper::*;
use crate::teximg::Teximg;

#[derive(Default)]
pub struct Texman {
	pub mapper: HashMap<i32, i32>,

	// pending remove_list record inner id only,
	// when an inner get pushed, it must have already been deleted from mapper
	// the removal is executed in compile_set
	remove_list: Vec<i32>,
	id_alloc: i32,

	image_views: Vec<VkwImageView>,
	dirty: bool,
}

fn create_image_view(image: Teximg, memalloc: VkwMemAlloc, builder: &mut VkwCommandBuilder) -> VkwImageView {
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
	ImageView::new(
		image.clone(),
		ImageViewCreateInfo {
			view_type: ImageViewType::Dim2d,
			..ImageViewCreateInfo::from_image(&image)
		},
	).unwrap()
}

impl Texman {
	// removed when TexHandle dropped
	pub fn upload(
		&mut self,
		image: Teximg,
		id: i32,
		memalloc: VkwMemAlloc,
		builder: &mut VkwCommandBuilder,
	) {
		if let Some(id_inner) = self.mapper.get(&id) {
			self.remove_list.push(*id_inner);
		}
		let image_view = create_image_view(image, memalloc, builder);
		self.mapper.insert(id, self.id_alloc);
		self.id_alloc += 1;
		self.image_views.push(image_view);
		self.dirty = true;
	}

	pub fn tex_len(&mut self) -> (usize, HashMap<i32, i32>) {
		let update_mapper = self.gc();
		(self.image_views.len(), update_mapper)
	}

	pub fn remove(&mut self, outer: i32) {
		assert!(outer >= 0);
		let inner = self.mapper.remove(&outer).unwrap();
		self.dirty = true;
		self.remove_list.push(inner);
	}

	pub fn get_dirty(&mut self) -> bool {
		self.dirty
	}

	fn gc(&mut self) -> HashMap<i32, i32> {
		let mut new_mapper: HashMap<i32, i32> = HashMap::new();
		let mut update_mapper = HashMap::new();
		let mut new_views = Vec::new();
		for (outer, inner) in self.mapper.iter() {
			if self.remove_list.iter().any(|x| x == inner) {
				continue;
			}
			update_mapper.insert(*inner, new_views.len() as i32);
			new_mapper.insert(*outer, new_views.len() as i32);
			new_views.push(self.image_views[*inner as usize].clone());
		}
		self.remove_list.clear();
		self.mapper = new_mapper;
		self.image_views = new_views;
		self.dirty = false;
		self.id_alloc = self.image_views.len() as i32;
		update_mapper
	}

	// NOTE: gc is called in tex_len, not called here!
	pub fn compile_set(
		&mut self,
		device: VkwDevice,
		dstalloc: VkwDstAlloc,
		layout: VkwTexLayout,
	) -> Option<VkwTextureSet> {
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
