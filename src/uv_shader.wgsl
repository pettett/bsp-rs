// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

fn integer_to_rgb(integer : ptr<function, i32>) -> vec3<f32>{
    var red = 		f32((*integer * 109 + 47) % 269) / 269.0;
    var green =  	f32((*integer * 83 + 251) % 127) / 127.0;
    var blue =  	f32((*integer * 251 + 83) % 293) / 293.0;
    return vec3<f32>(red, green, blue);
}
struct VertexInput {
	//@builtin(vertex_index) vert : i32,
    @location(0) position: vec3<f32>, 
    @location(1) tex_coords: vec2<f32>,  
    @location(2) alpha: f32,  
};

struct VertexOutput { 
    @builtin(position) clip_position: vec4<f32>,
	@location(0) tex_coords : vec2<f32>,
    //@location(1) env_coords: vec2<f32>,
    //@location(2) @interpolate(flat) color: vec3<i32>, 
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput; 
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
	//var v = f32(model.vert) * 123.0;
	out.tex_coords = model.tex_coords;
	//out.env_coords = model.env_coords;
	//out.color = model.color;
	return out;
}
// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	// if t.a < 0.1{
	// 	discard;
 
					
	return vec4<f32>(in.tex_coords, 1.0, 1.0);
}
