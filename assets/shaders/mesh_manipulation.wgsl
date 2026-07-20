@group(0) @binding(0) var<storage, read_write> vertices: array<Vertex>;

@group(0) @binding(1) var<uniform> config: MeshManipulationUniforms;

@group(0) @binding(2) var<storage, read_write> debug: array<u32>;

struct Vertex {
    position_u: vec4<f32>,
    normal_v: vec4<f32>,
}

struct MeshManipulationUniforms {
    subdivisions_x: u32,
    subdivisions_z: u32,
    vertex_count: u32,
    animation_progress: f32,
}

const WORKGROUP_SIZE = 32;

@compute @workgroup_size(WORKGROUP_SIZE,WORKGROUP_SIZE,1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>) {

    let x = invocation_id.x;
    let y = invocation_id.y;
    let index = y * WORKGROUP_SIZE + x;

    vertices[index].position_u.y = calc_height(x, 0.3);;
}

@compute @workgroup_size(WORKGROUP_SIZE,WORKGROUP_SIZE,1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let x = invocation_id.x;
    let y = invocation_id.y;
    let index = y * WORKGROUP_SIZE + x;

    vertices[index].position_u.y = calc_height(x, 0.1);
}

fn calc_height(x: u32, amplitude: f32) -> f32 {
    const PI_2 = 2 * 3.1415;
    const wave_count = 3;
    let x_norm = f32(x) / f32(WORKGROUP_SIZE);
    return amplitude * sin(PI_2 * x_norm * wave_count + PI_2 * config.animation_progress);
}
