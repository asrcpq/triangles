#version 450

layout(location = 0) in vec4 pos;
layout(location = 1) in vec4 rgba;

layout(location = 0) out vec4 f_rgba;

layout(set = 0, binding = 0) uniform Data {
	mat4 proj;
} uniforms;

void main() {
	gl_Position = uniforms.proj * pos;
	f_rgba = rgba;
}
