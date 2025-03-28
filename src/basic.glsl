-- vertex
#version 330 core

layout(location = 0) in vec3 a_pos;
layout(location = 1) in vec2 a_tex_coord;
layout(location = 2) in vec3 instance_pos;

out vec2 tex_coord;

// uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main() {
    gl_Position = projection * view * vec4(a_pos + instance_pos, 1.0);
    tex_coord = a_tex_coord;
}

-- fragment
#version 330 core

in vec2 tex_coord;
out vec4 frag_color;

uniform sampler2D tex;

void main() {
    frag_color = texture(tex, tex_coord);
}
