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
        value: &[T],
        usage: wgpu::BufferUsages,
        label: &'static str,
    ) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: &bytemuck::cast_slice(value),
            usage,
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
        usage: wgpu::BufferUsages,
        label: &'static str,
    ) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: &bytemuck::zeroed_slice_box(mem::size_of::<T>()),
            usage,
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
