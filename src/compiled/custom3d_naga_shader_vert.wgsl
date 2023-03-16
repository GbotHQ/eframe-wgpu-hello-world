struct AngleBlock {
    u_angle: f32,
}

struct VertexOutput {
    @location(0) v_color: vec4<f32>,
    @builtin(position) member: vec4<f32>,
}

const verts: array<vec2<f32>,3u> = array<vec2<f32>,3u>(vec2<f32>(0.0, 1.0), vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0));
const colors: array<vec4<f32>,3u> = array<vec4<f32>,3u>(vec4<f32>(1.0, 0.0, 0.0, 1.0), vec4<f32>(0.0, 1.0, 0.0, 1.0), vec4<f32>(0.0, 0.0, 1.0, 1.0));
var<private> v_color: vec4<f32>;
@group(0) @binding(0) 
var<uniform> global: AngleBlock;
var<private> gl_VertexIndex: u32;
var<private> gl_Position: vec4<f32>;

fn main_1() {
    var local: array<vec4<f32>,3u> = array<vec4<f32>,3u>(vec4<f32>(1.0, 0.0, 0.0, 1.0), vec4<f32>(0.0, 1.0, 0.0, 1.0), vec4<f32>(0.0, 0.0, 1.0, 1.0));
    var local_1: array<vec2<f32>,3u> = array<vec2<f32>,3u>(vec2<f32>(0.0, 1.0), vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0));

    let _e6 = gl_VertexIndex;
    let _e9 = local[_e6];
    v_color = _e9;
    let _e11 = gl_VertexIndex;
    let _e14 = local_1[_e11];
    gl_Position = vec4<f32>(_e14.x, _e14.y, 0.0, 1.0);
    let _e21 = gl_Position;
    _ = global.u_angle;
    let _e24 = global.u_angle;
    gl_Position.x = (_e21.x * cos(_e24));
    return;
}

@vertex 
fn main(@builtin(vertex_index) param: u32) -> VertexOutput {
    gl_VertexIndex = param;
    _ = array<vec2<f32>,3u>(vec2<f32>(0.0, 1.0), vec2<f32>(-(1.0), -(1.0)), vec2<f32>(1.0, -(1.0)));
    _ = array<vec4<f32>,3u>(vec4<f32>(1.0, 0.0, 0.0, 1.0), vec4<f32>(0.0, 1.0, 0.0, 1.0), vec4<f32>(0.0, 0.0, 1.0, 1.0));
    main_1();
    let _e40 = v_color;
    let _e42 = gl_Position;
    return VertexOutput(_e40, _e42);
}
