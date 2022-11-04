use std::collections::HashMap;
use std::sync::Arc;
use std::rc::Rc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};

use crate::helper::*;
use crate::model::Model;
use crate::vertex::VertexTex;

// never reuse id, u64 is considered sufficient
type Mid = u64;

#[derive(Hash, Clone, Debug)]
pub struct ModelHandle(Rc<Mid>);

struct CompiledModel {
	pub visible: bool,
	pub z: i32,
	pub vertices: Vec<VertexTex>,
}

const BUFSIZE: usize = 1 << 24;
type VertexTexBuffer = Arc<CpuAccessibleBuffer<[VertexTex; BUFSIZE]>>;

pub struct Modelman {
	pub buffer: VertexTexBuffer,
	id_alloc: Mid,
	models: HashMap<Rc<Mid>, CompiledModel>,
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
			id_alloc: 0,
			models: HashMap::new(),
		}
	}

	pub fn insert(
		&mut self,
		z: i32,
		model: &Model,
		mapper: &HashMap<i32, i32>,
	) -> ModelHandle {
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
		let handle = Rc::new(self.id_alloc);
		self.id_alloc += 1;
		self.models.insert(
			handle.clone(),
			CompiledModel {
				visible: true,
				z,
				vertices,
			},
		);
		ModelHandle(handle)
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

	pub fn set_z(&mut self, handle: &ModelHandle, z: i32) {
		self.models.get_mut(&handle.0).unwrap().z = z;
	}

	pub fn gc(&mut self) {
		let mut removal_list = Vec::new();
		for (k, _) in self.models.iter() {
			if Rc::strong_count(k) == 1 {
				removal_list.push(k.clone());
			}
		}
		for removal in removal_list.iter() {
			self.models.remove(removal);
		}
	}

	pub fn set_visibility(&mut self, handle: &ModelHandle, visible: bool) {
		self.models.get_mut(&handle.0).unwrap().visible = visible;
	}

	pub fn write_buffer(&mut self) -> Option<usize> {
		self.gc();
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
