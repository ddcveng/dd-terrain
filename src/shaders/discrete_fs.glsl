#version 400

uniform sampler2D block_pallette;

in vec2 texture_uv;
in vec3 v_normal;

out vec4 color;

void main() {
    vec3 light_dir = normalize(vec3(-2. ,3., 2.));
    float diffusion = dot(v_normal, light_dir);

    vec4 frag_color = texture(block_pallette, texture_uv);
    color = vec4(frag_color.rgb * diffusion, 1.0);
}
