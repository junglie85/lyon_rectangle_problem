// Vertex

struct Globals {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
};

struct Primitive {
    color: vec4<f32>,
    translate: vec2<f32>,
    scale: vec2<f32>,
    rotate: f32,
    z_index: i32,
    width: f32,
    pad_1: i32,
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
    // Stroke primitives start at index 0 and are every other entry => 0 2 4 6 ... => primitive_id + 2 * instance_id
    // Fill primitives start at index 1 and are every other entry =>   1 3 5 7 ... => primitive_id + 2 * instance_id
    var primitive: Primitive = primitives.primitives[primitive_id + u32(2 * i32(instance_idx))];

    var rotation = mat2x2<f32>(
        vec2<f32>(cos(primitive.rotate), -sin(primitive.rotate)),
        vec2<f32>(sin(primitive.rotate), cos(primitive.rotate))
    );

    var local_position = (position * primitive.scale + normal * primitive.width) * rotation;
    var world_position = local_position + primitive.translate;
    var transformed_position = globals.projection * globals.view * vec4<f32>(world_position, 0.0, 1.0);
    
    var z = f32(primitive.z_index) / 1000.0;
    var clip_position = vec4<f32>(transformed_position.x, transformed_position.y, z, 1.0);
    
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
