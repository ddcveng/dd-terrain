#version 400

in vec3 position;
in vec3 normal;
in mat4 vertex_material_weights;
//in vec4 blend_coefficients;
//in vec4 blend_indices;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;
uniform sampler2D block_pallette;

out vec3 v_normal;
out vec3 fragment_position;
out mat4 blend_weights;
//out vec4 blend_weights;
//out vec4 blend_materials;
//out vec4 fragment_color;

vec3 get_material_color(uint material_index) {
    switch (material_index) {
        case 1: // dirt
            return vec3(0.333, 0.129, 0.0);
        case 2: // grass
            return vec3(0.27, 0.451, 0.075);
        case 3: // Stone
            return vec3(0.431, 0.573, 0.631);
        case 5: // sand
            return vec3(1.0, 0.576, 0.012);
        default:
            return vec3(0.0, 0.0, 0.0);
    }
}

void main() {  
    v_normal = normalize(normal);
    fragment_position = vec3(model * vec4(position, 1.0));
    //fragment_color = assemble_color(fragment_position, v_normal);
    blend_weights = vertex_material_weights;

    gl_Position = projection * view * model * vec4(position, 1.);
}

