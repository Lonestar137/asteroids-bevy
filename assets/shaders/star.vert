// Vertex shader (e.g., star.vert)
#version 450

layout(location = 0) in vec2 Vertex_Position;

void main() {
    gl_Position = vec4(Vertex_Position, 0.0, 1.0);
}
