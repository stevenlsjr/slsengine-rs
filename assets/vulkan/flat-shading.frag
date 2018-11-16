out vec4 out_color;

in vec2 frag_uv;
in vec3 frag_normal;


uniform layout(binding=0)  sampler2D u_texture;

void
main()
{
  vec4 color = texture(u_texture, frag_uv);
  out_color = color;
}