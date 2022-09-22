#version 450

layout(location = 0) in vec4 f_rgba;
layout(location = 0) out vec4 f_color;

void main() {
	f_color = f_rgba;
}
