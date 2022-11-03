#[derive(Clone, Debug, Default)]
pub struct Model {
	pub vs: Vec<[f32; 4]>,
	pub uvs: Vec<[f32; 2]>,
	pub tex_faces: Vec<TexFace>,
}

#[derive(Clone, Debug)]
pub struct TexFace {
	pub color: [f32; 4],
	pub vid: [usize; 3],
	pub uvid: [usize; 3],
	pub layer: i32,
}
