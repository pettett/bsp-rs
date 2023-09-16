use glam::{vec3, Vec3};

use super::{consts::MAX_MAP_LIGHTING, Lump, LumpType};

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
            (value.r as f32).powi(value.exponent.into()) / 255.0,
            (value.g as f32).powi(value.exponent.into()) / 255.0,
            (value.b as f32).powi(value.exponent.into()) / 255.0,
        )
    }
}

impl Lump for ColorRGBExp32 {
    fn max() -> usize {
        MAX_MAP_LIGHTING
    }
    fn lump_type() -> LumpType {
        LumpType::Lighting
    }
    fn validate(lump: &Box<[Self]>) {}
}
