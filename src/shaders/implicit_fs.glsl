#version 400

in vec3 v_normal;
in vec3 fragment_position;
in mat4 blend_weights;
//in vec4 fragment_color;
//in vec4 blend_weights;
//in vec4 blend_materials;

uniform sampler2D block_pallette;

out vec4 color;

float ambience_strength = 0.1;
vec3 ambience_color = vec3(0.3, 0.3, 0.4);

vec3 light_position = vec3(0.0, 500.0, 0.0);
vec3 light_color = vec3(0.812, 0.847, 0.843);

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

const float EPSILON = 0.0001;
vec4 sample_blended_texture(float u, float v) {
    vec3 color = vec3(0.0, 0.0, 0.0);

    for (int col = 0; col < 4; col++) {
        for (int row = 0; row < 4; row++) {
            uint material_index = col * 4 + row;
            float weight = blend_weights[col][row];

            if (weight > EPSILON) {
                vec3 texture_color = sample_pallette(material_index, u, v).rgb;
                color += weight * texture_color;
            }
        }
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

const float TILE_SIZE = 0.5;
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

    vec4 texture_color = gamma * sample_blended_texture(x, y)
        + beta * sample_blended_texture(x, z)
        + alpha * sample_blended_texture(y, z);

    return vec4(texture_color.rgb, 1.0);
}

void main() {
    vec3 light_dir = normalize(light_position - fragment_position);

    float diffusion_coefficient = max(dot(v_normal, light_dir), 0.0);
    vec3 diffusion = diffusion_coefficient * light_color;

    vec3 ambience = ambience_strength * ambience_color;

    vec4 texture_color = assemble_color(fragment_position, v_normal);

    color = vec4((ambience + diffusion), 1.0) * texture_color;

    // Debug normals
    //color = vec4(v_normal, 1.0);
}
