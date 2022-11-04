#[derive(Clone, Debug, Default)]
pub struct Model {
	pub vs: Vec<[f32; 4]>,
	pub uvs: Vec<[f32; 2]>,
	pub faces: Vec<Face>,
}

#[derive(Clone, Debug)]
pub struct Face {
	pub color: [f32; 4],
	pub vid: [usize; 3],
	pub uvid: [usize; 3],
	pub layer: i32,
}
