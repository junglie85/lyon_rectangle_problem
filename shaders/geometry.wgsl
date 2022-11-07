// Vertex

struct ViewProjection {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> view_projection: ViewProjection;

struct VertexOutput {
    @location(0) v_color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) a_position: vec3<f32>,
    @location(1) a_color: vec4<f32>,
) -> VertexOutput {
    // return VertexOutput(a_color, vec4<f32>(a_position, 1.0));
    var clip_position = view_projection.projection * view_projection.view * vec4<f32>(a_position, 1.0);
    return VertexOutput(a_color, clip_position);
}


// Fragment

struct Output {
    @location(0) out_color: vec4<f32>,
};

@fragment
fn fs_main(@location(0) v_color: vec4<f32>) -> Output {
    return Output(v_color);
}
