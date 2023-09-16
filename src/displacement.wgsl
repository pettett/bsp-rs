// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_diffuse0: texture_2d<f32>;
@group(1)@binding(1)
var s_diffuse0: sampler;

@group(2) @binding(0)
var t_diffuse1: texture_2d<f32>;
@group(2)@binding(1)
var s_diffuse1: sampler;

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
    @location(1) alpha: f32, 
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput; 
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
	//var v = f32(model.vert) * 123.0;
	out.tex_coords = model.tex_coords;
	out.alpha = model.alpha;
	return out;
}
// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	var alpha = in.alpha / 255.0;
    return  textureSample(t_diffuse0, s_diffuse0, in.tex_coords) * (1.0-alpha) + 
			textureSample(t_diffuse1, s_diffuse1, in.tex_coords) * alpha;
}
