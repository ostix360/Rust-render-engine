#version 330 core

uniform mat4 transformation_matrix;

uniform vec3 segment_color;

out vec4 FragColor;

void main() {
    FragColor = vec4(1.0);
}
