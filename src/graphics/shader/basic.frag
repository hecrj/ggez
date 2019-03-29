#version 450

layout(binding = 0) uniform sampler2D t_Texture;

layout(location = 0) in vec2 v_Uv;
layout(location = 1) in vec4 v_Color;

layout(location = 0) out vec4 outColor;

layout (std140) uniform Globals {
    mat4 u_MVP;
};

void main() {
    outColor = texture(t_Texture, v_Uv) * v_Color;
}
