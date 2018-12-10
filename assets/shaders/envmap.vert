
layout (location = 0) in vec3 v_pos;
layout (location = 1) in vec3 v_normal;
layout (location = 2) in vec2 v_uv;

uniform mat4 projection;

uniform mat4 modelview;
uniform mat4 normal_matrix;

out vec3 frag_uv;

void main(){
    mat3 rotation_mat = mat3(modelview);
    // frag_uv = v_pos;
    vec3 eye = rotation_mat * v_pos;
    gl_Position = (projection * vec4(eye, 1.0)).xyww;
}