// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1)@binding(1)
var s_diffuse: sampler;

@group(2) @binding(0)
var t_envmap: texture_2d<f32>;
@group(2)@binding(1)
var s_envmap: sampler;

struct VertexInput {
	//@builtin(vertex_index) vert : u32,
    @location(0) position: vec3<f32>, 
    @location(1) tex_coords: vec2<f32>, 
    @location(2) env_coords: vec2<f32>, 
    @location(3) alpha: f32, 
};


struct VertexOutput { 
    @builtin(position) clip_position: vec4<f32>,
	@location(0) tex_coords : vec2<f32>,
	@location(1) env_coords : vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput; 
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
	//var v = f32(model.vert) * 123.0;
	out.tex_coords = model.tex_coords;
	out.env_coords = model.env_coords;
	return out;
}
// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var t = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    var e = textureSample(t_envmap, s_envmap, in.env_coords);
	// if t.a < 0.1{
	// 	discard;
	// }
	return t + e;
}
