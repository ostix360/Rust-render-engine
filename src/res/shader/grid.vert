#version 330 core

in vec3 position;
uniform mat4 transformation_matrix;
uniform mat4 projection_matrix;
uniform mat4 view_matrix;
uniform float tangent_mix;
uniform vec3 tangent_anchor_abstract;
uniform vec3 tangent_basis_x;
uniform vec3 tangent_basis_y;
uniform vec3 tangent_basis_z;
uniform float tangent_position_scale;

float f(vec3 pos) {
//    return pos.x;
    return pos.x * cos(pos.y) * sin(pos.z);
}
float g(vec3 pos) {
//    return pos.y;
    return pos.x * sin(pos.y) * sin(pos.z);
}
float h(vec3 pos) {
//    return pos.z;
    return pos.x * cos(pos.z);
}

vec3 coordinate_transform(vec3 pos) {
    float x = f(pos);
    float y = g(pos);
    float z = h(pos);
    return vec3(x, y, z);
}

vec3 tangent_transform(vec3 pos) {
    vec3 delta = (pos - tangent_anchor_abstract) * tangent_position_scale;
    return tangent_basis_x * delta.x
        + tangent_basis_y * delta.y
        + tangent_basis_z * delta.z;
}

void main() {
    vec3 abstract_pos = (transformation_matrix * vec4(position, 1.0)).xyz;
    vec3 world_pos = coordinate_transform(abstract_pos);
    vec3 tangent_pos = tangent_transform(abstract_pos);
    vec3 final_pos = mix(world_pos, tangent_pos, tangent_mix);
    gl_Position = projection_matrix * view_matrix * vec4(final_pos, 1.0);
}
