#import bevy_pbr::{
	mesh_functions::{
		get_model_matrix, mesh_position_local_to_clip
	},
	mesh_view_bindings::globals,
}

struct VertexInput {
	@builtin(instance_index) inst_ix: u32,
	@location(0) pos: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
	@builtin(position) pos: vec4<f32>,
	@location(2) uv: vec2<f32>,
}

@vertex
fn sky2d_vertex(vertex: VertexInput) -> VertexOutput {
	let model = get_model_matrix(vertex.inst_ix);
	let view_proj = globals.view.view_proj;
	let tf = view_proj * model;
	let forward = tf[2];

	var out: VertexOutput;
	out.pos = tf * vec4<f32>(vertex.pos, 1.0);
	out.uv = vec2<f32>(atan2(forward.x, forward.z), forward.y / forward.w);
	return out;
}

// Fragment ////////////////////////////////////////////////////////////////////

@group(1) @binding(200)
var texture: texture_2d<f32>;
@group(1) @binding(201)
var texture_sampler: sampler;

@group(1) @binding(202)
var<uniform> tiled_band_size: f32;

@fragment
fn sky2d_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
	let tiled_band_size: f32 = 1.0;
	let p = vec2<f32>(in.pos.x, in.pos.y);

	var uv = p / in.pos.w * vec2<f32>(1.0, -1.0);
	uv = vec2<f32>(uv.x - 4.0 * in.uv.x / 3.14159265358, uv.y + 1.0 + in.uv.y);

	if (uv.y < 0.0) {
		let rem = (-uv.y + tiled_band_size) % (tiled_band_size * 2.0);
		uv.y = abs(rem - tiled_band_size);
	} else if (uv.y >= 2.0) {
		let rem = (uv.y - 2.0 + tiled_band_size) % (tiled_band_size * 2.0);
		uv.y = abs(rem - tiled_band_size);
	} else if (uv.y >= 1.0) {
		uv.y = 1.0 - uv.y;
	}

	return textureSample(texture, texture_sampler, uv);
}
