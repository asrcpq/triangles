use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Zeroable, Pod, Default, Debug, Clone, Copy)]
pub struct VertexTex {
	pub pos: [f32; 4],
	pub tex_coord: [f32; 2],
	pub tex_layer: i32,
}
vulkano::impl_vertex!(VertexTex, pos, tex_coord, tex_layer);

#[repr(C)]
#[derive(Zeroable, Pod, Default, Debug, Clone, Copy)]
pub struct VertexSolid {
	pub pos: [f32; 4],
	pub rgba: [f32; 4],
}
vulkano::impl_vertex!(VertexSolid, pos, rgba);
