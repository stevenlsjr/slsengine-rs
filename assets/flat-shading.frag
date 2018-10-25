
out vec4 out_color;
void main(){
    float r = sin(gl_FragCoord.x / 10.0) / 0.5 + 0.5;
    float g = sin(gl_FragCoord.y / 10.0) / 0.5 + 0.5;
    float b = sin(gl_FragDepth / 10.0) / 0.5 + 0.5;
    out_color = vec4(b, b, b, 1.0);
}