use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Zeroable, Pod, Default, Debug, Clone, Copy)]
pub struct VertexTex {
	pub pos: [f32; 4],
	pub color: [f32; 4],
	pub tex_coord: [f32; 2],
	pub tex_layer: i32,
}
vulkano::impl_vertex!(VertexTex, pos, color, tex_coord, tex_layer);
