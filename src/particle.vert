#version 450
layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec2 Vertex_Uv;
layout(location = 2) in vec3 Particles_position;
layout(location = 3) in float Particles_rotation;
layout(location = 4) in float Particles_size;

layout(set = 0, binding = 0) uniform CameraViewProj { mat4 ViewProj; };

void main() {
    mat4 translation = mat4(1.0);
    mat4 rotation = mat4(1.0);
    mat4 scale = mat4(1.0);

    // Set up translation matrix
    translation[3] = vec4(Particles_position.xyz, 1.0);

    // Set up rotation matrix
    rotation[0][0] = cos(Particles_rotation);
    rotation[0][1] = sin(Particles_rotation);
    rotation[1][0] = -sin(Particles_rotation);
    rotation[1][1] = cos(Particles_rotation);

    // Set up scale matrix
    scale[0][0] = Particles_size;
    scale[1][1] = Particles_size;
    scale[2][2] = Particles_size;

    gl_Position = ViewProj * (translation * rotation * scale) * vec4(Vertex_Position, 1.0);
}
