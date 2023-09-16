use glam::Vec3;

use super::{consts::MAX_MAP_PLANES, Lump};

///Plane
///
///The basis of the BSP geometry is defined by planes, which are used as splitting surfaces across the BSP tree structure.
///
///The plane lump (Lump 1) is an array of dplane_t structures:
///
/// f32s are 4 i8s long; there are thus 20 i8s per plane, and the plane lump should be a multiple of 20 i8s long.
///
/// The plane is represented by the element normal, a normal vector, which is a unit vector (length 1.0) perpendicular to the plane's surface. The position of the plane is given by dist, which is the distance from the map origin (0,0,0) to the nearest poi32 on the plane.
///
/// Mathematically, the plane is described by the set of poi32s (x, y, z) which satisfy the equation:
///
/// `Ax + By + Cz = D`
///
/// where A, B, and C are given by the components normal.x, normal.y and normal.z, and D is dist. Each plane is infinite in extent, and divides the whole of the map coordinate volume into three pieces, on the plane (F=0), in front of the plane (F>0), and behind the plane (F<0).
///
/// Note that planes have a particular orientation, corresponding to which side is considered "in front" of the plane, and which is "behind". The orientation of a plane can be flipped by negating the A, B, C, and D components.
///
/// The type member of the structure contains the axis that the plane is facing. It can be 0-5, with 0, 1, and 2 corresponding with X, Y, and Z respectively. Values of 3, 4 and 5 are used when planes are not along an axis, with each number corresponding to the axis it is closest to.
///
/// There can be up to 65536 planes in a map (`MAX_MAP_PLANES`).
#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BSPPlane {
    pub normal: Vec3, // normal vector
    pub dist: f32,    // distance from origin
    pub axis: i32,    // plane axis identifier
}

impl Lump for BSPPlane {
    fn max() -> usize {
        MAX_MAP_PLANES
    }

    fn validate(lump: &Box<[Self]>) {
        assert!(lump.len() < MAX_MAP_PLANES);
        for plane in lump.iter() {
            let axis = plane.axis;
            assert!((0..=5).contains(&axis));
        }
        println!("Validated planes lump!")
    }

    fn lump_type() -> super::consts::LumpType {
        super::consts::LumpType::Places
    }
}
