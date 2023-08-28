use super::{consts::MAX_MAP_EDGES, Lump};

use super::consts::{LumpType, MAX_MAP_SURFEDGES};
///Edge
///
///The edge lump (Lump 12) is an array of dedge_t structures:
///Each edge is simply a pair of vertex indices (which index into the vertex lump array). The edge is defined as the straight line between the two vertices. Usually, the edge array is referenced through the Surfedge array (see below).
///
///As for vertices, edges can be shared between adjacent faces. There is a limit of 256000 edges in a map (`MAX_MAP_EDGES`).
#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct dedge_t {
    v0: u16, // vertex indices
    v1: u16, // vertex indices
}
impl Lump for dedge_t {
    fn max() -> usize {
        MAX_MAP_EDGES
    }
    fn lump_type() -> LumpType {
        LumpType::EDGES
    }
    fn validate(lump: &Box<[Self]>) {
        assert!(lump.len() < MAX_MAP_EDGES);

        println!("validated edge lump!");
    }
}

///Surfedge
///
///The Surfedge lump (Lump 13), presumable short for surface edge, is an array of (signed) integers. Surfedges are used to reference the edge array, in a somewhat complex way.
///The value in the surfedge array can be positive or negative. The absolute value of this number is an index into the edge array:
/// if positive, it means the edge is defined from the first to the second vertex; if negative, from the second to the first vertex.
///
///By this method, the Surfedge array allows edges to be referenced for a particular direction. (See the face lump entry below for more on why this is done).
///
///There is a limit of 512000 (MAX_MAP_SURFEDGES) surfedges per map. Note that the number of surfedges is not necessarily the same as the number of edges in the map.
#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct dsurfedge_t {
    index: i32,
}

impl Lump for dsurfedge_t {
    fn max() -> usize {
        MAX_MAP_SURFEDGES
    }
    fn lump_type() -> LumpType {
        LumpType::SURFEDGES
    }
    fn validate(lump: &Box<[Self]>) {
        assert!(lump.len() < MAX_MAP_SURFEDGES);
        for edge_index in lump.iter() {
            let index = edge_index.index;
            assert!((-(MAX_MAP_EDGES as i32)..=(MAX_MAP_EDGES as i32)).contains(&index));
        }
        println!("Validated planes lump!")
    }
}

impl dsurfedge_t {
    pub fn get_edge(&self, edges: &Box<[dedge_t]>) -> (u16, u16) {
        let id = self.index.abs() as usize;
        let rev = self.index.signum();
        let edge = edges[id];
        match rev {
            0 | 1 => (edge.v0, edge.v1),
            -1 => (edge.v1, edge.v0),
            _ => panic!("Invalid edge"),
        }
    }
}
