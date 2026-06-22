#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}
#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}

// @vertex
// fn vertex(in: VertexInput) -> VertexOutput {
//     var out: VertexOutput;

//     out.clip_position = mesh_position_local_to_clip(
//         get_world_from_local(in.instance_index),
//         vec4<f32>(in.position, 1.0),
//     );

//     return out;
// }

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var out: FragmentOutput;

    var pbr_input = pbr_input_from_standard_material(in, is_front);

    pbr_input.material.base_color = vec4<f32>(0, 1, 0.5, 0.5);

    out.color = apply_pbr_lighting(pbr_input);

    //let color = vec4<f32>(0, 1, 0.5, 0.3)

    return out;
}