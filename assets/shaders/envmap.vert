
layout (location = 0) in vec3 v_pos;
layout (location = 1) in vec3 v_normal;
layout (location = 2) in vec2 v_uv;

uniform mat4 projection;

uniform mat4 modelview;
uniform mat4 normal_matrix;

out vec2 frag_uv;
out vec3 frag_normal;
out vec3 frag_pos;
out vec3 frag_eye_normal;

void main(){
    mat3 rotation_mat = mat3(modelview);
    gl_Position = vec4(v_pos.xy, -1.0, 1.0);
    frag_uv = v_uv;
    frag_normal = v_normal;
    frag_eye_normal =  normalize((normal_matrix * vec4(v_normal, 0.0)).xyz);
    vec3 eye = rotation_mat * v_pos;
    gl_Position = vec4(eye, 1.0);
    frag_pos = eye.xyz;
}