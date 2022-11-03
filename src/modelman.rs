use std::collections::HashMap;
use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};

use crate::helper::*;
use crate::model::Model;
use crate::vertex::VertexTex;

struct CompiledModel {
	pub visible: bool,
	pub z: u32,
	pub vertices: Vec<VertexTex>,
}

const BUFSIZE: usize = 1 << 24;
type VertexTexBuffer = Arc<CpuAccessibleBuffer<[VertexTex; BUFSIZE]>>;

pub struct Modelman {
	pub buffer: VertexTexBuffer,
	models: HashMap<u32, CompiledModel>,
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
			models: HashMap::new(),
		}
	}

	pub fn insert(
		&mut self,
		id: u32,
		model: &Model,
		mapper: &HashMap<i32, i32>,
	) {
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
		self.models.insert(
			id,
			CompiledModel {
				visible: true,
				z: 0,
				vertices,
			},
		);
	}

	pub fn map_tex(&mut self, mapper: HashMap<i32, i32>) {
		for model in self.models.values_mut() {
			for v in model.vertices.iter_mut() {
				let l = &mut v.tex_layer;
				if *l >= 0 {
					*l = *mapper.get(l).unwrap();
				}
			}
		}
	}

	pub fn set_z(&mut self, id: u32, z: u32) {
		self.models.get_mut(&id).unwrap().z = z;
	}

	pub fn remove(&mut self, id: u32) -> bool {
		self.models.remove(&id).is_some()
	}

	pub fn set_visibility(&mut self, id: u32, visible: bool) {
		self.models.get_mut(&id).unwrap().visible = visible;
	}

	pub fn write_buffer(&mut self) -> Option<usize> {
		let mut buffers: Vec<&CompiledModel> =
			self.models.values().filter(|x| x.visible).collect();
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
