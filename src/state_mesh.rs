use std::{sync::{Arc, Mutex}, cell::RefCell};

use gltf::mesh::util::ReadIndices;
use wgpu::util::DeviceExt;

use crate::state::{State, StateRenderer, StateInstance};

pub struct StateMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_format: wgpu::IndexFormat,
    num_indices: u32,
    //puffin_ui : puffin_imgui::ProfilerUi,
}
impl State for StateMesh {
    fn render_pass(
        &mut self,
        state: &StateRenderer,
        encoder: &mut wgpu::CommandEncoder,
        output: &wgpu::SurfaceTexture,
        view: &wgpu::TextureView,
    ) {
        // 1.
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[
                // This is what @location(0) in the fragment shader targets
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }),
            ],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(state.render_pipeline());

        render_pass.set_bind_group(0, state.camera_bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), self.index_format);

        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }

    fn init(renderer: &StateRenderer) -> Self {

        let vertex_buffer =
            renderer
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: &[],
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            renderer
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: &[],
                    usage: wgpu::BufferUsages::INDEX,
                });

        StateMesh {
            vertex_buffer,
            index_buffer,
            num_indices: 0,
            index_format : wgpu::IndexFormat::Uint16,
        }
    }
}

impl StateMesh {
    pub fn load_mesh(into: Arc<Mutex<StateMesh>>, instance: Arc<StateInstance>) {
        let (document, buffers, images) =
            gltf::import("assets/dragon_high.glb").expect("Torus import should work");

        let mesh = document.meshes().next().unwrap();
        let prim = mesh.primitives().next().unwrap();

        let reader = prim.reader(|buffer| Some(&buffers[buffer.index()]));

        let iter = reader.read_positions().unwrap();
        let verts: Vec<[f32; 3]> = iter.collect();

        let vertex_buffer =
            instance
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&verts[..]),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let (index_buffer, index_format, num_indices) = match reader.read_indices() {
            Some(ReadIndices::U16(iter)) => {
                let indicies: Vec<u16> = iter.collect();

                (
                    instance
                        .device()
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: bytemuck::cast_slice(&indicies[..]),
                            usage: wgpu::BufferUsages::INDEX,
                        }),
                    wgpu::IndexFormat::Uint16,
                    indicies.len(),
                )
            }
            Some(ReadIndices::U32(iter)) => {
                let indicies: Vec<u32> = iter.collect();

                (
                    instance
                        .device()
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: bytemuck::cast_slice(&indicies[..]),
                            usage: wgpu::BufferUsages::INDEX,
                        }),
                    wgpu::IndexFormat::Uint32,
                    indicies.len(),
                )
            }
            _ => panic!("No indices"),
        };
		// Update the value stored in this mesh
		let mut into = into.lock().unwrap();
		*into = StateMesh {
            vertex_buffer,
            index_buffer,
            num_indices: num_indices as u32,
            index_format,
        };

    }
}
