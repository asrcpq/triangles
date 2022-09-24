#version 450

layout(location = 0) in vec4 pos;
layout(location = 1) in vec2 tex_coord;
layout(location = 2) in int tex_layer;

layout(location = 0) out vec2 f_tex_coord;
layout(location = 1) out int f_tex_layer;

layout(set = 0, binding = 0) uniform Data {
	mat4 proj;
} uniforms;

void main() {
	gl_Position = uniforms.proj * pos;
	f_tex_coord = tex_coord;
	f_tex_layer = tex_layer;
}
