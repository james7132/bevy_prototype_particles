#version 450

layout(set = 1, binding = 1) in Particles_color {
    vec4 Color;
};

layout(set = 2, binding = 1) in ParticleMaterial_texture {
    sampler2d mainTexture;
};

void main() {
    gl_Position = vec4(aPos + aOffset, 0.0, 1.0);
    fColor = aColor;
}
