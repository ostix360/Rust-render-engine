#version 330 core

in vec3 position;

uniform mat4 transformation_matrix;
uniform mat4 projection_matrix;
uniform mat4 view_matrix;


void main() {
    gl_Position = projection_matrix * view_matrix * transformation_matrix * vec4(position, 1.0);
}
