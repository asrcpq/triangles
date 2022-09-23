pub mod vs_solid {
	vulkano_shaders::shader! {
		ty: "vertex",
		path: "src/shader/solid/vert.glsl"
	}
}

pub mod fs_solid {
	vulkano_shaders::shader! {
		ty: "fragment",
		path: "src/shader/solid/frag.glsl"
	}
}

pub mod vs_tex {
	vulkano_shaders::shader! {
		ty: "vertex",
		path: "src/shader/tex/vert.glsl"
	}
}

pub mod fs_tex {
	vulkano_shaders::shader! {
		ty: "fragment",
		vulkan_version: "1.2",
		spirv_version: "1.5",
		path: "src/shader/tex/frag.glsl",
	}
}
