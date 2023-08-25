use glam::Vec3;

use super::{consts::MAX_MAP_VERTS, Lump};

impl Lump for Vec3 {
    fn max() -> usize {
        MAX_MAP_VERTS
    }

    fn validate(lump: &Vec<Self>) {
        assert!(lump.len() < MAX_MAP_VERTS);

        println!("validated vert lump!");
    }
}
