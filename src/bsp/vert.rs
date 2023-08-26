use glam::Vec3;

use super::{consts::MAX_MAP_VERTS, Lump};

impl Lump for Vec3 {
    fn max() -> usize {
        MAX_MAP_VERTS
    }

    fn validate(lump: &Box<[Self]>) {
        assert!(lump.len() < MAX_MAP_VERTS);

        let mut sum = Vec3::ZERO;
        for &v in lump.iter() {
            sum += v;
        }
        sum /= lump.len() as f32;

        println!("Average vertex {sum}");
        println!("validated vert lump!");
    }
}
