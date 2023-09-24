use std::path::PathBuf;
use std::thread::JoinHandle;
use std::{mem, thread, time};

use crate::assets::bsp::loader::load_bsp;
use crate::camera::update_view_proj;
use crate::camera_controller::{
    on_key_in, on_mouse_in, on_mouse_mv, update_camera, KeyIn, MouseIn, MouseMv,
};
use crate::game_data::GameData;
use crate::gui::state_imgui::StateImgui;
use crate::v::vrenderer::{draw_static, VRenderer};
use crate::v::VTexture;
use bevy_ecs::prelude::*;
use ini::Ini;
use rayon::ThreadPool;
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

/// Data that will be read only for the course of the program
pub struct StateInstance {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl StateInstance {
    pub fn new(surface: wgpu::Surface, device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self {
            surface,
            device,
            queue,
        }
    }

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
pub struct MapChangeEvent(pub PathBuf);

#[derive(bevy_ecs::event::Event)]
pub struct Test();
#[derive(Component)]
pub struct SpawnBatchTask {
    handle: Option<JoinHandle<Box<dyn Send + Fn(&mut Commands) -> ()>>>,
}

fn test(mut q: EventReader<Test>, mut c: Commands) {
    for t in q.iter() {
        println!("Got event!");

        c.spawn(SpawnBatchTask {
            handle: Some(thread::spawn(|| {
                let ten_millis = time::Duration::from_millis(1000);

                thread::sleep(ten_millis);

                Box::new(|c: &mut Commands| c.add(|w: &mut World| w.send_event(Test())))
                    as Box<dyn Send + Fn(&mut Commands) -> ()>
            })),
        });
    }
}

fn complete(mut q: Query<(Entity, &mut SpawnBatchTask)>, mut c: Commands) {
    for (e, mut p) in q.iter_mut() {
        if p.handle.as_ref().unwrap().is_finished() {
            let t = mem::replace(&mut p.handle, None).unwrap().join().unwrap();

            println!("Finished event: ");

            c.entity(e).despawn();

            t(&mut c);
        }
    }
}

impl StateApp {
    /// Creating some of the wgpu types requires async code
    /// https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#state-new
    pub fn new(mut world: World, renderer: VRenderer) -> Self {
        world.insert_non_send_resource(StateImgui::init(&renderer));
        world.insert_resource(renderer);

        let mut schedule = Schedule::default();

        world.insert_resource(Events::<MouseIn>::default());
        world.insert_resource(Events::<MouseMv>::default());
        world.insert_resource(Events::<KeyIn>::default());
        world.insert_resource(Events::<MapChangeEvent>::default());
        world.insert_resource(Events::<Test>::default());

        world.send_event(Test());

        // Add our system to the schedule
        schedule.add_systems((
            load_map,
            test,
            complete,
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
        &self.world.get_resource().unwrap()
    }
    pub fn renderer_mut(&mut self) -> Mut<'_, VRenderer> {
        self.world.get_resource_mut().unwrap()
    }
    // impl State
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            let mut renderer = self.world.get_resource_mut::<VRenderer>().unwrap();

            renderer.size = new_size;
            //TODO:
            //renderer.camera.aspect = new_size.width as f32 / new_size.height as f32;
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
            .get_resource::<VRenderer>()
            .unwrap()
            .window()
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

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        puffin::profile_function!();

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
    renderer: Res<VRenderer>,
    game_data_opt: Option<Res<GameData>>,
) {
    if let Some(game_data) = &game_data_opt {
        for e in events.iter() {
            load_bsp(&e.0, &mut commands, &game_data, &renderer)
        }
    }
}
