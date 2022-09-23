use std::collections::HashMap;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::sampler::{Sampler, SamplerCreateInfo};
use vulkano::image::{ImageDimensions, ImmutableImage, MipmapsCount};
use vulkano::image::view::{ImageView, ImageViewCreateInfo, ImageViewType};
use vulkano::sync::GpuFuture;
use vulkano::format::Format;

use crate::helper::*;
use crate::teximg::Teximg;

#[derive(Default)]
pub struct Texman {
	pub mapper: HashMap<usize, usize>,

	// pending remove_list record inner id only,
	// when an inner get pushed, it must have already been deleted from mapper
	// the removal is executed in compile_set
	remove_list: Vec<usize>,

	image_views: Vec<VkwImageView>,
	future: Option<VkwFuture>,
}

impl Texman {
	// removed when TexHandle dropped
	pub fn upload(&mut self, image: Teximg, id: usize, queue: VkwQueue) {
		if let Some(id_inner) = self.mapper.get(&id) {
			self.remove_list.push(*id_inner);
		}
		let (texture, future) = {
			let dimensions = ImageDimensions::Dim2d {
				width: 1024,
				height: 1024,
				array_layers: 1,
			};
			let format = Format::R8G8B8A8_SRGB;
			let (image, future) = ImmutableImage::from_iter(
				image.data.into_iter(),
				dimensions,
				MipmapsCount::One,
				format,
				queue,
			).unwrap();
			let image_view = ImageView::new(
				image.clone(),
				ImageViewCreateInfo {
					view_type: ImageViewType::Dim2d,
					..ImageViewCreateInfo::from_image(&image)
				},
			)
			.unwrap();
			(image_view, future)
		};
		if let Some(f) = self.future.take() {
			self.future = Some(Box::new(f.join(future)));
		} else {
			self.future = Some(Box::new(future));
		}
		self.mapper.insert(id, self.image_views.len());
		self.image_views.push(texture);
	}

	pub fn remove(&mut self, outer: usize) {
		let inner = self.mapper.remove(&outer).unwrap();
		self.remove_list.push(inner);
	}

	pub fn compile_set(
		&mut self,
		device: VkwDevice,
		layout: VkwTexLayout,
	) -> Option<VkwTextureSet> {
		let mut new_mapper = HashMap::new();
		let mut new_views = Vec::new();
		for (outer, inner) in self.mapper.iter() {
			eprintln!("{} -> {}", outer, inner);
			if self.remove_list.iter().any(|x| x == inner) {
				continue
			}
			new_mapper.insert(*outer, new_views.len());
			new_views.push(self.image_views[*inner].clone());
		}
		self.mapper = new_mapper;
		self.image_views = new_views;

		if let Some(future) = self.future.take() {
			future.flush().unwrap();
		}
		let iter: Vec<_> = self.image_views
			.iter()
			.cloned()
			.map(|view| {
				let sampler = Sampler::new(
					device.clone(),
					SamplerCreateInfo::simple_repeat_linear(),
				).unwrap();
				(view as _, sampler)
			}).collect();
		if iter.is_empty() { return None }
	
		PersistentDescriptorSet::new(
			layout.clone(),
			[WriteDescriptorSet::image_view_sampler_array(0, 0, iter)],
		).ok()
	}
}
