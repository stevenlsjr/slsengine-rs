layout(location = 0) in vec3 v_pos;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec2 v_uv;

uniform MatricesBlock
{
  mat4 projection;
  mat4 modelview;
};

out vec2 frag_uv;
out vec3 frag_normal;

void
main()
{
  frag_uv = v_uv;
  frag_normal = v_normal;
  gl_Position = projection * modelview * vec4(v_pos, 1.0);
}