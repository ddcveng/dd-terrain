#version 400

in vec3 frag_color;
in vec3 v_normal;

out vec4 color;

void main() {
    vec3 light_dir = normalize(vec3(-2. ,3., 2.));
    float diffusion = dot(v_normal, light_dir);

    color = vec4(frag_color * diffusion, 1.0);
}
