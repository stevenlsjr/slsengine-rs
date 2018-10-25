
layout (location = 0) in vec3 v_pos;
layout (location = 1) in vec3 v_normal;

uniform mat4 projection;
uniform mat4 modelview;

void main(){
    gl_Position = projection * modelview * vec4(v_pos, 1.0);
}