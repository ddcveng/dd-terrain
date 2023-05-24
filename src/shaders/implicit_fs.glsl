#version 400

in vec3 v_normal;
in vec3 fragment_position;

out vec4 color;

vec3 light_position = vec3(0.0, 500.0, 0.0);
vec3 light_color = vec3(0.51, 0.42, 0.07);

void main() {
    vec3 light_dir = normalize(light_position - fragment_position);
    float diffusion_coefficient = dot(v_normal, light_dir);
    vec3 diffusion = diffusion_coefficient * light_color;

    vec3 frag_color = diffusion * vec3(0.5, 0.5, 0.5);

    color = vec4(frag_color, 1.0);
}
