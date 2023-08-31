use crate::{
    state::StateRenderer,
    vertex::{UVAlphaVertex, UVVertex, Vertex},
};

pub struct Shader {
    render_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl Shader {
    pub fn draw<'a>(
        &'a self,
        _state: &'a StateRenderer,
        render_pass: &mut wgpu::RenderPass<'a>,
        _output: &wgpu::SurfaceTexture,
        _view: &wgpu::TextureView,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
    }
    pub fn new_textured(renderer: &StateRenderer) -> Self {
        let shader = renderer
            .device()
            .create_shader_module(wgpu::include_wgsl!("textured_shader.wgsl"));
        Self::new::<UVVertex>(renderer, shader, wgpu::PrimitiveTopology::TriangleList)
    }
    pub fn new_displacement(renderer: &StateRenderer) -> Self {
        let shader = renderer
            .device()
            .create_shader_module(wgpu::include_wgsl!("displacement.wgsl"));
        Self::new::<UVAlphaVertex>(renderer, shader, wgpu::PrimitiveTopology::TriangleList)
    }
    pub fn new_white_lines<V: Vertex + bytemuck::Pod>(renderer: &StateRenderer) -> Self {
        let shader = renderer
            .device()
            .create_shader_module(wgpu::include_wgsl!("solid_white.wgsl"));
        Self::new::<V>(renderer, shader, wgpu::PrimitiveTopology::LineList)
    }

    pub fn new<V: Vertex + bytemuck::Pod>(
        renderer: &StateRenderer,
        shader: wgpu::ShaderModule,
        topology: wgpu::PrimitiveTopology,
    ) -> Self {
        let texture_bind_group_layout =
            renderer
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            // This should match the filterable field of the
                            // corresponding Texture entry above.
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });

        let render_pipeline_layout =
            renderer
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &renderer.camera_bind_group_layout(),
                        &texture_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            renderer
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main", // 1.
                        buffers: &[<V>::desc()],
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
        Self {
            render_pipeline,
            texture_bind_group_layout,
        }
    }
}
