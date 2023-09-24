use crate::{
    state::StateInstance,
    vertex::{UVVertex, Vertex},
};

use super::vrenderer::VRenderer;

pub struct VShader {
    render_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    tex_bind_start: u32,
}

impl VShader {
    pub fn texture_bind_group_layout(&self, i: u32) -> Option<&wgpu::BindGroupLayout> {
        self.texture_bind_group_layouts.get(i as usize)
    }

    pub fn draw<'a>(&'a self, _state: &'a VRenderer, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
    }
    pub fn new_textured(renderer: &StateInstance) -> Self {
        let shader = renderer
            .device
            .create_shader_module(wgpu::include_wgsl!("../textured_shader.wgsl"));
        Self::new::<UVVertex>(
            renderer,
            shader,
            1,
            wgpu::PrimitiveTopology::TriangleList,
            Some(wgpu::Face::Back),
            "Textured",
        )
    }
    pub fn new_textured_envmap(renderer: &StateInstance) -> Self {
        let shader = renderer
            .device
            .create_shader_module(wgpu::include_wgsl!("../textured_shader_envmap.wgsl"));
        Self::new::<UVVertex>(
            renderer,
            shader,
            2,
            wgpu::PrimitiveTopology::TriangleList,
            Some(wgpu::Face::Back),
            "Textured Envmap",
        )
    }
    pub fn new_displacement(renderer: &StateInstance) -> Self {
        let shader = renderer
            .device
            .create_shader_module(wgpu::include_wgsl!("../displacement.wgsl"));
        Self::new::<UVVertex>(
            renderer,
            shader,
            2,
            wgpu::PrimitiveTopology::TriangleList,
            Some(wgpu::Face::Back),
            "Displacement",
        )
    }
    pub fn new_white_lines<V: Vertex>(renderer: &StateInstance) -> Self {
        let shader = renderer
            .device
            .create_shader_module(wgpu::include_wgsl!("../solid_white.wgsl"));
        Self::new::<V>(
            renderer,
            shader,
            0,
            wgpu::PrimitiveTopology::LineList,
            Some(wgpu::Face::Back),
            "Line list",
        )
    }

    pub fn new_instanced_prop<V: Vertex, I: Vertex>(renderer: &StateInstance) -> Self {
        let shader = renderer
            .device
            .create_shader_module(wgpu::include_wgsl!("../uv_shader.wgsl"));
        Self::new_instanced::<V, I>(
            renderer,
            shader,
            1,
            wgpu::PrimitiveTopology::TriangleList,
            Some(wgpu::Face::Front),
            "Triangle Strip",
        )
    }

    pub fn new<V: Vertex>(
        renderer: &StateInstance,
        shader: wgpu::ShaderModule,
        textures: usize,
        topology: wgpu::PrimitiveTopology,
        cull_mode: Option<wgpu::Face>,
        name: &str,
    ) -> Self {
        let mut texture_bind_group_layouts = Vec::new();

        for i in 0..textures {
            texture_bind_group_layouts.push(renderer.device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
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
                    label: Some(&format!("{} texture {}", name, i)),
                },
            ));
        }

        let mut bind_group_layouts = Vec::new();
        bind_group_layouts.push(&renderer.camera_bind_group_layout);

        bind_group_layouts.push(&renderer.lighting_bind_group_layout);

        for t in &texture_bind_group_layouts {
            bind_group_layouts.push(t)
        }

        let render_pipeline_layout =
            renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(name),
                    bind_group_layouts: &bind_group_layouts[..],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            renderer
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(name),
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
                            format: renderer.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology, // 1.
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw, // 2.
                        cull_mode,
                        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                        polygon_mode: wgpu::PolygonMode::Fill,
                        // Requires Features::DEPTH_CLIP_CONTROL
                        unclipped_depth: false,
                        // Requires Features::CONSERVATIVE_RASTERIZATION
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: crate::v::VTexture::DEPTH_FORMAT,
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
            texture_bind_group_layouts,
            tex_bind_start: 2,
        }
    }

    pub fn new_instanced<V: Vertex, I: Vertex>(
        renderer: &StateInstance,
        shader: wgpu::ShaderModule,
        textures: usize,
        topology: wgpu::PrimitiveTopology,
        cull_mode: Option<wgpu::Face>,
        name: &str,
    ) -> Self {
        let mut texture_bind_group_layouts = Vec::new();

        for i in 0..textures {
            texture_bind_group_layouts.push(renderer.device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
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
                    label: Some(&format!("{} texture {}", name, i)),
                },
            ));
        }

        let mut bind_group_layouts = Vec::new();
        bind_group_layouts.push(&renderer.camera_bind_group_layout);

        for t in &texture_bind_group_layouts {
            bind_group_layouts.push(t)
        }

        let render_pipeline_layout =
            renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(name),
                    bind_group_layouts: &bind_group_layouts[..],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            renderer
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(name),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main", // 1.
                        buffers: &[<V>::desc(), <I>::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        // 3.
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            // 4.
                            format: renderer.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology, // 1.
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw, // 2.
                        cull_mode,
                        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                        polygon_mode: wgpu::PolygonMode::Fill,
                        // Requires Features::DEPTH_CLIP_CONTROL
                        unclipped_depth: false,
                        // Requires Features::CONSERVATIVE_RASTERIZATION
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: crate::v::VTexture::DEPTH_FORMAT,
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
            texture_bind_group_layouts,
            tex_bind_start: 1,
        }
    }

    pub fn tex_bind_start(&self) -> u32 {
        self.tex_bind_start
    }
}
