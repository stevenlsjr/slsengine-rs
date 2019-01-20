#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(set = 0, binding = 0) uniform MatrixData
{
  mat4 modelview;
  mat4 projection;
  mat4 normal;
}
m;

layout(location = 0) out vec3 frag_normal; 


void
main()
{
  frag_normal = normal;
  gl_Position = m.projection * m.modelview * vec4(position, 1.0);
}