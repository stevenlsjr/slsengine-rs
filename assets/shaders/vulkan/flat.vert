#version 450

layout(location = 0) in vec3 position;
layout(set = 0, binding = 0) uniform MatrixData
{
  mat4 modelview;
  mat4 projection;
  mat4 normal;
}
m;

void
main()
{
  gl_Position = m.projection * m.modelview * vec4(position, 1.0);
}