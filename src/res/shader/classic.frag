#version 330 core

uniform vec4 color;

out vec4 FragColor;

void main() {
//    vec2 p = gl_PointCoord * 2.0 - 1.0; // [-1,1]
//    if (dot(p,p) > 1.0) discard;
    FragColor = color;
}
