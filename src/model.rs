use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct Model {
	pub vs: Vec<[f32; 4]>,
	pub uvs: Vec<[f32; 2]>,
	pub tex_faces: Vec<TexFace>,
	pub z: i32,
}

#[derive(Clone)]
pub struct TexFace {
	pub vid: [usize; 3],
	pub uvid: [usize; 3],
	pub layer: u32,
}

#[derive(Default)]
pub struct Modelman {
	models: HashMap<u32, Model>,
}

impl Modelman {
	pub fn insert(&mut self, id: u32, model: Model) -> Option<Model> {
		self.models.insert(id, model)
	}

	pub fn remove(&mut self, id: u32) -> Option<Model> {
		self.models.remove(&id)
	}
	// pub show
	// pub hide
}
