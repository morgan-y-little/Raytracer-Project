
struct VertexInput {
    @location(0) position: vec2<f32>, 
    @location(1) tex_coords: vec2<f32>, 
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>, 
    @location(0) uv: vec2<f32>,                  
};

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(vertex.position, 1.0, 1.0);
    out.uv = vertex.tex_coords;
    return out;
}

@group(0) @binding(0) var<storage, read> color_buffer: array<vec4<f32>>;

@fragment
fn fs_main(@location(0) in_uv: vec2<f32>) -> @location(0) vec4<f32> {

    let width = u32(2400);
    let height = u32(1600);
    
    let x = u32(in_uv.x * f32(width));
    let y = u32(in_uv.y * f32(height));
    
    let index = y * width + x;

    return color_buffer[index];
}
