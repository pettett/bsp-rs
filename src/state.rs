use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::bsp::pak::BSPPak;
use crate::camera::{update_view_proj, Camera, CameraUniform};
use crate::camera_controller::{
    on_key_in, on_mouse_in, on_mouse_mv, update_camera, CameraController, KeyIn, MouseIn, MouseMv,
};
use crate::gui::state_imgui::StateImgui;
use crate::state_mesh::StateMesh;
use crate::texture::{self, Texture};

use crate::vpk::VPKDirectory;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Events;
use bevy_ecs::schedule::Schedule;
use bevy_ecs::system::{Res, Resource};
use bevy_ecs::world::{Mut, World};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::event::*;
use winit::window::{Window, WindowId};
const TEX_PATH: &str =
    "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_textures_dir.vpk";

const MISC_PATH: &str =
    "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_misc_dir.vpk";

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
    world: World,
    schedule: Schedule,
}

/// Data that will be read only for the course of the program
pub struct StateInstance {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
}
#[derive(Resource)]
pub struct StateRenderer {
    instance: Arc<StateInstance>,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Arc<Window>,
    depth_texture: Texture,
    camera_entity: Entity,
    camera_buffer: wgpu::Buffer,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
    texture_dir: Arc<VPKDirectory>,
    misc_dir: Arc<VPKDirectory>,
    pub pak: Option<BSPPak>,
}

struct ImGuiMarker();

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
#[derive(bevy_ecs::event::Event)]
struct MyWindowEvent<'a> {
    window_id: WindowId,
    event: WindowEvent<'a>,
}

impl StateApp {
    /// Creating some of the wgpu types requires async code
    /// https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#state-new
    pub async fn new(mut world: World, renderer: StateRenderer) -> Self {
        world.insert_non_send_resource(StateImgui::init(&renderer));
        world.insert_resource(renderer);
        let mut schedule = Schedule::default();

        world.insert_resource(Events::<MouseIn>::default());
        world.insert_resource(Events::<MouseMv>::default());
        world.insert_resource(Events::<KeyIn>::default());

        // Add our system to the schedule
        schedule.add_systems((
            on_mouse_in,
            on_mouse_mv,
            on_key_in,
            update_camera,
            update_view_proj,
        ));
        Self {
            world,
            schedule, //puffin_ui
        }
    }
    pub fn world(&self) -> &World {
        &self.world
    }
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
    pub fn renderer(&self) -> &StateRenderer {
        &self.world.get_resource().unwrap()
    }
    pub fn renderer_mut(&mut self) -> Mut<'_, StateRenderer> {
        self.world.get_resource_mut().unwrap()
    }
    // impl State
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            let mut renderer = self.world.get_resource_mut::<StateRenderer>().unwrap();

            renderer.size = new_size;
            //TODO:
            //renderer.camera.aspect = new_size.width as f32 / new_size.height as f32;
            renderer.config.width = new_size.width;
            renderer.config.height = new_size.height;
            renderer.depth_texture = texture::Texture::create_depth_texture(
                renderer.device(),
                renderer.config(),
                "depth_texture",
            );
            renderer
                .instance
                .surface
                .configure(&renderer.instance.device, &renderer.config);
        }
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.renderer().size
    }

    pub fn handle_event<T>(&mut self, event: &Event<T>) -> bool {
        let window = self
            .world
            .get_resource::<StateRenderer>()
            .unwrap()
            .window
            .clone();
        self.world
            .get_non_send_resource_mut::<StateImgui>()
            .unwrap()
            .handle_event(&window, event)
    }

    pub fn input(&mut self, event: &WindowEvent, can_use_mouse: bool) {
        //let mut renderer = self.world.get_resource_mut::<StateRenderer>().unwrap();

        match event {
            WindowEvent::MouseInput { state, button, .. } if can_use_mouse => self
                .world
                .send_event(MouseIn(state.clone(), button.clone())),
            WindowEvent::KeyboardInput { input, .. } => self.world.send_event(KeyIn(input.clone())),
            WindowEvent::CursorMoved { position, .. } => {
                self.world.send_event(MouseMv(position.clone()))
            }
            _ => (),
        }

        //renderer
        //    .camera_controller
        //    .process_events(event, can_use_mouse)
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        puffin::profile_function!();

        self.schedule.run(&mut self.world);

        let renderer = self.world.get_resource::<StateRenderer>().unwrap();

        let output = renderer.instance.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            renderer
                .instance
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        //renderer
        //    .camera_controller
        //    .update_camera(&mut renderer.camera);

        //renderer.camera_uniform.update_view_proj(&renderer.camera);

        renderer.instance.queue.write_buffer(
            &renderer.camera_buffer,
            0,
            bytemuck::cast_slice(&[*self
                .world
                .entity(*renderer.camera())
                .get::<CameraUniform>()
                .unwrap()]),
        );

        {
            let mut q = self.world.query::<&StateMesh>();
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
                        view: &self.renderer().depth_texture.view(),
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                });

            for mesh in q.iter(&self.world) {
                mesh.draw(&self.renderer(), &mut render_pass, &output, &view);
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

        //let mut imgui = self
        //    .world
        //    .get_non_send_resource_mut::<StateImgui>()
        //    .unwrap();
        //imgui.render_pass(&self.world.resource(), &mut encoder, &output, &view);

        // submit will accept anything that implements IntoIter
        self.renderer()
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
    pub async fn new(window: Window, world: &mut World) -> Self {
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

        let camera_controller = CameraController::new(10.0);

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let texture_dir = Arc::new(VPKDirectory::load(PathBuf::from(TEX_PATH)).unwrap());
        let misc_dir = Arc::new(VPKDirectory::load(PathBuf::from(MISC_PATH)).unwrap());

        puffin::set_scopes_on(true); // you may want to control this with a flag
                                     //let  puffin_ui = puffin_imgui::ProfilerUi::default();

        let camera_entity = world
            .spawn((
                Camera::new(1.),
                CameraController::new(10.),
                CameraUniform::default(),
            ))
            .id();

        Self {
            window: Arc::new(window),
            instance: Arc::new(StateInstance {
                surface,
                device,
                queue,
            }),
            camera_bind_group_layout,
            config,
            size,
            camera_entity,
            depth_texture,
            camera_buffer,
            camera_bind_group,
            texture_dir,
            misc_dir,
            pak: None,
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
