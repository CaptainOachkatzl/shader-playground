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

    // copy all input vertices to output
    for (var i = 0u; i < config.num_vertices; i++) {
        let input = config.input_vertex_start + i;
        let output = config.output_vertex_start + i;
        vertices[output] = vertices[input];
    }

    let appendix_start = config.num_vertices;

    // copy the first 4 front facing vertices to the appendix section to allow for normals fine tuning
    for (var i = 0u; i < 4; i++) {
        let input = config.input_vertex_start + i;
        let output = config.output_vertex_start + appendix_start + i;
        vertices[output] = vertices[input];
    }

    let vertex_idx_bottom_left_normals_left = 0u;
    let vertex_idx_bottom_left_normals_bottom = appendix_start;

    let vertex_idx_bottom_right_normals_right = 1u;
    let vertex_idx_bottom_right_normals_bottom = appendix_start + 1;

    let vertex_idx_top_right_normals_right = 2u;
    let vertex_idx_top_right_normals_top = appendix_start + 2;

    let vertex_idx_top_left_normals_left = 3u;
    let vertex_idx_top_left_normals_top = appendix_start + 3;

    let vertex_idx_center_normals_bottom = appendix_start + 4;
    let vertex_idx_center_normals_top = appendix_start + 5;
    let vertex_idx_center_normals_left = appendix_start + 6;
    let vertex_idx_center_normals_right = appendix_start + 7;

    // in place edit the first 2 triangles
    let index_start = config.output_index_start;
    // bottom
    indices[index_start] = vertex_idx_bottom_left_normals_bottom;
    indices[index_start + 1] = vertex_idx_bottom_right_normals_bottom;
    indices[index_start + 2] = vertex_idx_center_normals_bottom;
    // top
    indices[index_start + 3] = vertex_idx_top_right_normals_top;
    indices[index_start + 4] = vertex_idx_top_left_normals_top;
    indices[index_start + 5] = vertex_idx_center_normals_top;

    // append 2 triangles at the end
    let index_end = config.output_index_start + config.num_indices;
    // right
    indices[index_end] = vertex_idx_center_normals_right;
    indices[index_end + 1] = vertex_idx_bottom_right_normals_right;
    indices[index_end + 2] = vertex_idx_top_right_normals_right;
    // left
    indices[index_end + 3] = vertex_idx_top_left_normals_left;
    indices[index_end + 4] = vertex_idx_bottom_left_normals_left;
    indices[index_end + 5] = vertex_idx_center_normals_left;

    // define normals for each of the denting triangles
    let normals_top = normalize(vec3<f32>(0, -1, 1));
    let normals_bottom = normalize(vec3<f32>(0, 1, 1));
    let normals_left = normalize(vec3<f32>(1, 0, 1));
    let normals_right = normalize(vec3<f32>(-1, 0, 1));

    // adjust normals of front facing vertices since they are now bent inwards

    // bottom left
    apply_normals_to_vertex(vertex_idx_bottom_left_normals_bottom, normals_bottom);
    apply_normals_to_vertex(vertex_idx_bottom_left_normals_left, normals_left);

    // bottom right
    apply_normals_to_vertex(vertex_idx_bottom_right_normals_bottom, normals_bottom);
    apply_normals_to_vertex(vertex_idx_bottom_right_normals_right, normals_right);

    // top right
    apply_normals_to_vertex(vertex_idx_top_right_normals_top, normals_top);
    apply_normals_to_vertex(vertex_idx_top_right_normals_right, normals_right);

    // top left
    apply_normals_to_vertex(vertex_idx_top_left_normals_top, normals_top);
    apply_normals_to_vertex(vertex_idx_top_left_normals_left, normals_left);

    // append 4 dent-center vertices
    // this could just be one center vertex but 4 are required to have accurate normals
    let center = vec3<f32>(0, 0, 0);
    vertices[config.output_vertex_start + vertex_idx_center_normals_bottom] = Vertex(center.x, center.y, center.z, normals_bottom.x, normals_bottom.y, normals_bottom.z, 0, 0);
    vertices[config.output_vertex_start + vertex_idx_center_normals_top] = Vertex(center.x, center.y, center.z, normals_top.x, normals_top.y, normals_top.z, 0, 0);
    vertices[config.output_vertex_start + vertex_idx_center_normals_right] = Vertex(center.x, center.y, center.z, normals_right.x, normals_right.y, normals_right.z, 0, 0);
    vertices[config.output_vertex_start + vertex_idx_center_normals_left] = Vertex(center.x, center.y, center.z, normals_left.x, normals_left.y, normals_left.z, 0, 0);

    debug[0] = u32(config.num_vertices + 4);
}

fn apply_normals_to_vertex(vertex_idx: u32, normals: vec3<f32>) {
    vertices[config.output_vertex_start + vertex_idx].normal_x = normals.x;
    vertices[config.output_vertex_start + vertex_idx].normal_y = normals.y;
    vertices[config.output_vertex_start + vertex_idx].normal_z = normals.z;
}