// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
	@builtin(vertex_index) vert : u32,
    @location(0) position: vec3<f32>, 
};


struct VertexOutput { 
    @builtin(position) clip_position: vec4<f32>,
	@location(0) color : vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput; 
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
	var v = f32(model.vert) * 123.0;
	out.color = vec3<f32>(v % 1.1, (v*17.0) % 1.1, (v*7.0) % 1.1);
	return out;
}
// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
