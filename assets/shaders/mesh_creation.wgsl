
struct MeshCreationUniforms {
    num_vertices: u32,
    vertex_start: u32,
    vertex_end: u32,
    index_start: u32,
    index_end: u32,
}

struct Vertex {
    position: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
};

@group(0) @binding(0)
var<storage, read_write> input_vertices: array<Vertex>;

@group(0) @binding(1)
var<storage, read_write> input_indices: array<u32>;

@group(0) @binding(2)
var<storage, read_write> output_vertices: array<Vertex>;

@group(0) @binding(3)
var<storage, read_write> output_indices: array<u32>;

@group(0) @binding(4)
var<uniform> config: MeshCreationUniforms;

@group(0) @binding(5)
var<storage, read_write> debug: array<u32>;

@compute @workgroup_size(1,1,1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    for (var i = 0; i < 24; i++) {
        output_vertices[i] = input_vertices[i];
    }

    for (var i = 0; i < 36; i++) {
        output_indices[i] = input_indices[i];
    }
}
