// Vertex

struct Globals {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
};

struct Primitive {
    color: vec4<f32>,
    z_index: i32,
    width: f32,
    pad_1: i32,
    pad_2: i32,
};

struct Primitives {
    primitives: array<Primitive, 256>,
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var<uniform> primitives: Primitives;

struct VertexOutput {
    @location(0) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) normal: vec2<f32>,
    @location(2) primitive_id: u32,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    var primitive: Primitive = primitives.primitives[primitive_id + instance_idx];

    var local_position = position + normal * primitive.width;

    var world_position = globals.projection * globals.view * vec4<f32>(local_position, 0.0, 1.0);
    
    var z = f32(primitive.z_index) / 1000.0;
    var clip_position = vec4<f32>(world_position.x, world_position.y, z, 1.0);
    
    return VertexOutput(primitive.color, clip_position);
}


// Fragment

struct Output {
    @location(0) color: vec4<f32>,
};

@fragment
fn fs_main(@location(0) color: vec4<f32>) -> Output {
    return Output(color);
}
