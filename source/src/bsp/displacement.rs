use crate::bsp::consts::MAX_DISP_CORNER_NEIGHBORS;
use bytemuck::{Pod, Zeroable};
use flagset::flags;
use glam::Vec3;

use super::{
    consts::{LumpType, MAX_MAP_DISPINFO},
    Lump,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
pub struct BSPDispInfo {
    pub start_position: Vec3,      // start position used for orientation
    pub disp_vert_start: i32,      // Index into LUMP_DISP_VERTS.
    pub disp_tri_start: i32,       // Index into LUMP_DISP_TRIS.
    pub power: u32,                // power - indicates size of surface (2^power 1)
    pub min_tess: i32,             // minimum tesselation allowed
    pub smoothing_angle: f32,      // lighting smoothing angle
    pub contents: i32,             // surface contents
    pub map_face: u16,             // Which map face this displacement comes from.
    pub lightmap_alpha_start: i32, // Index into ddisplightmapalpha.
    pub lightmap_sample_position_start: i32, // Index into LUMP_DISP_LIGHTMAP_SAMPLE_POSITIONS.
    pub edge_neighbours: [CDispNeighbour; 4], // Indexed by NEIGHBOREDGE_ defines.
    pub corner_neighbours: [CDispCornerNeighbours; 4], // Indexed by CORNER_ defines.
    pub allowed_verts: [u32; 10],  // active verticies
}

impl Lump for BSPDispInfo {
    fn max() -> usize {
        MAX_MAP_DISPINFO as usize
    }

    fn lump_type() -> super::consts::LumpType {
        LumpType::DispInfo
    }

    //fn validate(_lump: &Box<[Self]>) {}
}
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BSPDispVert {
    pub vec: Vec3,  // Vec3 field defining displacement volume.
    pub dist: f32,  // Displacement distances.
    pub alpha: f32, // "per vertex" alpha values.
}

impl Lump for BSPDispVert {
    fn max() -> usize {
        MAX_MAP_DISPINFO as usize
    }

    fn lump_type() -> super::consts::LumpType {
        LumpType::DispVerts
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct BSPDispTri {
    tags: DispTri, // Displacement triangle tags.
}

flags! {
    #[repr(u16)]
    pub enum DispTri: u16 {
        TagSurface 		= 0x1,
        TagWalkable 	= 0x2,
        TagBuildable 	= 0x4,
        FlagSurfprop1 	= 0x8,
        FlagSurfprop2	= 0x10
    }
}

unsafe impl Zeroable for DispTri {}
unsafe impl Pod for DispTri {}

// These can be used to index g_ChildNodeIndexMul.
pub enum ChildNode {
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
pub enum NeighbourEdge {
    LEFT = 0,
    TOP = 1,
    RIGHT = 2,
    BOTTOM = 3,
}
// These define relative orientations of displacement neighbors.
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum NeighbourOrientation {
    OrientationCcw0 = 0,
    OrientationCcw90 = 1,
    OrientationCcw180 = 2,
    OrientationCcw270 = 3,
}

unsafe impl Zeroable for NeighbourOrientation {}
unsafe impl Pod for NeighbourOrientation {}
// These denote where one dispinfo fits on another.
// Note: tables are generated based on these indices so make sure to update
//       them if these indices are changed.
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum NeighbourSpan {
    CornerToCorner = 0,
    CornerToMidpoint = 1,
    MidpointToCorner = 2,
}

unsafe impl Zeroable for NeighbourSpan {}
unsafe impl Pod for NeighbourSpan {}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CDispNeighbour {
    // Note: if there is a neighbour that fills the whole side (CORNER_TO_CORNER),
    //       then it will always be in CDispNeighbour::Neighbours[0]
    pub sub_neighbours: [CDispSubNeighbour; 2],
}

// NOTE: see the section above titled "displacement neighbour rules".
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CDispSubNeighbour {
    pub i_neighbour: u16, // This indexes into ddispinfos.
    // 0xFFFF if there is no neighbour here.
    pub neighbour_orientation: u8, // (CCW) rotation of the neighbour wrt this displacement.

    // These use the NeighbourSpan type.
    pub span: u8, // Where the neighbour fits onto this side of our displacement.
    pub neighbour_span: u8, // Where we fit onto our neighbour.

    pub offset: u8,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
pub struct CDispCornerNeighbours {
    neighbours: [u16; MAX_DISP_CORNER_NEIGHBORS], // indices of neighbours.
    n_neighbours: u8,
}
