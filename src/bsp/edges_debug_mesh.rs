use std::sync::{Arc, Mutex};

use glam::Vec3;
use wgpu::util::DeviceExt;

use crate::state::{State, StateInstance, StateRenderer};

use super::{consts::LumpType, edges::dedge_t, header::dheader_t};

pub struct EdgesDebugMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_format: wgpu::IndexFormat,
    num_indices: u32,
    //puffin_ui : puffin_imgui::ProfilerUi,
}
impl State for EdgesDebugMesh {
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

        EdgesDebugMesh {
            vertex_buffer,
            index_buffer,
            num_indices: 0,
            index_format: wgpu::IndexFormat::Uint16,
        }
    }
}

impl EdgesDebugMesh {
    pub fn load_mesh(into: Arc<Mutex<EdgesDebugMesh>>, instance: Arc<StateInstance>) {
        let (header,mut buffer) = dheader_t::load(
			"D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\maps\\d1_trainstation_01.bsp").unwrap();

        header.validate();

        // let planes: Vec<dplane_t> = header.get_lump(LumpType::PLANES).decode(&mut buffer);
        // Lump::validate(&planes);

        // let faces: Vec<dface_t> = header.get_lump(LumpType::FACES).decode(&mut buffer);
        // Lump::validate(&faces);

        let edges = header.get_lump::<dedge_t>(&mut buffer);
        let verts = header.get_lump::<Vec3>(&mut buffer);

        let vertex_buffer =
            instance
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&verts[..]),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            instance
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&edges[..]),
                    usage: wgpu::BufferUsages::INDEX,
                });

        // Update the value stored in this mesh
        let mut into = into.lock().unwrap();
        *into = EdgesDebugMesh {
            vertex_buffer,
            index_buffer,
            num_indices: edges.len() as u32,
            index_format: wgpu::IndexFormat::Uint16,
        };
    }
}
