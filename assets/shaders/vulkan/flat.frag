#version 450

layout(location = 0) in vec2 frag_uv; 
layout(location = 1) in vec3 frag_pos; 
layout(location = 2) in mat3 frag_tbn_matrix;

layout(location = 0) out vec4 out_color;

void
main()
{
  vec3 N = normalize(frag_tbn_matrix[2]);
  out_color = vec4((N + 0.5) * 0.5, 1.0);
}