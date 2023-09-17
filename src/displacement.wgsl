// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<storage, read> lighting: array<vec3<f32>>;

@group(2) @binding(0)
var t_diffuse0: texture_2d<f32>;
@group(2)@binding(1)
var s_diffuse0: sampler;

@group(3) @binding(0)
var t_diffuse1: texture_2d<f32>;
@group(3)@binding(1)
var s_diffuse1: sampler;

struct VertexInput {
	//@builtin(vertex_index) vert : i32,
    @location(0) position: vec3<f32>, 
    @location(1) tex_coords: vec2<f32>, 
    @location(2) env_coords: vec2<f32>, 
    @location(3) alpha: f32, 
    @location(4) @interpolate(flat) color: vec3<i32>, 
};
 

struct VertexOutput { 
    @builtin(position) clip_position: vec4<f32>,
	@location(0) tex_coords : vec2<f32>,
    @location(1) alpha: f32, 
    @location(2) env_coords: vec2<f32>, 
    @location(3) @interpolate(flat) color: vec3<i32>, 
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
	out.color = model.color;
	out.env_coords = model.env_coords;
	return out;
}
// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	var alpha = in.alpha / 255.0;

	var width = i32(in.color.y);
	var first_light = i32(in.color.x);

	var bottom_coord_x = floor(in.env_coords.y);
	var bottom_coord_y = floor(in.env_coords.x);
	var top_coord_x = ceil(in.env_coords.y);
	var top_coord_y = ceil(in.env_coords.x);

	var fracts = 1.0 - fract(in.env_coords);

	var top_1 = first_light + i32(bottom_coord_x) * width + i32(bottom_coord_y);
	var top_0 = first_light + i32(top_coord_x) * width + i32(bottom_coord_y);

	var bottom_1 = first_light + i32(bottom_coord_x) * width + i32(top_coord_y);
	var bottom_0 = first_light + i32(top_coord_x) * width + i32(top_coord_y);

	var first_col = mix(lighting[top_0], lighting[top_1], fracts.y);
	var second_col = mix(lighting[bottom_0], lighting[bottom_1], fracts.y);

	var col = mix(second_col, first_col, fracts.x);


    var tex_blend = mix(textureSample(t_diffuse0, s_diffuse0, in.tex_coords), textureSample(t_diffuse1, s_diffuse1, in.tex_coords) , alpha);

	return vec4<f32>(col.rgb * tex_blend.rgb, 1.0);

}
