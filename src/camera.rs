use glam::{Mat4, Vec3};

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
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
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: Vec3::new(2000.0, 1.0, 200.0),
            // have it look at the origin
            target: Vec3::new(0.0, 0.0, 0.0),
            // which way is "up"
            up: Vec3::Z,
            aspect,
            fovy: 45.0,
            znear: 0.1,
            zfar: 10000.0,
        }
    }
    pub fn build_view_projection_matrix(&self) -> Mat4 {
        // 1.
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
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
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}
impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d();
    }
}
