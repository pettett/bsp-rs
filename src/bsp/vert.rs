use glam::Vec3;

use super::{consts::MAX_MAP_VERTS, Lump};

impl Lump for Vec3 {
    fn max() -> usize {
        MAX_MAP_VERTS
    }

    fn validate(lump: &Box<[Self]>) {
        assert!(lump.len() < MAX_MAP_VERTS);

        for i in 0..100 {
            println!("{:?}", lump[i * 100]);
        }

        println!("validated vert lump!");
    }
}
