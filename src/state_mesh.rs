use crate::{
    bsp::{edges::BSPEdge, header::BSPHeader},
    shader::Shader,
    texture,
    vertex::{UVVertex, Vertex},
};
use std::{collections::HashMap, fs::File, io::BufReader, sync::Arc};

use bevy_ecs::component::Component;
use glam::{vec3, Vec3};

use wgpu::util::DeviceExt;

use crate::state::{StateInstance, StateRenderer};
#[derive(Component)]
pub struct StateMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_format: wgpu::IndexFormat,
    texture_bind_group: HashMap<u32, wgpu::BindGroup>,
    shader: Arc<Shader>,
    num_indices: u32,
    //puffin_ui : puffin_imgui::ProfilerUi,
}

impl StateMesh {
    pub fn draw<'a>(
        &'a self,
        state: &'a StateRenderer,
        render_pass: &mut wgpu::RenderPass<'a>,
        _output: &wgpu::SurfaceTexture,
        _view: &wgpu::TextureView,
    ) {
        // 1.

        self.shader.draw(state, render_pass, _output, _view);

        render_pass.set_bind_group(0, state.camera_bind_group(), &[]);

        for (i, tex) in &self.texture_bind_group {
            render_pass.set_bind_group(*i, tex, &[]);
        }

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), self.index_format);

        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }

    pub fn new_box(renderer: &StateRenderer, min: Vec3, max: Vec3, shader: Arc<Shader>) -> Self {
        Self::new(
            renderer,
            &[
                vec3(min.x, min.y, min.z),
                vec3(max.x, min.y, min.z),
                vec3(min.x, max.y, min.z),
                vec3(max.x, max.y, min.z),
                vec3(min.x, min.y, max.z),
                vec3(max.x, min.y, max.z),
                vec3(min.x, max.y, max.z),
                vec3(max.x, max.y, max.z),
            ],
            &[
                0u16, 1, 1, 3, 3, 2, 2, 0, 4, 5, 5, 7, 7, 6, 6, 4, 0, 4, 1, 5, 2, 6, 3, 7,
            ],
            shader,
        )
    }
    pub fn new_empty(renderer: &StateRenderer, shader: Arc<Shader>) -> Self {
        Self::new::<UVVertex>(renderer, &[], &[], shader)
    }
    pub fn new<V: Vertex + bytemuck::Pod>(
        renderer: &StateRenderer,
        verts_data: &[V],
        indices_data: &[u16],
        shader: Arc<Shader>,
    ) -> Self {
        let vertex_buffer =
            renderer
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(verts_data),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            renderer
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(indices_data),
                    usage: wgpu::BufferUsages::INDEX,
                });

        StateMesh {
            vertex_buffer,
            index_buffer,
            num_indices: indices_data.len() as u32,
            texture_bind_group: Default::default(),
            shader,
            index_format: wgpu::IndexFormat::Uint16,
        }
    }

    pub fn load_debug_edges(
        &mut self,
        instance: Arc<StateInstance>,
        header: &BSPHeader,
        buffer: &mut BufReader<File>,
    ) {
        let edges = header.get_lump::<BSPEdge>(buffer);
        let verts = header.get_lump::<Vec3>(buffer);

        let mut annotated_verts = bytemuck::zeroed_slice_box::<UVVertex>(verts.len());

        for i in 0..verts.len() {
            annotated_verts[i].position = verts[i];
        }

        self.vertex_buffer =
            instance
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&annotated_verts[..]),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        self.index_buffer =
            instance
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&edges[..]),
                    usage: wgpu::BufferUsages::INDEX,
                });

        // Update the value stored in this mesh

        self.num_indices = edges.len() as u32;
        self.index_format = wgpu::IndexFormat::Uint16;
    }

    pub fn from_verts_and_tris(
        &mut self,
        instance: Arc<StateInstance>,
        verts: &[u8],
        tris: &[u8],
        num_indicies: u32,
    ) {
        self.vertex_buffer =
            instance
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: verts,
                    usage: wgpu::BufferUsages::VERTEX,
                });

        self.index_buffer =
            instance
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: tris,
                    usage: wgpu::BufferUsages::INDEX,
                });

        // Update the value stored in this mesh

        self.num_indices = num_indicies;
        self.index_format = wgpu::IndexFormat::Uint16;
    }

    pub fn load_tex(
        &mut self,
        instance: Arc<StateInstance>,
        bind_index: u32,
        texture: &texture::Texture,
    ) {
        self.texture_bind_group.insert(
            bind_index,
            instance
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.shader.texture_bind_group_layout(bind_index).unwrap(),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(texture.view()),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(texture.sampler()),
                        },
                    ],
                    label: Some("diffuse_bind_group"),
                }),
        );
    }
}
