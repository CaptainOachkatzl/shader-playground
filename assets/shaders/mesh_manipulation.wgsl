@group(0) @binding(0) var<storage, read_write> vertices: array<f32>;

@group(0) @binding(1) var<storage, read_write> debug: array<u32>;

@group(0) @binding(2) var<uniform> config: MeshManipulationUniforms;

struct Vertex {
    position: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
}

struct MeshManipulationUniforms {
    pane_x_count: u32,
    pane_y_count: u32,
}

@compute @workgroup_size(1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let i = invocation_id.x;
    debug[0] = 1234;

    vertices[1] = 100.0;
}

@compute @workgroup_size(1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    vertices[1] = 100.0;
}
