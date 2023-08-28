use crate::bsp::consts::MAX_DISP_CORNER_NEIGHBORS;
use bytemuck::{Pod, Zeroable};
use flagset::flags;
use glam::Vec3;

#[repr(C, packed)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct BSPDispTri {
    tags: DispTri, // Displacement triangle tags.
}

flags! {
    #[repr(u16)]
    enum DispTri: u16 {
        TAG_SURFACE 	= 0x1,
        TAG_WALKABLE 	= 0x2,
        TAG_BUILDABLE = 0x4,
        FLAG_SURFPROP1 = 0x8,
        FLAG_SURFPROP2 = 0x10
    }
}

unsafe impl Zeroable for DispTri {}
unsafe impl Pod for DispTri {}

#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct BSPDispVert {
    vec: Vec3,  // Vec3 field defining displacement volume.
    dist: f32,  // Displacement distances.
    alpha: f32, // "per vertex" alpha values.
}

// These can be used to index g_ChildNodeIndexMul.
enum ChildNode {
    UpperRight = 0,
    UpperLeft = 1,
    LowerLeft = 2,
    LowerRight = 3,
}

// Corner indices. Used to index CornerNeighbours.
pub enum Corner {
    LowerLeft = 0,
    UpperLeft = 1,
    UpperRight = 2,
    LowerRight = 3,
}

// These edge indices must match the edge indices of the CCoreDispSurface.
enum NeighbourEdge {
    LEFT = 0,
    TOP = 1,
    RIGHT = 2,
    BOTTOM = 3,
}

#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct BSPDispInfo {
    start_position: Vec3,                 // start position used for orientation
    disp_vert_start: i32,                 // Index into LUMP_DISP_VERTS.
    disp_tri_start: i32,                  // Index into LUMP_DISP_TRIS.
    power: i32,                           // power - indicates size of surface (2^power 1)
    min_tess: i32,                        // minimum tesselation allowed
    smoothing_angle: f32,                 // lighting smoothing angle
    contents: i32,                        // surface contents
    map_face: u16,                        // Which map face this displacement comes from.
    lightmap_alpha_start: i32,            // Index into ddisplightmapalpha.
    lightmap_sample_position_start: i32,  // Index into LUMP_DISP_LIGHTMAP_SAMPLE_POSITIONS.
    edge_neighbours: [CDispNeighbour; 4], // Indexed by NEIGHBOREDGE_ defines.
    corner_neighbours: [CDispCornerNeighbours; 4], // Indexed by CORNER_ defines.
    allowed_verts: [u32; 10],             // active verticies
}
#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CDispNeighbour {
    // Note: if there is a neighbour that fills the whole side (CORNER_TO_CORNER),
    //       then it will always be in CDispNeighbour::Neighbours[0]
    sub_neighbours: [CDispSubNeighbour; 2],
}

// NOTE: see the section above titled "displacement neighbour rules".
#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CDispSubNeighbour {
    i_neighbour: u16, // This indexes into ddispinfos.
    // 0xFFFF if there is no neighbour here.
    neighbour_orientation: u8, // (CCW) rotation of the neighbour wrt this displacement.

    // These use the NeighbourSpan type.
    span: u8,           // Where the neighbour fits onto this side of our displacement.
    neighbour_span: u8, // Where we fit onto our neighbour.
}

#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CDispCornerNeighbours {
    neighbours: [u16; MAX_DISP_CORNER_NEIGHBORS], // indices of neighbours.
    n_neighbours: u8,
}
