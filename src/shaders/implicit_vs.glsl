#version 400

in vec3 position;
in vec3 normal;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

out vec3 v_normal;
out vec3 fragment_position;

void main() {  
    v_normal = normal;
    fragment_position = vec3(model * vec4(position, 1.0));

    gl_Position = projection * view * model * vec4(position, 1.);
}

