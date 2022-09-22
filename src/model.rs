#[derive(Default)]
pub struct Model {
	pub vs: Vec<[f32; 4]>,
	pub uvs: Vec<[f32; 2]>,
	pub solid_faces: Vec<SolidFace>,
	pub tex_faces: Vec<TexFace>,
}

pub struct TexFace {
	pub vid: [usize; 3],
	pub uvid: [usize; 3],
	pub layer: usize,
}

pub struct SolidFace {
	pub vid: [usize; 3],
	pub rgba: [f32; 4],
}
