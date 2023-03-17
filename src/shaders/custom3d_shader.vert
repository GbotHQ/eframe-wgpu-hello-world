const vec2 verts[3] = vec2[3](
    vec2(0.0, 1.0),
    vec2(-1.0, -1.0),
    vec2(1.0, -1.0)
);
const vec4 colors[3] = vec4[3](
    vec4(1.0, 0.0, 0.0, 1.0),
    vec4(0.0, 1.0, 0.0, 1.0),
    vec4(0.0, 0.0, 1.0, 1.0)
);

out vec4 v_color;
layout(binding=0) uniform AngleBlock {
    float u_angle;
};

void main() {
    v_color = colors[gl_VertexIndex];
    gl_Position = vec4(verts[gl_VertexIndex], 0.0, 1.0);
    gl_Position.x *= cos(u_angle);
}
