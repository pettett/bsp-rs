use std::{path::PathBuf, sync::Arc};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, NonSendMut, Query, Res, Resource},
    world::World,
};
use wgpu::util::DeviceExt;

use crate::{
    bsp::lightmap::LightingData,
    camera::{Camera, CameraUniform},
    camera_controller::CameraController,
    gui::state_imgui::StateImgui,
    state::StateInstance,
    vpk::VPKDirectory,
};
const TEX_PATH: &str =
    "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_textures_dir.vpk";

const MISC_PATH: &str =
    "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_misc_dir.vpk";

use super::{VMesh, VTexture};

#[derive(Resource)]
pub struct VRenderer {
    instance: Arc<StateInstance>,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    window: Arc<winit::window::Window>,
    pub depth_texture: VTexture,
    camera_entity: Entity,
    camera_buffer: wgpu::Buffer,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,

    texture_dir: Arc<VPKDirectory>,
    misc_dir: Arc<VPKDirectory>,
}

pub fn draw(
    meshes: Query<(&VMesh,)>,
    cameras: Query<(&CameraUniform,)>,
    mut imgui: NonSendMut<StateImgui>,
    renderer: Res<VRenderer>,
    lighting: Res<LightingData>,
    mut commands: Commands,
) {
    let output = renderer.instance.surface().get_current_texture().unwrap();

    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = renderer
        .device()
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    //renderer
    //    .camera_controller
    //    .update_camera(&mut renderer.camera);

    //renderer.camera_uniform.update_view_proj(&renderer.camera);

    renderer.queue().write_buffer(
        &renderer.camera_buffer,
        0,
        bytemuck::cast_slice(&[*cameras.single().0]),
    );

    {
        //let meshes = self.meshes.lock().unwrap();
        let mut render_pass: wgpu::RenderPass<'_> =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &renderer.depth_texture.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

        for (mesh,) in meshes.iter() {
            mesh.draw(&renderer, &mut render_pass, &lighting);
        }
    }

    imgui.render_pass(&renderer, &mut encoder, &output, &view, &mut commands);

    // submit will accept anything that implements IntoIter
    renderer.queue().submit(std::iter::once(encoder.finish()));
    output.present();
}

impl VRenderer {
    /// Creating some of the wgpu types requires async code
    /// https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#state-new
    pub async fn new(window: winit::window::Window, world: &mut World) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::TEXTURE_COMPRESSION_BC,
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let camera = Camera::new(config.width as f32 / config.height as f32);

        let mut camera_uniform = CameraUniform::default();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout: wgpu::BindGroupLayout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let depth_texture = VTexture::create_depth_texture(&device, &config, "depth_texture");

        let texture_dir = Arc::new(VPKDirectory::load(PathBuf::from(TEX_PATH)).unwrap());
        let misc_dir = Arc::new(VPKDirectory::load(PathBuf::from(MISC_PATH)).unwrap());

        puffin::set_scopes_on(true); // you may want to control this with a flag
                                     //let  puffin_ui = puffin_imgui::ProfilerUi::default();

        let camera_entity = world
            .spawn((camera, CameraController::new(10.), camera_uniform))
            .id();

        Self {
            window: Arc::new(window),
            instance: Arc::new(StateInstance::new(surface, device, queue)),
            camera_bind_group_layout,
            config,
            size,
            camera_entity,
            depth_texture,
            camera_buffer,
            camera_bind_group,
            texture_dir,
            misc_dir,
        }
    }

    pub fn window(&self) -> &Arc<winit::window::Window> {
        &self.window
    }
    pub fn surface(&self) -> &wgpu::Surface {
        &self.instance.surface()
    }
    pub fn device(&self) -> &wgpu::Device {
        &self.instance.device()
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.instance.queue()
    }
    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }
    pub fn camera_bind_group(&self) -> &wgpu::BindGroup {
        &self.camera_bind_group
    }

    pub fn texture_dir(&self) -> &Arc<VPKDirectory> {
        &self.texture_dir
    }
    pub fn misc_dir(&self) -> &Arc<VPKDirectory> {
        &self.misc_dir
    }
    pub fn camera_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.camera_bind_group_layout
    }

    pub fn instance(&self) -> Arc<StateInstance> {
        self.instance.clone()
    }
    pub fn camera(&self) -> &Entity {
        &self.camera_entity
    }
}