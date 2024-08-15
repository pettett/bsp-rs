use glam::IVec2;

use super::{
    consts::{LumpType, MAX_MAP_FACES},
    edges::{BSPEdge, BSPSurfEdge},
    Lump,
};

///

///The face array is limited to 65536 (MAX_MAP_FACES) entries.
///
///The original face lump (Lump 27) has the same structure as the face lump, but contains the array of faces before the BSP splitting process is done. These faces are therefore closer to the original brush faces present in the precompile map than the face array, and there are less of them. The origFace entry for all original faces is zero. The maximum size of the original face array is also 65536 entries.
///
///Both the face and original face arrays are culled; that is, many faces present before compilation of the map (primarily those that face towards the "void" outside the map) are removed from the array.

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BSPFace {
    ///The first member plane_num is the plane number, i.e., the index into the plane array that corresponds to
    /// the plane that is aligned with this face in the world.
    ///
    /// the plane number
    pub plane_num: u16, //
    /// Side is zero if this plane faces in the same direction as the face (i.e. "out" of the face) or non-zero otherwise.
    ///
    /// faces opposite to the node's plane direction
    pub side: i8,
    // 1 of on node, 0 if in leaf
    pub on_node: i8,
    /// Firstedge is an index into the Surfedge array; this and the following numedges entries in the surfedge array define the edges of the face.
    /// As mentioned above, whether the value in the surfedge array is positive or negative indicates whether the corresponding pair of vertices listed
    /// in the Edge array should be traced from the first vertex to the second, or vice versa.
    ///
    /// The vertices which make up the face are thus referenced in clockwise order; when looking towards the face,
    /// each edge is traced in a clockwise direction.
    ///
    /// This makes rendering the faces easier, and allows quick culling of faces that face away from the viewpoint.
    pub first_edge: i32,
    /// number of surfedges
    pub num_edges: i16,
    ///Texinfo is an index into the Texinfo array (see below), and represents the texture to be drawn on the face.
    pub tex_info: i16,
    /// Dispinfo is an index into the Dispinfo array is the face is a displacement surface (in which case, the face defines the boundaries of the surface); otherwise, it is -1.
    pub disp_info: i16,
    /// SurfaceFogVolumeID appears to be related to drawing fogging when the player's viewpoint is underwater or looking through water.
    pub surface_fog_volume_id: i16,
    /// switchable lighting info
    pub styles: [i8; 4],
    /// offset into lightmap lump
    pub light_ofs: i32,
    /// face area in units^2
    pub area: f32,
    /// texture lighting info
    pub lightmap_texture_mins_in_luxels: IVec2,
    /// texture lighting info
    pub lightmap_texture_size_in_luxels: IVec2,
    ///OrigFace is the index of the original face which was split to produce this face.
    pub orig_face: i32,
    /// NumPrims and firstPrimID are related to the drawing of "Non-polygonal primitives" (see below).
    /// The other members of the structure are used to reference face-lighting info (see the Lighting lump, below).
    pub num_prims: u16,
    pub first_prim_id: u16,
    /// lightmap smoothing group
    pub smoothing_groups: u32,
}

impl BSPFace {
    pub fn get_verts(&self, edges: &[BSPEdge], surfedges: &[BSPSurfEdge]) -> Vec<usize> {
        (0..self.num_edges)
            .map(|i| {
                surfedges[self.first_edge as usize + i as usize]
                    .get_edge(&edges)
                    .0 as usize
            })
            .collect()
    }
}

impl Lump for BSPFace {
    fn max() -> usize {
        MAX_MAP_FACES
    }
    fn lump_type() -> LumpType {
        LumpType::Faces
    }
}
