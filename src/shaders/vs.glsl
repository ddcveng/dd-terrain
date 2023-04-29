#version 400

in vec3 position;
in vec3 color;
in vec3 normal;

// instance data
in vec3 offset;
in vec3 instance_color;
in uint height;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

out vec3 frag_color;
out vec3 v_normal;

void main() {  
    frag_color = instance_color;
    v_normal = normal;

    // We start with a block that has 0,0,0 in its center
    // We want 0,0,0 to be one of its corners so we offset x and z by cube_size / 2
    // y is scaled by height, so we have to offset it by height * cube_size / 2
    float additional_offset_y = height / 2.0;
    vec4 scale_offset = vec4(offset.x + 0.5, offset.y + additional_offset_y, offset.z + 0.5, 1.0);

    mat4 scale;
    scale[0] = vec4(1.0, 0.0, 0.0, 0.0);
    scale[1] = vec4(0.0, height, 0.0, 0.0);
    scale[2] = vec4(0.0, 0.0, 1.0, 0.0);
    scale[3] = scale_offset;

    //vec3 real_position = position + offset;
    gl_Position = projection * view * scale * model * vec4(position, 1.);
}

