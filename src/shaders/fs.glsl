#version 330

in vec3 frag_color;
in vec3 v_normal;

out vec3 color;

void main() {
    vec3 light_dir = normalize(vec3(-2. ,3., 2.));
    float diffusion = dot(v_normal, light_dir);

    color = frag_color * diffusion;
}
