#version 400

uniform sampler2D block_pallette;

in vec2 texture_uv;
in vec3 v_normal;
in vec3 fragment_position;

out vec4 color;

float ambience_strength = 0.1;
vec3 ambience_color = vec3(0.3, 0.3, 0.4);

vec3 light_position = vec3()

void main() {
    vec3 light_dir = normalize(vec3(-2. ,3., 2.));
    float diffusion = dot(v_normal, light_dir);

    vec4 ambience = vec4(ambience_strength * ambience_color, 1.0);
    vec4 texture_color = texture(block_pallette, texture_uv);

    vec4 frag_color = ambience * texture_color;
    color = frag_color;
}
