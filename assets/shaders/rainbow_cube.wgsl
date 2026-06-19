#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

struct Material {
    // a value between 0.0 and 1.0 to indicate how far the animation has progressed
    animation_progress: f32,
};
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: Material;

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_y: f32,
};

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(in.instance_index),
        vec4<f32>(in.position, 1.0),
    );

    out.local_y = in.position.y;

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    const frequency: f32 = 10.0;
    let normalized_y = in.local_y + 0.5;
    let hsv = vec3<f32>((normalized_y - material.animation_progress), 1, 1);
    return vec4<f32>(hsv_to_rgb(hsv), 1.0);
}

fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let h = hsv.x;
    let s = hsv.y;
    let v = hsv.z;

    let k = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);

    let p = abs(fract(vec3<f32>(h) + k.xyz) * 6.0 - vec3<f32>(k.w));

    return v * mix(
        vec3<f32>(1.0),
        clamp(p - vec3<f32>(1.0), vec3<f32>(0.0), vec3<f32>(1.0)),
        vec3<f32>(s)
    );
}