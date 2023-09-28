use glam::Vec3;

use super::{
    consts::{LumpType, MAX_MAP_MODELS},
    Lump,
};

#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BSPModel {
    mins: Vec3,
    maxs: Vec3,
    origin: Vec3,
    headnode: i32,
    firstface: i32,
    numfaces: i32,
}

impl BSPModel {
    pub fn maxs(&self) -> Vec3 {
        self.maxs
    }

    pub fn mins(&self) -> Vec3 {
        self.mins
    }
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
