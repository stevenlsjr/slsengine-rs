
out vec4 out_color;
void main(){
    float r = sin(gl_FragCoord.x / 10.0) / 0.5 + 0.5;
    float g = sin(gl_FragCoord.y / 10.0) / 0.5 + 0.5;
    out_color = vec4(1.0, 1.0, 0.0, 1.0);
}