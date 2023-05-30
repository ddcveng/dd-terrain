#version 400

in vec3 v_normal;
in vec3 fragment_position;
in vec4 fragment_color;

out vec4 color;

float ambience_strength = 0.1;
vec3 ambience_color = vec3(0.3, 0.3, 0.4);

vec3 light_position = vec3(0.0, 500.0, 0.0);
vec3 light_color = vec3(0.812, 0.847, 0.843);

void main() {
    vec3 light_dir = normalize(light_position - fragment_position);

    float diffusion_coefficient = max(dot(v_normal, light_dir), 0.0);
    vec3 diffusion = diffusion_coefficient * light_color;

    vec3 ambience = ambience_strength * ambience_color;

    vec4 frag_color = vec4((ambience + diffusion), 1.0) * fragment_color;

    color = frag_color;
}
