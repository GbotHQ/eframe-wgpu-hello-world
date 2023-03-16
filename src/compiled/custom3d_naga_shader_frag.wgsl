struct FragmentOutput {
    @location(0) out_color: vec4<f32>,
}

var<private> v_color_1: vec4<f32>;
var<private> out_color: vec4<f32>;

fn main_1() {
    let _e2 = v_color_1;
    out_color = _e2;
    return;
}

@fragment 
fn main(@location(0) v_color: vec4<f32>) -> FragmentOutput {
    v_color_1 = v_color;
    main_1();
    let _e7 = out_color;
    return FragmentOutput(_e7);
}
