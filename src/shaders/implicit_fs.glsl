#version 400

in vec3 v_normal;
in vec3 fragment_position;
in mat4 blend_weights;
//in vec4 fragment_color;
//in vec4 blend_weights;
//in vec4 blend_materials;

uniform sampler2D block_pallette;
uniform vec3 sun_position;

out vec4 fragment_color;

float ambience_strength = 0.0;
vec3 ambience_color = vec3(0.3, 0.3, 0.4);

//vec3 sun_position = vec3(0.0, 500.0, 0.0);
//vec3 sun_color = vec3(1.0, 0.9255, 0.2588);
vec3 sun_color = vec3(1.64, 1.27, 0.99);

const float SKY_COLOR_STRENGTH = 0.1;
//vec3 sky_color = vec3(0.651, 0.8627, 0.9294);
vec3 sky_color = vec3(0.16, 0.20, 0.28);
vec3 indirect_color = vec3(0.4, 0.28, 0.20);

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
        case 4: // Wood
            return vec2(1, 2);
        case 5: // Leaves
            return vec2(2, 2);
        case 6: // Sand
            return vec2(2, 3);
        case 7: // Ore
            return vec2(3, 3);
        default: // The last pallette element contains some invalid texture
            return vec2(MAX_PALLETTE_OFFSET, 0);
    }
}

const float PALLETTE_TILE_SIZE = 1.0 / PALLETTE_SIZE;
vec4 sample_pallette(uint material_index, float u, float v) {
    vec2 pallette_offset = get_pallette_offset(material_index);

    // TODO: calculate tile base positions once and use them as lookup to avoid floating point error
    vec2 tile_base = clamp(PALLETTE_TILE_SIZE * pallette_offset, 0.0, 1.0);

    vec2 tile_coords = clamp(PALLETTE_TILE_SIZE * vec2(u, v), 0.0, PALLETTE_TILE_SIZE);

    vec2 tex_coords = tile_base + tile_coords;
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

const float TILE_SIZE = 1.0;
float get_tiled_pallette_coord(float coord) {
    float tiled_coord = mod(abs(coord), TILE_SIZE); // Get coord in the range 0..TILE_SIZE
    float tiled_coord_normalized = tiled_coord / TILE_SIZE; // Get it back into range 0..1

    return tiled_coord_normalized;
}

vec4 assemble_color(vec3 world_position, vec3 normal) {
    float x = get_tiled_pallette_coord(world_position.x);
    float y = get_tiled_pallette_coord(world_position.y);
    float z = get_tiled_pallette_coord(world_position.z);

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

// All components are in the range [0…1], including hue.
vec3 rgb2hsv(vec3 c)
{
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

// All components are in the range [0…1], including hue.
vec3 hsv2rgb(vec3 c)
{
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void main() {
    vec3 sunlight_dir = normalize(sun_position - fragment_position);

    float sun_factor = clamp(dot(v_normal, sunlight_dir), 0.0, 1.0);
    vec3 sunlight = sun_factor * sun_color;

    vec3 skylight_dir = vec3(0.0, 1.0, 0.0); // Light coming directly from above
    float sky_factor = 0.5 + 0.5 * v_normal.y; //max(dot(v_normal, skylight_dir), 0.0);
    vec3 skylight = sky_factor * sky_color;

    vec3 indirect_light_dir = normalize(vec3(-sunlight_dir.x, 0.0, -sunlight_dir.z));
    float indirect_coefficient = clamp(dot(v_normal, indirect_light_dir), 0.0, 1.0);
    vec3 indirect = indirect_coefficient * indirect_color;

    vec4 texture_color = assemble_color(fragment_position, v_normal);
    //vec3 texture_color_hsv = rgb2hsv(texture_color.rgb);
    //float value = min(texture_color_hsv.b, 0.2);
    //texture_color_hsv = vec3(texture_color_hsv.x, texture_color_hsv.y, value);

    //vec3 diffuse_color = hsv2rgb(texture_color_hsv);
    vec3 diffuse_color = texture_color.rgb;

    //vec3 color = (ambience + diffusion + indirect) * diffuse_color;
    vec3 color = (sunlight + skylight + indirect) * diffuse_color;//vec3(0.2, 0.2, 0.2);

    // Debug normals
    // color = v_normal;

    // The output is gamma corrected automatically - GL_FRAMEBUFFER_SRGB
    fragment_color = vec4(color, 1.0);
}
