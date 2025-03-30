-- vertex
#version 330 core

layout(location = 0) in vec3 a_pos;
layout(location = 1) in vec2 a_tex_coord;
layout(location = 2) in vec3 a_normal;
layout(location = 3) in vec3 instance_pos;

out vec3 normal;
out vec3 frag_position;
out vec2 tex_coord;

uniform mat4 view;
uniform mat4 projection;

uniform vec3 light_position;

void main() {
    gl_Position = projection * view * vec4(a_pos + instance_pos, 1.0);

    tex_coord = a_tex_coord;
    normal = a_normal;
    frag_position = a_pos + instance_pos;
}

-- fragment
#version 330 core

in vec2 tex_coord;
in vec3 normal;
in vec3 frag_position;

out vec4 frag_color;

uniform sampler2D tex;

uniform vec3 light_color;
uniform vec3 light_position;
uniform vec3 eye_position;

void main() {
    vec3 norm = normalize(normal);

    // ambient
    float ambient_strength = 0.1;
    vec3 ambient = ambient_strength * light_color;

    // diffuse
    vec3 light_direction = normalize(light_position - frag_position);
    float diff = max(dot(norm, light_direction), 0.0);
    vec3 diffuse = diff * light_color;

    // specular
    float specular_strength = 0.5;
    vec3 eye_direction = normalize(eye_position - frag_position);
    vec3 reflect_dir = reflect(-light_direction, norm);

    float spec = pow(max(dot(eye_direction, reflect_dir), 0.0), 32);
    vec3 specular = specular_strength * spec * light_color;

    vec4 intensity = vec4(ambient + diffuse + specular, 1.0);
    frag_color = texture(tex, tex_coord) * intensity;
}
