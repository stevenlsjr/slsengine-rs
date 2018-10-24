
layout (location = 0) in vec3 v_pos;

uniform mat4 projection;
uniform mat4 modelview;

void main(){
    gl_Position = modelview * vec4(v_pos, 1.0);
}