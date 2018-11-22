out vec4 out_color;

in vec3 frag_pos;
in vec2 frag_uv;
in vec3 frag_normal;

uniform sampler2D u_texture;
const vec3 light_dir = vec3(0.1, 1.0, -0.5);
const float ambient_factor = 0.1;
const float diffuse_factor = 1.0 - ambient_factor;

void main(){
    float d = dot(light_dir, frag_normal);
    d = max(d, 0.0);
    vec4 albedo = texture(u_texture, frag_uv);

    vec3 diffuse = d * albedo.xyz * diffuse_factor;
    vec3 ambient = albedo.xyz * ambient_factor;
    out_color = vec4(ambient + diffuse, albedo.w);
}