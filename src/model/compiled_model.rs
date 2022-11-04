use crate::vertex::VertexTex;

pub struct CompiledModel {
	pub visible: bool,
	pub z: i32,
	pub vertices: Vec<VertexTex>,
}
