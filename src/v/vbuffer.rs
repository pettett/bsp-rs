use std::mem;

use wgpu::util::DeviceExt;

pub struct VBuffer {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl VBuffer {
    pub fn new<T: bytemuck::Pod>(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        value: T,
        label: &'static str,
    ) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: &bytemuck::cast_slice(&[value]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some(label),
        });

        Self { buffer, bind_group }
    }

    pub fn new_zeroed<T: bytemuck::Pod>(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        label: &'static str,
    ) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: &bytemuck::zeroed_slice_box(mem::size_of::<T>()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some(label),
        });

        Self { buffer, bind_group }
    }
}
