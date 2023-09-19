// 3 pos, 4 normal, 4 tangent, 4 bone weight, 4 bone id, 2 uv

use glam::{Vec2, Vec3, Vec4};

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
pub struct ModelVertex {
    pos: Vec3,
    norm: Vec4,
    tang: Vec4,
    bone: Vec4,
    bone_id: Vec4,
    uv: Vec2,
}
