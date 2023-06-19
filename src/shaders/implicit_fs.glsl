#version 400

in vec3 v_normal;
in vec3 fragment_position;
in mat4 blend_weights;

uniform sampler2D block_pallette;
uniform vec3 sun_position;

out vec4 fragment_color;

float ambience_strength = 0.0;
vec3 ambience_color = vec3(0.3, 0.3, 0.4);

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

const vec2 TILE_RESOLUTION = vec2(16, 16);
const vec2 PALLETTE_RESOLUTION = PALLETTE_SIZE * TILE_RESOLUTION;
vec2 nearest_pixel_filter(vec2 uv) {
    vec2 pixel = uv * PALLETTE_RESOLUTION;
    pixel = floor(pixel) + 0.5;

    return pixel / PALLETTE_RESOLUTION;
}

const float PALLETTE_TILE_SIZE = 1.0 / PALLETTE_SIZE;
// Texture coords u, v are indexes into a single tile in the pallette
// and are from range 0.0 to 1.0
vec4 sample_pallette(uint material_index, float u, float v) {
    vec2 pallette_offset = get_pallette_offset(material_index);

    vec2 tile_base = PALLETTE_TILE_SIZE * pallette_offset;
    vec2 tile_coords = PALLETTE_TILE_SIZE * vec2(u, v);

    vec2 tex_coords = tile_base + tile_coords;

    tex_coords = nearest_pixel_filter(tex_coords);

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

// With increasing value of AXIS_BLEND_SMOOTHNESS
// the length of the normal vector gets smaller as well as the sum of the coefficients
// This makes surfaces that are at an angle look dark regardless of lighting 
// and doesn't look very nice. Plus it doesn't make much sense, what is this code trying to achieve?
const int AXIS_BLEND_SMOOTHNESS = 8; 
const float SMOOTHNESS_ROOT = 1.0 / AXIS_BLEND_SMOOTHNESS;
vec3 get_projection_coefficients_arches(vec3 normal) {
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

// This implementation keeps the sum of coefficients equal to 1.0
// This way no matter the angle, the brightness of the texture  stays the same
// Note: this is effectively equal to the arches implementation with AXIS_BLEND_SMOOTHNESS=1;
vec3 get_projection_coefficients(vec3 normal) {
    vec3 positive_n = abs(normal);

    float coord_sum = positive_n.x + positive_n.y + positive_n.z;

    return positive_n / coord_sum;
}

vec4 assemble_color(vec3 world_position, vec3 normal) {
    float x = fract(world_position.x);
    float y = fract(world_position.y);
    float z = fract(world_position.z);

    vec3 projection_coefficients = get_projection_coefficients(normal);
    float alpha = projection_coefficients.x;
    float beta = projection_coefficients.y;
    float gamma = projection_coefficients.z;

    // alpha and gamma are swapped in contrast to the version used in the arches paper
    // since their version doesn't make sense
    // 
    // alpha, beta, gamma correspond to the x,y,z coordinates of the normal respectively
    // each coefficient controls the plane perpendicular to its axis 
    vec4 texture_color = gamma * sample_blended_texture(x, y)
        + beta * sample_blended_texture(x, z)
        + alpha * sample_blended_texture(y, z);

    // Visualize the darkening caused by the arches coefficients with p > 1
    //    float xx = 1.0 - (alpha + beta + gamma);
    //    if (xx > 0.2) {
    //        texture_color = vec4(normal, 1.0);
    //    }

    return vec4(texture_color.rgb, 1.0);
}

void main() {
    vec3 sunlight_dir = normalize(sun_position - fragment_position);
    float sun_factor = clamp(dot(v_normal, sunlight_dir), 0.0, 1.0);
    vec3 sunlight = sun_factor * sun_color;

    vec3 skylight_dir = vec3(0.0, 1.0, 0.0); // Light coming directly from above
    float sky_factor = 0.5 + 0.5 * v_normal.y;
    vec3 skylight = sky_factor * sky_color;

    vec3 indirect_light_dir = normalize(vec3(-sunlight_dir.x, 0.0, -sunlight_dir.z));
    float indirect_coefficient = clamp(dot(v_normal, indirect_light_dir), 0.0, 1.0);
    vec3 indirect = indirect_coefficient * indirect_color;

    vec3 lighting = sunlight + skylight + indirect;

    vec4 texture_color = assemble_color(fragment_position, v_normal);
    vec3 diffuse_color = texture_color.rgb;

    vec3 color = lighting * diffuse_color;

    // Debug normals
    // color = v_normal;

    // The output is gamma corrected automatically - GL_FRAMEBUFFER_SRGB
    fragment_color = vec4(color, 1.0);
}
