#version 400

in vec3 position;
in vec3 normal;
in vec4 blend_coefficients;
in vec4 blend_indices;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;
uniform sampler2D block_pallette;

out vec3 v_normal;
out vec3 fragment_position;
out vec4 fragment_color;

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

const int PALLETTE_SIZE = 4;
const int MAX_PALLETTE_OFFSET = PALLETTE_SIZE - 1;
vec2 get_pallette_offset(uint material_index) {
    switch (material_index) {
        case 1: // Dirt
            return vec2(0, 2);
        case 2: // Grass
            return vec2(0, 3);
        case 3: // Stone
            return vec2(1, 3);
        case 5: // Sand
            return vec2(2, 3);
        case 6: // Ore
            return vec2(3, 3);
        default: // The last pallette element contains some invalid texture
            return vec2(MAX_PALLETTE_OFFSET, MAX_PALLETTE_OFFSET);
    }
}

const float PALLETTE_TILE_SIZE = 1.0 / PALLETTE_SIZE; 
vec4 sample_pallette(uint material_index, float u, float v) {
    vec2 pallette_offset = get_pallette_offset(material_index);
    vec2 tex_coords = ( PALLETTE_TILE_SIZE * pallette_offset ) + vec2(u, v);

    vec4 tex_color = texture(block_pallette, tex_coords);
    return tex_color;
}

vec4 sample_blended_texture(float u, float v) {
    vec3 color = vec3(0.0, 0.0, 0.0);

    for (int i = 0; i < 4; i++) {
        uint material_index = uint(blend_indices[i]);
        if (material_index == 0) {
            continue;
        }

        float material_weight = blend_coefficients[i];

        vec3 texture_color = sample_pallette(material_index, u, v).rgb;
        color += material_weight * texture_color;
    }

    return vec4(color, 1.0);
}

const int AXIS_BLEND_SMOOTHNESS = 8;
const float SMOOTHNESS_ROOT = 1.0 / AXIS_BLEND_SMOOTHNESS;
vec3 get_projection_coefficients(vec3 position_abs, vec3 normal) {
    float abs_x = abs(normal.x);
    float abs_y = abs(normal.y);
    float abs_z = abs(normal.z);

    vec3 np = vec3(
        pow(abs_x, AXIS_BLEND_SMOOTHNESS), 
        pow(abs_y, AXIS_BLEND_SMOOTHNESS),
        pow(abs_z, AXIS_BLEND_SMOOTHNESS));

    float pnorm = pow(np.x + np.y + np.z, SMOOTHNESS_ROOT);

    float alpha = np.x / pnorm;
    float beta = np.y / pnorm;
    float gamma = np.z / pnorm;

    return vec3(alpha, beta, gamma);
}

const float TILE_SIZE = 16.0;
vec4 assemble_color(vec3 world_position, vec3 normal) {
    float x = PALLETTE_TILE_SIZE * mod(abs(world_position.x), TILE_SIZE) / TILE_SIZE;
    float y = PALLETTE_TILE_SIZE * mod(abs(world_position.y), TILE_SIZE) / TILE_SIZE;
    float z = PALLETTE_TILE_SIZE * mod(abs(world_position.z), TILE_SIZE) / TILE_SIZE;

    vec3 position_abs = vec3(abs(world_position.x), 
                             abs(world_position.y), 
                             abs(world_position.z));
    vec3 projection_coefficients = get_projection_coefficients(position_abs, normal);
    float alpha = projection_coefficients.x;
    float beta = projection_coefficients.y;
    float gamma = projection_coefficients.z;

    vec4 texture_color = alpha * sample_blended_texture(x, y)
        + beta * sample_blended_texture(x, z)
        + gamma * sample_blended_texture(y, z);

    return texture_color;
}

void main() {  
    v_normal = normalize(normal);
    fragment_position = vec3(model * vec4(position, 1.0));
    fragment_color = assemble_color(fragment_position, v_normal);

    gl_Position = projection * view * model * vec4(position, 1.);
}

