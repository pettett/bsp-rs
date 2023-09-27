use std::path::PathBuf;
use std::sync::Arc;

#[cfg(feature = "desktop")]
use std::{thread, thread::JoinHandle};

use std::{mem, time};

use crate::assets::bsp::loader::load_bsp;
use crate::camera::{update_view_proj, Camera};
use crate::camera_controller::{
    on_key_in, on_mouse_in, on_mouse_mv, update_camera, KeyIn, MouseIn, MouseMv,
};
use crate::game_data::GameDataArc;
#[cfg(feature = "desktop")]
use crate::gui::{Gui, GuiWindow, TaskViewer};
use crate::v::vfile::VFileSystem;
use crate::v::vrenderer::{draw_static, VRenderer};
use crate::v::VTexture;
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemState;
use winit::dpi::PhysicalSize;
use winit::event::*;

pub trait State {
    fn render_pass(
        &mut self,
        renderer: &VRenderer,
        encoder: &mut wgpu::CommandEncoder,
        output: &wgpu::SurfaceTexture,
        view: &wgpu::TextureView,
    );

    fn init(renderer: &VRenderer) -> Self;
}

pub struct StateApp {
    world: World,
    schedule: Schedule,
}

/// Data that will be read only for the course of the program, containing everything needed to create shaders and pipelines
pub struct StateInstance {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub format: wgpu::TextureFormat,

    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub lighting_bind_group_layout: wgpu::BindGroupLayout,
}

#[derive(bevy_ecs::event::Event)]
pub struct MapChangeEvent(pub PathBuf);

#[derive(bevy_ecs::event::Event)]
pub struct Test();

pub type CommandTaskResult = Box<dyn Sync + Send + FnOnce(&mut Commands) -> ()>;

#[derive(Component)]
pub struct CommandTask {
    pub name: &'static str,
    #[cfg(feature = "desktop")]
    handle: Option<JoinHandle<CommandTaskResult>>,
    #[cfg(feature = "wasm")]
    handle: Option<CommandTaskResult>,
}

/// A quite annoying function to cast to the correct dyn type
pub fn box_cmds(f: impl Sync + Send + FnOnce(&mut Commands) -> () + 'static) -> CommandTaskResult {
    Box::new(f) as CommandTaskResult
}

pub fn spawn_command_task(
    commands: &mut Commands,
    name: &'static str,
    f: impl 'static + Send + FnOnce() -> CommandTaskResult,
) {
    commands.spawn(command_task(name, f));
}

pub fn command_task(
    name: &'static str,
    f: impl 'static + Send + FnOnce() -> CommandTaskResult,
) -> CommandTask {
    #[cfg(feature = "desktop")]
    let t = CommandTask {
        name,
        handle: Some(thread::spawn(f)),
    };
    #[cfg(feature = "wasm")]
    let t = CommandTask {
        name,
        handle: Some(f()),
    };

    t
}

// fn test(mut q: EventReader<Test>, mut c: Commands) {
//     for t in q.iter() {
//         println!("Got event!");

//         spawn_command_task(&mut c, || {
//             let ten_millis = time::Duration::from_millis(1000);

//             thread::sleep(ten_millis);

//             box_cmds(|c: &mut Commands| c.add(|w: &mut World| w.send_event(Test())))
//         });
//     }
// }

fn complete_command_tasks(mut q: Query<(Entity, &mut CommandTask)>, mut c: Commands) {
    for (e, mut p) in q.iter_mut() {
        #[cfg(feature = "desktop")]
        if p.handle.as_ref().unwrap().is_finished() {
            let t = mem::replace(&mut p.handle, None).unwrap().join();

            c.entity(e).despawn();

            if let Ok(cmd) = t {
                cmd(&mut c);
            }
        }
        #[cfg(feature = "wasm")]
        {
            let t = mem::replace(&mut p.handle, None);

            c.entity(e).despawn();

            if let Some(cmd) = t {
                cmd(&mut c);
            }
        }
    }
}

impl StateApp {
    /// Creating some of the wgpu types requires async code
    /// https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#state-new
    pub fn new(mut world: World, renderer: VRenderer) -> Self {
        #[cfg(feature = "desktop")]
        world.insert_non_send_resource(Gui::init(&renderer));
        world.insert_non_send_resource(renderer);

        let mut schedule = Schedule::default();

        world.insert_resource(Events::<MouseIn>::default());
        world.insert_resource(Events::<MouseMv>::default());
        world.insert_resource(Events::<KeyIn>::default());
        world.insert_resource(Events::<MapChangeEvent>::default());
        world.insert_resource(Events::<Test>::default());

        world.send_event(Test());
        #[cfg(feature = "desktop")]
        world.spawn(GuiWindow::new_open(Arc::new(TaskViewer::new())));

        // Add our system to the schedule
        schedule.add_systems((
            load_map,
            //    test,
            complete_command_tasks,
            on_mouse_in,
            on_mouse_mv,
            on_key_in,
            update_camera,
            update_view_proj,
            draw_static,
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
    pub fn renderer(&self) -> &VRenderer {
        &self.world.get_non_send_resource().unwrap()
    }
    pub fn renderer_mut(&mut self) -> Mut<'_, VRenderer> {
        self.world.get_non_send_resource_mut().unwrap()
    }
    // impl State
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            let mut state =
                SystemState::<(Query<&mut Camera>, NonSendMut<VRenderer>)>::new(&mut self.world);

            let (mut cameras, mut renderer) = state.get_mut(&mut self.world);

            renderer.size = new_size;

            for mut camera in cameras.iter_mut() {
                camera.aspect = new_size.width as f32 / new_size.height as f32;
            }
            renderer.config.width = new_size.width;
            renderer.config.height = new_size.height;
            renderer.depth_texture = VTexture::create_depth_texture(
                renderer.device(),
                renderer.config(),
                "depth_texture",
            );
            renderer
                .surface()
                .configure(&renderer.device(), &renderer.config());
        }
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.renderer().size
    }

    pub fn handle_event<T>(&mut self, event: &winit::event::Event<T>) -> bool {
        let window = self
            .world
            .get_non_send_resource::<VRenderer>()
            .unwrap()
            .window()
            .clone();

        #[cfg(feature = "desktop")]
        let r = self
            .world
            .get_non_send_resource_mut::<Gui>()
            .unwrap()
            .handle_event(&window, event);

        #[cfg(not(feature = "desktop"))]
        let r = false;

        r
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

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        //puffin::profile_function!();

        self.schedule.run(&mut self.world);

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
        //

        Ok(())
    }
}

pub fn load_map(
    mut events: EventReader<MapChangeEvent>,
    mut commands: Commands,
    renderer: NonSend<VRenderer>,
    game_data_opt: Option<Res<GameDataArc>>,
    file_system_opt: Option<Res<VFileSystem>>,
) {
    for e in events.iter() {
        if let Some(game_data) = &game_data_opt {
            println!("Loading map {:?}", e.0);

            load_bsp(
                &e.0,
                &mut commands,
                game_data.inner.clone(),
                &renderer,
                //TODO:
                match file_system_opt.as_ref() {
                    Some(sys) => Some(VFileSystem {
                        files: sys.files.clone(),
                    }),
                    None => None,
                },
            )
        } else {
            panic!("No game data to load map");
        }
    }
}
