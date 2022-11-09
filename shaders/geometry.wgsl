// Vertex

struct Globals {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;

struct VertexOutput {
    @location(0) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
) -> VertexOutput {
    var clip_position = globals.projection * globals.view * vec4<f32>(position, 1.0);
    
    return VertexOutput(color, clip_position);
}


// Fragment

struct Output {
    @location(0) color: vec4<f32>,
};

@fragment
fn fs_main(@location(0) color: vec4<f32>) -> Output {
    return Output(color);
}
