layout(location = 0) in vec3 v_pos;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec2 v_uv;

layout(binding = 0) uniform Matrices
{
  mat4 projection;

  mat4 modelview;
  mat4 normal_matrix;
};

// note: uv for cubmap
out vec3 frag_cubemap_uv;
out vec3 frag_normal;
out vec3 frag_pos;
out vec3 frag_eye_normal;


void main(){
    mat3 rotation_mat = mat3(modelview);
    frag_cubemap_uv = v_pos;
    vec3 eye = rotation_mat * v_pos;
    gl_Position = (projection * vec4(eye, 1.0)).xyww;
}