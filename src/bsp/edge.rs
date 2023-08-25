use super::{consts::MAX_MAP_EDGES, Lump};

///Edge
///
///The edge lump (Lump 12) is an array of dedge_t structures:
///Each edge is simply a pair of vertex indices (which index into the vertex lump array). The edge is defined as the straight line between the two vertices. Usually, the edge array is referenced through the Surfedge array (see below).
///
///As for vertices, edges can be shared between adjacent faces. There is a limit of 256000 edges in a map (`MAX_MAP_EDGES`).
#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct dedge_t {
    v: [u16; 2], // vertex indices
}
impl Lump for dedge_t {
    fn max() -> usize {
        MAX_MAP_EDGES
    }

    fn validate(lump: &Box<[Self]>) {
        assert!(lump.len() < MAX_MAP_EDGES);

        println!("validated edge lump!");
    }
}
