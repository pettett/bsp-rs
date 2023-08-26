use glam::{Quat, Vec2, Vec3, Vec3A};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
};

use crate::camera::Camera;

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
            last_mouse_pos: Default::default(),
            mouse_delta: Default::default(),
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::E => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Q => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
                ..
            } => match button {
                winit::event::MouseButton::Left => {
                    self.dragging = match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };
                    true
                }
                _ => false,
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_delta.x = (self.last_mouse_pos.x - position.x) as f32;
                self.mouse_delta.y = (self.last_mouse_pos.y - position.y) as f32;
                self.last_mouse_pos = *position;
                true
            }

            _ => false,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera) {
        let forward = camera.transform.forward();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed {
            camera.transform.translate(forward * self.speed);
        }
        if self.is_backward_pressed {
            camera.transform.translate(-forward * self.speed);
        }

        let left = camera.transform.left();

        if self.is_right_pressed {
            camera.transform.translate(-left * self.speed);
        }
        if self.is_left_pressed {
            camera.transform.translate(left * self.speed);
        }

        let up = camera.transform.up();

        if self.is_down_pressed {
            camera.transform.translate(-up * self.speed);
        }
        if self.is_up_pressed {
            camera.transform.translate(up * self.speed);
        }

        // rotate camera
        if self.dragging {
            let rot = camera.transform.get_rot_mut();
            *rot = Quat::from_axis_angle(Vec3::Z, self.mouse_delta.x / 100.0) * *rot;

            *rot *= Quat::from_axis_angle(left.into(), -self.mouse_delta.y / 100.0);
        }
        self.mouse_delta = Vec2::ZERO;
    }
}
