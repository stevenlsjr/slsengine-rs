
out vec4 out_color;

in vec2 frag_uv;
in vec3 frag_normal;

void main(){
    vec3 color = frag_normal * 0.5 + 0.5;
    out_color = vec4(color, 1.0);
}