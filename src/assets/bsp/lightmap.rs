use bevy_ecs::system::Resource;
use glam::{vec3, vec4, Vec3, Vec4};

use super::{consts::MAX_MAP_LIGHTING, Lump, LumpType};

#[derive(Resource)]
pub struct LightingData {
    pub lighting_buffer: wgpu::Buffer,
    pub lighting_bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorRGBExp32 {
    r: u8,
    g: u8,
    b: u8,
    exponent: i8,
}

impl From<ColorRGBExp32> for Vec3 {
    fn from(value: ColorRGBExp32) -> Self {
        vec3(
            (value.r as f32) * 2f32.powi(value.exponent.into()) / 255.0,
            (value.g as f32) * 2f32.powi(value.exponent.into()) / 255.0,
            (value.b as f32) * 2f32.powi(value.exponent.into()) / 255.0,
        )
    }
}
impl From<ColorRGBExp32> for Vec4 {
    fn from(value: ColorRGBExp32) -> Self {
        let v3: Vec3 = value.into();
        vec4(v3.x, v3.y, v3.z, 1.0)
    }
}

impl Lump for ColorRGBExp32 {
    fn max() -> usize {
        MAX_MAP_LIGHTING
    }
    fn lump_type() -> LumpType {
        LumpType::Lighting
    }
}
