
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
    gl_Position = projection * modelview * vec4(v_pos, 1.0);
}