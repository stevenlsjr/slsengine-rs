#version 450

layout(location = 0) in vec3 frag_normal;

layout(location = 0) out vec4 out_color;

void
main()
{
  vec3 N = normalize(frag_normal);
  if (gl_FrontFacing) {
    out_color = vec4((frag_normal + 0.5) * 0.5, 1.0);
  } else {
    out_color = vec4(1.0, 0.0, 1.0, 1.0);
  }
}