use crate::camera::Camera;
use bevy_ecs::prelude::*;
use glam::{Quat, Vec2, Vec3};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode},
};
#[derive(Component)]
pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    dragging: bool,
    last_mouse_pos: PhysicalPosition<f64>,
    mouse_delta: Vec2,
    look: Vec2,
}
#[derive(Event)]
pub struct KeyIn(pub KeyboardInput);
#[derive(Event)]
pub struct MouseIn(pub ElementState, pub MouseButton);

#[derive(Event)]
pub struct MouseMv(pub PhysicalPosition<f64>);

pub fn on_key_in(
    mut controllers: Query<(&mut CameraController,)>,
    mut ev_keyin: EventReader<KeyIn>,
) {
    for KeyIn(input) in ev_keyin.iter() {
        let KeyboardInput {
            state,
            virtual_keycode: Some(keycode),
            ..
        } = input
        else {
            continue;
        };

        for (mut c,) in controllers.iter_mut() {
            let is_pressed = *state == ElementState::Pressed;
            match keycode {
                VirtualKeyCode::W | VirtualKeyCode::Up => c.is_forward_pressed = is_pressed,
                VirtualKeyCode::A | VirtualKeyCode::Left => c.is_left_pressed = is_pressed,
                VirtualKeyCode::S | VirtualKeyCode::Down => c.is_backward_pressed = is_pressed,
                VirtualKeyCode::D | VirtualKeyCode::Right => c.is_right_pressed = is_pressed,
                VirtualKeyCode::E => c.is_up_pressed = is_pressed,
                VirtualKeyCode::Q => c.is_down_pressed = is_pressed,
                _ => (),
            }
        }
    }
}

pub fn on_mouse_in(
    mut controllers: Query<(&mut CameraController,)>,
    mut ev_keyin: EventReader<MouseIn>,
) {
    for MouseIn(state, button) in ev_keyin.iter() {
        for (mut c,) in controllers.iter_mut() {
            match button {
                winit::event::MouseButton::Left => {
                    c.dragging = match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    }
                }
                _ => (),
            }
        }
    }
}

pub fn on_mouse_mv(
    mut controllers: Query<(&mut CameraController,)>,
    mut ev_keyin: EventReader<MouseMv>,
) {
    for MouseMv(position) in ev_keyin.iter() {
        for (mut c,) in controllers.iter_mut() {
            c.mouse_delta.x = (c.last_mouse_pos.x - position.x) as f32;
            c.mouse_delta.y = (c.last_mouse_pos.y - position.y) as f32;
            c.last_mouse_pos = *position;
        }
    }
}

pub fn update_camera(mut q: Query<(&mut CameraController, &mut Camera)>) {
    for (mut controller, mut camera) in q.iter_mut() {
        let forward = camera.transform.forward();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if controller.is_forward_pressed {
            camera.transform.translate(forward * controller.speed);
        }
        if controller.is_backward_pressed {
            camera.transform.translate(-forward * controller.speed);
        }

        let left = camera.transform.left();

        if controller.is_right_pressed {
            camera.transform.translate(-left * controller.speed);
        }
        if controller.is_left_pressed {
            camera.transform.translate(left * controller.speed);
        }

        let up = camera.transform.up();

        if controller.is_down_pressed {
            camera.transform.translate(-up * controller.speed);
        }
        if controller.is_up_pressed {
            camera.transform.translate(up * controller.speed);
        }

        // rotate camera
        if controller.dragging {
            let off = controller.mouse_delta / 100.0;
            controller.look += off;

            let rot = Quat::from_axis_angle(Vec3::Z, controller.look.x)
                * Quat::from_axis_angle(Vec3::Y, -controller.look.y);

            *camera.transform.get_rot_mut() = rot;
        }
        controller.mouse_delta = Vec2::ZERO;
    }
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            dragging: false,
            look: Default::default(),
            last_mouse_pos: Default::default(),
            mouse_delta: Default::default(),
        }
    }
}
