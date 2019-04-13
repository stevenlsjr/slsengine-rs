#version 450

layout(location = 0) in vec3 v_position;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec2 v_uv;
layout(location = 3) in vec3 v_tangent;
layout(location = 4) in vec3 v_bitangent;

layout(set = 0, binding = 0) uniform MatrixData
{
  mat4 modelview;
  mat4 projection;
  mat4 normal;
}
m;

layout(location = 0) out vec2 frag_uv; 
layout(location = 1) out vec3 frag_pos; 
layout(location = 2) out mat3 frag_tbn_matrix; 

mat3 make_tbn_matrix(vec3 eye_normal){
    vec3 eye_t = normalize(vec3(m.normal * vec4(v_tangent, 0.0)));
    vec3 eye_b = normalize(vec3(m.normal * vec4(v_bitangent, 0.0)));
    return mat3(eye_t, eye_b, normalize(eye_normal));
}

void
main()
{
  vec3 eye_normal = vec3(m.normal * vec4(v_normal, 0.0));
  frag_tbn_matrix = make_tbn_matrix(eye_normal);
  vec4 eye_pos = m.modelview * vec4(v_position, 1.0);
  frag_pos = eye_pos.xyz;
  frag_uv = v_uv;
  gl_Position = m.projection * eye_pos;
}