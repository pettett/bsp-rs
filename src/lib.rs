pub mod binaries;
pub mod bsp;
pub mod camera;
pub mod camera_controller;
pub mod state;
pub mod state_imgui;
pub mod state_mesh;
pub mod texture;
pub mod transform;
pub mod vertex;
pub mod vmt;
pub mod vpk;
pub mod vtf;
use std::thread;

use state::{StateApp, StateRenderer};
use state_mesh::StateMesh;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub async fn run<F>(init: F)
where
    F: FnOnce(&mut StateApp) -> (),
{
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let renderer = StateRenderer::new(window).await;

    let mut state = StateApp::new(renderer).await;

    init(&mut state);

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
        state.handle_event(&event);

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
                    state.input(event);
                }
            },
            Event::RedrawRequested(window_id) if window_id == state.renderer().window().id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size()),
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
    });
}
