
in vec3 frag_uv;

out vec4 out_color;

void main(){
    vec3 color = (frag_uv / 2.0) + 0.5;
    out_color = vec4(color, 1.0);
}