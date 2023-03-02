in vec3 v_normal;

out vec3 frag_color;

void main() {
    vec3 color = vec3(0.6, 0.6, 0.6);
    vec3 light_dir = vec3(0., -1., -0.5);

    float diffusion = dot(v_normal, -light_dir);

    frag_color = color * diffusion;
}
