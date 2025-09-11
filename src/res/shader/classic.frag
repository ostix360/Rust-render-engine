#version 330 core

uniform mat4 transformation_matrix;

out vec4 FragColor;

void main() {
    FragColor = vec4(0.8, 0.5, 0.6, 1.0);
}
