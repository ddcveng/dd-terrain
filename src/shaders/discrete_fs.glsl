#version 400

uniform sampler2D block_pallette;
uniform vec3 sun_position;

in vec2 texture_uv;
in vec3 v_normal;
in vec3 fragment_position;

out vec4 color;

float ambience_strength = 0.1;
vec3 ambience_color = vec3(0.3, 0.3, 0.4);

vec3 sun_color = vec3(1.64, 1.27, 0.99);
vec3 sky_color = vec3(0.16, 0.20, 0.28);
vec3 indirect_color = vec3(0.4, 0.28, 0.20);

const float SKY_COLOR_STRENGTH = 0.1;

const vec2 TILE_RESOLUTION = vec2(16, 16);
const vec2 PALLETTE_RESOLUTION = 4 * TILE_RESOLUTION;
vec2 nearest_pixel_filter(vec2 uv) {
    vec2 pixel = uv * PALLETTE_RESOLUTION;
    pixel = floor(pixel) + 0.5;

    return pixel / PALLETTE_RESOLUTION;
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

//    vec3 light_dir = normalize(light_position - fragment_position);
//    float diffusion_coefficient = max(dot(v_normal, light_dir), 0.0);
//    vec4 diffusion = vec4(diffusion_coefficient * light_color, 1.0);
//
//    vec4 ambience = vec4(ambience_strength * ambience_color, 1.0);
    vec2 tex_coords = nearest_pixel_filter(texture_uv);
    vec3 texture_color = texture(block_pallette, tex_coords).rgb;

    vec3 frag_color = lighting * texture_color;
    color = vec4(frag_color, 1.0);
}
