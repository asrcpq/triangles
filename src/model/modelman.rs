use std::collections::HashMap;
use std::sync::Arc;
use std::cell::Ref;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};

use crate::helper::*;
use crate::vertex::VertexTex;
use super::compiled_model::CompiledModel;
use super::cmodel::Model;
use super::model_ref::ModelRef;

const BUFSIZE: usize = 1 << 24;
type VertexTexBuffer = Arc<CpuAccessibleBuffer<[VertexTex; BUFSIZE]>>;

pub struct Modelman {
	pub buffer: VertexTexBuffer,
	models: Vec<ModelRef>,
}

impl Modelman {
	pub fn new(memalloc: VkwMemAlloc) -> Self {
		let buffer = unsafe {
			CpuAccessibleBuffer::uninitialized(
				&memalloc,
				BufferUsage {
					vertex_buffer: true,
					..BufferUsage::empty()
				},
				true,
			)
			.unwrap()
		};
		Self {
			buffer,
			models: Default::default(),
		}
	}

	pub fn insert(
		&mut self,
		z: i32,
		model: &Model,
		mapper: &HashMap<i32, i32>,
	) -> ModelRef {
		let vertices = model
			.tex_faces
			.iter()
			.flat_map(|face| {
				(0..3).map(|i| {
					let tex_coord = if face.layer < 0 {
						[0.0; 2]
					} else {
						model.uvs[face.uvid[i]]
					};
					let tex_layer = if face.layer < 0 {
						face.layer
					} else {
						*mapper.get(&face.layer).unwrap()
					};
					VertexTex {
						pos: model.vs[face.vid[i]],
						color: face.color,
						tex_coord,
						tex_layer,
					}
				})
			})
			.collect::<Vec<_>>();
		assert!(!vertices.is_empty()); // TODO: allow empty model
		let model = CompiledModel {
			visible: true,
			z,
			vertices,
		};
		let model = ModelRef::new(model);
		self.models.push(model.clone());
		model
	}

	pub fn map_tex(&mut self, mapper: HashMap<i32, i32>) {
		for model in self.models.iter_mut() {
			let mut model = model.borrow_mut();
			for v in model.vertices.iter_mut() {
				let l = &mut v.tex_layer;
				if *l >= 0 {
					*l = *mapper.get(l).unwrap();
				}
			}
		}
	}

	pub fn gc(&mut self) {
		for model in std::mem::take(&mut self.models).into_iter() {
			if !model.dropped() {
				self.models.push(model);
			}
		}
	}

	pub fn write_buffer(&mut self) -> Option<usize> {
		self.gc();
		let mut buffers: Vec<Ref<CompiledModel>> =
			self.models.iter().map(|x| x.borrow()).filter(|x| x.visible).collect();
		buffers.sort_by_key(|x| x.z);
		let len = buffers.iter().map(|x| x.vertices.len()).sum();

		let buffer = self.buffer.clone();
		let mut writer = if let Ok(writer) = buffer.write() {
			writer
		} else {
			eprintln!("ERROR: Gpu locked");
			return None;
		};
		for (v, w) in writer
			.iter_mut()
			.zip(buffers.iter().flat_map(|x| &x.vertices))
		{
			*v = *w;
		}
		Some(len)
	}
}
