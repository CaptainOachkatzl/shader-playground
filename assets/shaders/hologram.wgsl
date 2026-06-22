#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(in.instance_index),
        vec4<f32>(in.position, 1.0),
    );

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0, 1, 0.5, 0.3);
}