use std::sync::Arc;
use std::collections::HashMap;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};

use crate::vertex::VertexTex;
use crate::helper::*;
use crate::model::Model;

struct CompiledModel {
	pub visible: bool,
	pub z: i32,
	pub vertices: Vec<VertexTex>,
}

const BUFSIZE: usize = 1 << 24;
type VertexTexBuffer = Arc<CpuAccessibleBuffer<[VertexTex; BUFSIZE]>>;

pub struct Modelman {
	pub buffer: VertexTexBuffer,
	models: HashMap<u32, CompiledModel>,
}

impl Modelman {
	pub fn new(device: VkwDevice) -> Self {
		let buffer = unsafe {
			CpuAccessibleBuffer::uninitialized(
				device.clone(),
				BufferUsage {
					vertex_buffer: true,
					..BufferUsage::empty()
				},
				true,
			).unwrap()
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
		mapper: &HashMap<u32, usize>,
	) {
		let vertices = model
			.tex_faces
			.iter()
			.flat_map(|face| {
				(0..3).map(|i| VertexTex {
					pos: model.vs[face.vid[i]],
					tex_coord: model.uvs[face.uvid[i]],
					tex_layer: *mapper.get(&face.layer).unwrap() as i32,
				})
			})
			.collect::<Vec<_>>();
		assert!(!vertices.is_empty()); // TODO: return error instead
		self.models.insert(id, CompiledModel {
			visible: true,
			z: 0,
			vertices,
		});
	}

	pub fn set_z(&mut self, id: u32, z: i32) {
		self.models.get_mut(&id).unwrap().z = z;
	}

	pub fn remove(&mut self, id: u32) -> bool {
		self.models.remove(&id).is_some()
	}

	pub fn set_visibility(&mut self, id: u32, visible: bool) {
		self.models.get_mut(&id).unwrap().visible = visible;
	}

	pub fn write_buffer(&mut self) -> usize {
		let mut buffers: Vec<&CompiledModel> = self.models.values().filter(|x| x.visible).collect();
		buffers.sort_by_key(|x| x.z);
		let len = buffers.iter().map(|x| x.vertices.len()).sum();

		let buffer = self.buffer.clone();
		let mut writer = buffer.write().unwrap();
		for (v, w) in writer.iter_mut().zip(buffers.iter().flat_map(|x| &x.vertices)) {
			*v = *w;
		}
		len
	}
}
