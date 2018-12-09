
layout(location = 0) in vec3 v_pos;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec2 v_uv;

layout(binding = 0) uniform Matrices
{
  mat4 projection;

  mat4 modelview;
  mat4 normal_matrix;
};

out vec2 frag_uv;
out vec3 frag_normal;
out vec3 frag_pos;
out vec3 frag_eye_normal;

void
main()
{
  frag_uv = v_uv;
  frag_normal = v_normal;
  frag_eye_normal = normalize((normal_matrix * vec4(v_normal, 0.0)).xyz);
  vec4 eye = modelview * vec4(v_pos, 1.0);
  gl_Position = projection * eye;
  frag_pos = eye.xyz;
}