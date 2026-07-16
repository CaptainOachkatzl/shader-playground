//! Very simple shader used to demonstrate how to get the world position and pass data
//! between the vertex and fragment shader. Also shows the custom vertex layout.

// First we import everything we need from bevy_pbr
// A 2D shader would be very similar but import from bevy_sprite instead
#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip,
    mesh_view_bindings::view,
}

struct Vertex {
    // This is needed if you are using batching and/or gpu preprocessing
    // It's a built in so you don't need to define it in the vertex layout
    @builtin(instance_index) instance_index: u32,
    // Like we defined for the vertex layout
    // position is at location 0
    @location(0) position: vec3<f32>,
    // and color at location 1
    @location(1) color: vec4<f32>,
};

// This is the output of the vertex shader and we also use it as the input for the fragment shader
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) color: vec3<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = billboard_position(vertex.position, vertex.instance_index);
    out.color = vertex.color.rgb;
    return out;
}

// billboard shader to always draw mesh from frontal view
fn billboard_position(vertex_position: vec3<f32>, instance_index: u32) -> vec4<f32> {
    let clip_from_world = view.clip_from_world;
    let camera_right = normalize(vec3<f32>(clip_from_world[0].x, clip_from_world[1].x, clip_from_world[2].x));
    let camera_up = normalize(vec3<f32>(clip_from_world[0].y, clip_from_world[1].y, clip_from_world[2].y));

    let world_space = camera_right * vertex_position.x + camera_up * vertex_position.y;
    return view.clip_from_world * mesh_functions::get_world_from_local(instance_index) * vec4<f32>(world_space, 1.);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // output the color directly
    return vec4(in.color, 1.0);
}
