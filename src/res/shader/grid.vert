#version 330 core

in vec3 position;
uniform mat4 transformation_matrix;
uniform mat4 projection_matrix;
uniform mat4 view_matrix;

float f(vec3 pos) {
    return pos.x * cos(pos.y);
}
float g(vec3 pos) {
    return pos.x * sin(pos.y);
}
float h(vec3 pos) {
    return pos.z;
}

vec3 coordinate_transform(vec3 pos) {
    float x = f(pos);
    float y = g(pos);
    float z = h(pos);
    return vec3(x, y, z);
}

void main() {
    gl_Position = projection_matrix * view_matrix * vec4(coordinate_transform((transformation_matrix * vec4(position, 1.0)).xyz), 1.0);
}
