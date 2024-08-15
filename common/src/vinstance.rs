/// Data that will be read only for the course of the program, containing everything needed to create shaders and pipelines
pub struct StateInstance {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub format: wgpu::TextureFormat,

    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub lighting_bind_group_layout: wgpu::BindGroupLayout,
}
