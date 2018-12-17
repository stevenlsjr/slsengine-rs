
layout(location = 0) in vec3 v_pos;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec2 v_uv;
layout(location=3) in vec3 v_tangent;
layout(location=4) in vec3 v_bitangent;

uniform mat4 projection;

uniform mat4 modelview;
uniform mat4 normal_matrix;

out vec2 frag_uv;
out vec3 frag_normal;
out vec3 frag_pos;
out vec3 frag_eye_normal;
out mat3 tbn_matrix;

mat3 make_tbn_matrix(vec3 eye_normal){
    vec3 eye_t = normalize(vec3(normal_matrix * vec4(v_tangent, 0.0)));
    vec3 eye_b = normalize(vec3(normal_matrix * vec4(v_bitangent, 0.0)));
    return mat3(eye_t, eye_b, eye_normal);
}

void
main()
{
  frag_uv = v_uv;
  frag_normal = v_normal;
  frag_eye_normal = normalize((normal_matrix * vec4(v_normal, 0.0)).xyz);
  vec4 eye = modelview * vec4(v_pos, 1.0);
  gl_Position = projection * eye;
  frag_pos = eye.xyz;
  tbn_matrix = make_tbn_matrix(frag_eye_normal);
}