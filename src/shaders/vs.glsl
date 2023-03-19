#version 330

in vec3 position;
in vec3 color;
in vec3 normal;
in vec3 offset;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

out vec3 frag_color;
out vec3 v_normal;

void main() {  
    frag_color = color;
    v_normal = normal;

    vec3 real_position = position + offset;
    gl_Position = projection * view * model * vec4(real_position, 1.);
}
