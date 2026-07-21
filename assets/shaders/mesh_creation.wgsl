
struct MeshCreationUniforms {
    num_vertices: u32,
    num_indices: u32,
    input_vertex_start: u32,
    input_vertex_end: u32,
    input_index_start: u32,
    input_index_end: u32,
    output_vertex_start: u32,
    output_vertex_end: u32,
    output_index_start: u32,
    output_index_end: u32,
}

struct Vertex {
    position: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
};

@group(0) @binding(0)
var<storage, read_write> vertices: array<Vertex>;

@group(0) @binding(1)
var<storage, read_write> indices: array<u32>;

@group(0) @binding(2)
var<uniform> config: MeshCreationUniforms;

@group(0) @binding(3)
var<storage, read_write> debug: array<u32>;

@compute @workgroup_size(1,1,1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    for (var i = 0; i < 24; i++) {
    }

    for (var i = 0; i < 36; i++) {
    }
}
