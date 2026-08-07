#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0)
var data: texture_2d<u32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let size = textureDimensions(data);

    let pixel = vec2<i32>(
        i32(in.uv.x * f32(size.x)),
        i32(in.uv.y * f32(size.y)),
    );

    let value = textureLoad(data, pixel, 0).r;
    var red = 0.;
    if value == 0 { red = 1; } else { red = 0; }

    var blue = 0.;
    if value == 1 { blue = 1; } else { blue = 0; }

    return vec4<f32>(
        red,
        1,
        blue,
        1.0,
    );
}