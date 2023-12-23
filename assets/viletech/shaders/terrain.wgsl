#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip}

// Vertex //////////////////////////////////////////////////////////////////////

struct Vertex {
	@builtin(instance_index) inst_ix: u32,
    @location(0) position: vec3<f32>,
	@location(1) normal: vec3<f32>,
	@location(2) uv: vec2<f32>,
	@location(3) tex_ix: u32,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
	@location(1) tex_ix: u32,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    out.clip_pos = mesh_position_local_to_clip(
        get_model_matrix(vertex.inst_ix),
        vec4<f32>(vertex.position, 1.0),
    );

	out.uv = vertex.uv;
	out.tex_ix = vertex.tex_ix;
    return out;
}

// Fragment ////////////////////////////////////////////////////////////////////

@group(1) @binding(0)
var textures: binding_array<texture_2d<f32>>;
@group(1) @binding(1)
var nearest_sampler: sampler;

struct FragmentInput {
	@location(0) uv: vec2<f32>,
	@location(1) tex_ix: u32,
}

@fragment
fn fragment(input: FragmentInput) -> @location(0) vec4<f32> {
    // Select the texture to sample from using non-uniform UV coordinates.
    let coords = clamp(vec2<u32>(input.uv), vec2<u32>(0u), vec2<u32>(3u));
    let inner_uv = fract(input.uv);
    return textureSample(textures[input.tex_ix], nearest_sampler, inner_uv);
}
