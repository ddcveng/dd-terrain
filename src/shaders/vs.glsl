#version 400

in vec3 position;
in vec3 color;
in vec3 normal;

// instance data
in vec3 offset;
in vec3 instance_color;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

out vec3 frag_color;
out vec3 v_normal;

void main() {  
    frag_color = instance_color;
    v_normal = normal;

    vec3 real_position = position + offset;
    gl_Position = projection * view * model * vec4(real_position, 1.);
}

