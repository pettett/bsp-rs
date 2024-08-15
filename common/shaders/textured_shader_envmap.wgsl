// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<storage, read> lighting: array<vec4<f32>>;

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2)@binding(1)
var s_diffuse: sampler;

@group(3) @binding(0)
var t_envmap: texture_2d<f32>;
@group(3)@binding(1)
var s_envmap: sampler;

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
	@location(1) env_coords : vec2<f32>,
	@location(2) @interpolate(flat) color : vec3<i32>,
};


fn integer_to_rgb(integer : ptr<function, i32>) -> vec3<f32>{
    var red = 		f32((*integer * 109 + 47) % 269) / 269.0;
    var green =  	f32((*integer * 83 + 251) % 127) / 127.0;
    var blue =  	f32((*integer * 251 + 83) % 293) / 293.0;
    return vec3<f32>(red, green, blue);
}


@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput; 
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
	//var v = f32(model.vert) * 123.0;
	out.tex_coords = model.tex_coords;
	out.env_coords = model.env_coords;
	out.color = model.color;
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

	var width = i32(in.color.y);
	var first_light = i32(in.color.x);

	var bottom_coord_x = floor(in.env_coords.y);
	var bottom_coord_y = floor(in.env_coords.x);
	var top_coord_x = ceil(in.env_coords.y);
	var top_coord_y = ceil(in.env_coords.x);

	var fracts = fract(in.env_coords);

	var top_1 = first_light + i32(bottom_coord_x) * width + i32(bottom_coord_y);
	var top_0 = first_light + i32(top_coord_x) * width + i32(bottom_coord_y);

	var bottom_1 = first_light + i32(bottom_coord_x) * width + i32(top_coord_y);
	var bottom_0 = first_light + i32(top_coord_x) * width + i32(top_coord_y);

	var first_col = mix(lighting[top_0], lighting[top_1], fracts.y);
	var second_col = mix(lighting[bottom_0], lighting[bottom_1], fracts.y);

	var col = mix(second_col, first_col, fracts.x);

	return vec4<f32>(col.rgb, 1.0);
}
