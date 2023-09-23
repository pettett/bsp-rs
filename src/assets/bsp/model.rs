use glam::Vec3;

use super::{
    consts::{LumpType, MAX_MAP_MODELS},
    Lump,
};
use getset::CopyGetters;

#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, CopyGetters)]
pub struct BSPModel {
    #[getset(get_copy = "pub")]
    mins: Vec3,
    #[getset(get_copy = "pub")]
    maxs: Vec3,
    origin: Vec3,
    headnode: i32,
    firstface: i32,
    numfaces: i32,
}

impl Lump for BSPModel {
    fn max() -> usize {
        MAX_MAP_MODELS
    }

    fn lump_type() -> super::consts::LumpType {
        LumpType::Models
    }

    //fn validate(_lump: &Box<[Self]>) {}
}
