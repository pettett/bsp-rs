pub mod assets;
pub mod binaries;
pub mod camera;
pub mod camera_controller;
pub mod game_data;
pub mod geo;
#[cfg(feature = "desktop")]
pub mod gui;
pub mod state;
pub mod transform;
pub mod v;
pub mod vertex;

use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use bevy_ecs::world::World;
use state::StateApp;

use v::vrenderer::VRenderer;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub async fn vinit() -> (StateApp, EventLoop<()>) {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    (vinit_state(window).await, event_loop)
}

pub async fn vinit_state(window: winit::window::Window) -> StateApp {
    let mut world = World::default();

    let renderer = VRenderer::new(window, &mut world).await;

    StateApp::new(world, renderer)
}

pub fn vrun(mut state: StateApp, event_loop: EventLoop<()>) -> ! {
    //let instance = state.renderer().instance();
    //let debug_mesh = state.debug_mesh.clone();
    //thread::spawn(move || {
    //    EdgesDebugMesh::load_mesh(debug_mesh, instance);
    //});
    //
    //let instance = state.renderer().instance();
    //let debug_mesh = state.faces_debug_mesh.clone();
    //thread::spawn(move || {
    //    FacesDebugMesh::load_mesh(debug_mesh, instance);
    //});

    event_loop.run(move |event, _, control_flow| {
        let mouse_capture = state.handle_event(&event);

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.renderer().window().id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    // new_inner_size is &&mut so we have to dereference it twice
                    state.resize(**new_inner_size);
                }

                event => {
                    state.input(event, !mouse_capture);
                }
            },
            Event::RedrawRequested(window_id) if window_id == state.renderer().window().id() => {
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => {
                        let size = state.size();
                        state.resize(size);
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.renderer().window().request_redraw();
            }
            _ => {}
        };
    })
}
