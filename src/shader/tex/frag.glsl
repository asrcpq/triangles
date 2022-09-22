#version 450

layout(location = 0) in vec2 f_tex_coord;
layout(location = 1) flat in int f_tex_layer;
layout(location = 0) out vec4 f_color;

layout(set = 1, binding = 0) uniform sampler2DArray tex;

void main() {
	f_color = texture(tex, vec3(f_tex_coord, f_tex_layer));
}
