use glam::{Mat4, Quat, Vec3A};

use crate::transform::Transform;

pub struct Camera {
    pub transform: Transform,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols_array(
    &[
		1.0, 0.0, 0.0, 0.0,
    	0.0, 1.0, 0.0, 0.0,
    	0.0, 0.0, 0.5, 0.5,
    	0.0, 0.0, 0.0, 1.0
	]
);

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            transform: Transform::new(Vec3A::new(-6000.0, 1.0, 200.0), Quat::IDENTITY),
            aspect,
            fovy: 45.0,
            znear: 0.1,
            zfar: 10000.0,
        }
    }
    pub fn transform(&self) -> &Transform {
        &self.transform
    }
    pub fn build_view_projection_matrix(&self) -> Mat4 {
        // 1.
        //let view = Mat4::look_at_rh(
        //    (*self.transform.get_pos()).into(),
        //    Vec3::new(-5603.8857, -3556.1858, -132.67264),
        //    Vec3::Z,
        //);
        let view = Mat4::look_to_rh(
            (*self.transform.get_pos()).into(),
            self.transform.forward().into(),
            self.transform.up().into(),
        );

        // 2.
        let proj = Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);

        // 3.
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: Mat4,
}
impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY,
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix();
    }
}
