pub mod vs {
	vulkano_shaders::shader! {
		ty: "vertex",
		path: "src/shader/vert.glsl"
	}
}

pub mod fs {
	vulkano_shaders::shader! {
		ty: "fragment",
		vulkan_version: "1.2",
		spirv_version: "1.5",
		path: "src/shader/frag.glsl",
	}
}
