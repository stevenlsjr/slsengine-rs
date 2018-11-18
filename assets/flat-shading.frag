out vec4 out_color;

in vec3 frag_pos;
in vec2 frag_uv;
in vec3 frag_normal;

uniform sampler2D u_texture;

void main(){
    vec4 color = texture(u_texture, frag_uv);
    if (frag_pos.y < 0.0){
        out_color = vec4(1.0, 1.0, 1.0, 1.0);
    } else {
        out_color = color;
    }
}