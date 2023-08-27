use crate::{
    bsp::{
        consts::LumpType,
        edges::{dedge_t, dsurfedge_t},
        face::dface_t,
        header::dheader_t,
    },
    vertex::Vertex,
};
use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, Mutex},
};

use glam::Vec3;
use gltf::mesh::util::ReadIndices;
use stream_unzip::ZipReader;
use wgpu::util::DeviceExt;

use crate::state::{State, StateInstance, StateRenderer};

pub struct StateMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_format: wgpu::IndexFormat,
    render_pipeline: wgpu::RenderPipeline,
    num_indices: u32,
    //puffin_ui : puffin_imgui::ProfilerUi,
}

impl StateMesh {
    pub fn draw<'a>(
        &'a self,
        state: &'a StateRenderer,
        render_pass: &mut wgpu::RenderPass<'a>,
        output: &wgpu::SurfaceTexture,
        view: &wgpu::TextureView,
    ) {
        // 1.

        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group(0, state.camera_bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), self.index_format);

        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }

    pub fn new(renderer: &StateRenderer, topology: wgpu::PrimitiveTopology) -> Self {
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
        let render_pipeline_layout =
            renderer
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&renderer.camera_bind_group_layout()],
                    push_constant_ranges: &[],
                });

        let shader = renderer
            .device()
            .create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline =
            renderer
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main", // 1.
                        buffers: &[<[f32; 3]>::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        // 3.
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            // 4.
                            format: renderer.config().format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology, // 1.
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw, // 2.
                        cull_mode: Some(wgpu::Face::Back),
                        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                        polygon_mode: wgpu::PolygonMode::Fill,
                        // Requires Features::DEPTH_CLIP_CONTROL
                        unclipped_depth: false,
                        // Requires Features::CONSERVATIVE_RASTERIZATION
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: crate::texture::Texture::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less, // 1.
                        stencil: wgpu::StencilState::default(),     // 2.
                        bias: wgpu::DepthBiasState::default(),
                    }), // 1.
                    multisample: wgpu::MultisampleState {
                        count: 1,                         // 2.
                        mask: !0,                         // 3.
                        alpha_to_coverage_enabled: false, // 4.
                    },
                    multiview: None, // 5.
                });

        StateMesh {
            vertex_buffer,
            index_buffer,
            num_indices: 0,
            render_pipeline,
            index_format: wgpu::IndexFormat::Uint16,
        }
    }

    pub fn load_glb_mesh(&mut self, instance: Arc<StateInstance>) {
        let (document, buffers, images) =
            gltf::import("assets/dragon_high.glb").expect("Torus import should work");

        let mesh = document.meshes().next().unwrap();
        let prim = mesh.primitives().next().unwrap();

        let reader = prim.reader(|buffer| Some(&buffers[buffer.index()]));

        let iter = reader.read_positions().unwrap();
        let verts: Vec<[f32; 3]> = iter.collect();

        self.vertex_buffer =
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
        self.index_buffer = index_buffer;

        self.num_indices = num_indices as u32;
        self.index_format = index_format;
    }

    pub fn load_debug_edges(
        &mut self,
        instance: Arc<StateInstance>,
        header: &dheader_t,
        buffer: &mut BufReader<File>,
    ) {
        let edges = header.get_lump::<dedge_t>(buffer);
        let verts = header.get_lump::<Vec3>(buffer);

        self.vertex_buffer =
            instance
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&verts[..]),
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

    pub fn load_debug_faces(
        &mut self,
        instance: Arc<StateInstance>,
        header: &dheader_t,
        buffer: &mut BufReader<File>,
    ) {
        let pakfile = header.get_lump_header(LumpType::PAKFILE);

        let pakfile_data = pakfile.get_bytes(buffer).unwrap();

        let entries = {
            let mut zip_reader = ZipReader::default();

            zip_reader.update(pakfile_data.into());

            // Or read the whole file and deal with the entries
            // at the end.
            zip_reader.finish();

            zip_reader.drain_entries()
        };

        println!("{:?}", entries[0]);

        let faces = header.get_lump::<dface_t>(buffer);
        let surfedges = header.get_lump::<dsurfedge_t>(buffer);
        let edges = header.get_lump::<dedge_t>(buffer);
        let verts = header.get_lump::<Vec3>(buffer);

        let mut tris = Vec::<u16>::new();

        for face in faces.iter() {
            let root_edge_index = face.first_edge as usize;
            let root_edge = surfedges[root_edge_index].get_edge(&edges);

            for i in 1..(face.num_edges as usize) {
                let edge = surfedges[root_edge_index + i].get_edge(&edges);

                tris.extend([edge.0, root_edge.0, edge.1])
            }
        }

        self.vertex_buffer =
            instance
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&verts[..]),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        self.index_buffer =
            instance
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&tris[..]),
                    usage: wgpu::BufferUsages::INDEX,
                });

        // Update the value stored in this mesh

        self.num_indices = tris.len() as u32;
        self.index_format = wgpu::IndexFormat::Uint16;
    }
}
