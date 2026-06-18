#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) face_id: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) face_id: u32,
};

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(in.instance_index),
        vec4<f32>(in.position, 1.0),
    );
    out.face_id = in.face_id;

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    switch in.face_id {
        case 0u: {
            return vec4<f32>(1.0, 0.0, 0.0, 1.0); // red
        }
        case 1u: {
            return vec4<f32>(0.0, 1.0, 0.0, 1.0); // green
        }
        case 2u: {
            return vec4<f32>(0.0, 0.0, 1.0, 1.0); // blue
        }
        default: {
            return vec4<f32>(1.0, 1.0, 1.0, 1.0); // white
        }
    }
}