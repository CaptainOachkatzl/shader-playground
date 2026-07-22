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
    position_x: f32,
    position_y: f32,
    position_z: f32,
    normal_x: f32,
    normal_y: f32,
    normal_z: f32,
    u: f32,
    v: f32,
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

    // create a dent in the front face of a cube
    // this requires an additional 2 triangles

    // copy all input indices to output
    for (var i = 0u; i < config.num_indices; i++) {
        let input = config.input_index_start + i;
        let output = config.output_index_start + i;

        indices[output] = indices[input];
    }

    // in place edit the first 2 triangles
    let index_start = config.output_index_start;
    // bottom
    indices[index_start + 2] = config.num_vertices;
    // top
    indices[index_start + 5] = config.num_vertices + 1;

    // append 2 triangles at the end
    let index_end = config.output_index_start + config.num_indices;
    // right
    indices[index_end] = config.num_vertices + 2;
    indices[index_end + 1] = 1;
    indices[index_end + 2] = 2;
    // left
    indices[index_end + 3] = 3;
    indices[index_end + 4] = 0;
    indices[index_end + 5] = config.num_vertices + 3;

    // copy all input vertices to output
    for (var i = 0u; i < config.num_vertices; i++) {
        let input = config.input_vertex_start + i;
        let output = config.output_vertex_start + i;
        vertices[output] = vertices[input];
    }

    // define normals for each of the denting triangles
    let normal_top = normalize(vec3<f32>(0, -1, 1));
    let normal_bottom = normalize(vec3<f32>(0, 1, 1));
    let normal_left = normalize(vec3<f32>(1, 0, 1));
    let normal_right = normalize(vec3<f32>(-1, 0, 1));

    // adjust normals of front facing vertices since they are now bent inwards
    let vertex_start = config.output_vertex_start;
    vertices[vertex_start].normal_x = normal_bottom.x;
    vertices[vertex_start].normal_y = normal_bottom.y;
    vertices[vertex_start].normal_y = normal_bottom.z;
    vertices[vertex_start + 1].normal_x = normal_top.x;
    vertices[vertex_start + 1].normal_y = normal_top.y;
    vertices[vertex_start + 1].normal_y = normal_top.z;
    vertices[vertex_start + 2].normal_x = normal_right.x;
    vertices[vertex_start + 2].normal_y = normal_right.y;
    vertices[vertex_start + 2].normal_y = normal_right.z;
    vertices[vertex_start + 3].normal_x = normal_left.x;
    vertices[vertex_start + 3].normal_y = normal_left.y;
    vertices[vertex_start + 3].normal_y = normal_left.z;

    // append 4 center vertices
    // this could just be one center vertex but 4 are required to have accurate normals
    let center = vec3<f32>(0, 0, 0);
    let output = config.output_vertex_start + config.num_vertices;
    vertices[output] = Vertex(center.x, center.y, center.z, normal_bottom.x, normal_bottom.y, normal_bottom.z, 0, 0);
    vertices[output + 1] = Vertex(center.x, center.y, center.z, normal_top.x, normal_top.y, normal_top.z, 0, 0);
    vertices[output + 2] = Vertex(center.x, center.y, center.z, normal_right.x, normal_right.y, normal_right.z, 0, 0);
    vertices[output + 3] = Vertex(center.x, center.y, center.z, normal_left.x, normal_left.y, normal_left.z, 0, 0);

    debug[0] = u32(vertices[output + 2].position_x);
}
