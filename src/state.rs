
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::camera::{Camera, CameraUniform};
use crate::camera_controller::CameraController;
use crate::state_imgui::StateImgui;
use crate::state_mesh::StateMesh;
use crate::texture::{self, Texture};

use crate::vpk::VPKDirectory;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::event::*;
use winit::window::Window;
const PATH: &str =
    "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_textures_dir.vpk";

pub trait State {
    fn render_pass(
        &mut self,
        renderer: &StateRenderer,
        encoder: &mut wgpu::CommandEncoder,
        output: &wgpu::SurfaceTexture,
        view: &wgpu::TextureView,
    );

    fn init(renderer: &StateRenderer) -> Self;
}

pub struct StateApp {
    meshes: Arc<Mutex<Vec<StateMesh>>>,
    imgui: StateImgui,
    renderer: StateRenderer,
    //puffin_ui : puffin_imgui::ProfilerUi,
}

/// Data that will be read only for the course of the program
pub struct StateInstance {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

pub struct StateRenderer {
    instance: Arc<StateInstance>,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    depth_texture: Texture,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_controller: CameraController,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
    texture_dir: VPKDirectory,
}
impl StateInstance {
    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

impl StateApp {
    /// Creating some of the wgpu types requires async code
    /// https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#state-new
    pub async fn new(renderer: StateRenderer) -> Self {
        Self {
            imgui: StateImgui::init(&renderer),
            meshes: Arc::new(Mutex::new(Vec::new())),
            renderer,
            //puffin_ui
        }
    }

    pub fn add_mesh(&self, mesh: StateMesh) {
        self.meshes.lock().unwrap().push(mesh)
    }

    pub fn renderer(&self) -> &StateRenderer {
        &self.renderer
    }
    // impl State
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.renderer.size = new_size;
            self.renderer.config.width = new_size.width;
            self.renderer.config.height = new_size.height;
            self.renderer.depth_texture = texture::Texture::create_depth_texture(
                self.renderer.device(),
                self.renderer.config(),
                "depth_texture",
            );
            self.renderer
                .instance
                .surface
                .configure(&self.renderer.instance.device, &self.renderer.config);
        }
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.renderer.size
    }

    pub fn handle_event<T>(&mut self, event: &Event<T>) {
        self.imgui.handle_event(&self.renderer, event);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.renderer.camera_controller.process_events(event)
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        puffin::profile_function!();

        let output = self.renderer.instance.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.renderer
                .instance
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        self.renderer
            .camera_controller
            .update_camera(&mut self.renderer.camera);

        self.renderer
            .camera_uniform
            .update_view_proj(&self.renderer.camera);
        self.renderer.instance.queue.write_buffer(
            &self.renderer.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.renderer.camera_uniform]),
        );

        {
            let meshes = self.meshes.lock().unwrap();
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
                        view: &self.renderer.depth_texture.view(),
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                });
            for mesh in meshes.iter() {
                mesh.draw(&self.renderer, &mut render_pass, &output, &view);
            }
        }
        // self.debug_mesh
        //     .lock()
        //     .unwrap()
        //     .render_pass(&self.renderer, &mut encoder, &output, &view);
        // self.faces_debug_mesh.lock().unwrap().render_pass(
        //     &self.renderer,
        //     &mut encoder,
        //     &output,
        //     &view,
        // );

        self.imgui
            .render_pass(&self.renderer, &mut encoder, &output, &view);

        // submit will accept anything that implements IntoIter
        self.renderer
            .instance
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

impl StateRenderer {
    /// Creating some of the wgpu types requires async code
    /// https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#state-new
    pub async fn new(window: Window) -> Self {
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

        let mut camera_uniform = CameraUniform::new();
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

        let camera_controller = CameraController::new(10.0);

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let texture_dir = VPKDirectory::load(PathBuf::from(PATH)).unwrap();

        puffin::set_scopes_on(true); // you may want to control this with a flag
                                     //let  puffin_ui = puffin_imgui::ProfilerUi::default();

        Self {
            window,
            instance: Arc::new(StateInstance {
                surface,
                device,
                queue,
            }),
            camera_bind_group_layout,
            config,
            size,
            camera,
            depth_texture,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            texture_dir,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
    pub fn surface(&self) -> &wgpu::Surface {
        &self.instance.surface
    }
    pub fn device(&self) -> &wgpu::Device {
        &self.instance.device
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.instance.queue
    }
    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }
    pub fn camera_bind_group(&self) -> &wgpu::BindGroup {
        &self.camera_bind_group
    }
    pub fn texture_dir(&self) -> &VPKDirectory {
        &self.texture_dir
    }
    pub fn camera_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.camera_bind_group_layout
    }

    pub fn instance(&self) -> Arc<StateInstance> {
        self.instance.clone()
    }
    pub fn camera(&self) -> &Camera {
        &self.camera
    }
}
