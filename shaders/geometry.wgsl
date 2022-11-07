// Vertex

struct VertexOutput {
    @location(0) v_color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) a_position: vec3<f32>,
    @location(1) a_color: vec4<f32>,
) -> VertexOutput {
    return VertexOutput(a_color, vec4<f32>(a_position, 1.0));
}


// Fragment

struct Output {
    @location(0) out_color: vec4<f32>,
};

@fragment
fn fs_main(@location(0) v_color: vec4<f32>) -> Output {
    return Output(v_color);
}
