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
uniform float tangent_local_radius;

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

bool segment_intersects_local_box(vec3 p0, vec3 p1, float radius) {
    vec3 min_delta = min(p0, p1);
    vec3 max_delta = max(p0, p1);
    return !(max_delta.x < -radius || min_delta.x > radius
        || max_delta.y < -radius || min_delta.y > radius
        || max_delta.z < -radius || min_delta.z > radius);
}

void main() {
    vec3 abstract_pos = (transformation_matrix * vec4(position, 1.0)).xyz;
    vec3 segment_start = (transformation_matrix * vec4(0.0, 0.0, 0.0, 1.0)).xyz;
    vec3 segment_end = (transformation_matrix * vec4(1.0, 0.0, 0.0, 1.0)).xyz;
    vec3 local_start = (segment_start - tangent_anchor_abstract) * tangent_position_scale;
    vec3 local_end = (segment_end - tangent_anchor_abstract) * tangent_position_scale;
    if (tangent_mix > 0.0 && !segment_intersects_local_box(local_start, local_end, tangent_local_radius)) {
        gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
        return;
    }
    vec3 world_pos = coordinate_transform(abstract_pos);
    vec3 tangent_pos = tangent_transform(abstract_pos);
    vec3 final_pos = mix(world_pos, tangent_pos, tangent_mix);
    gl_Position = projection_matrix * view_matrix * vec4(final_pos, 1.0);
}
