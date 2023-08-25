use super::{consts::MAX_MAP_FACES, Lump};

///The first member planenum is the plane number, i.e., the index into the plane array that corresponds to the plane that is aligned with this face in the world. Side is zero if this plane faces in the same direction as the face (i.e. "out" of the face) or non-zero otherwise.
///
///Firstedge is an index into the Surfedge array; this and the following numedges entries in the surfedge array define the edges of the face. As mentioned above, whether the value in the surfedge array is positive or negative indicates whether the corresponding pair of vertices listed in the Edge array should be traced from the first vertex to the second, or vice versa. The vertices which make up the face are thus referenced in clockwise order; when looking towards the face, each edge is traced in a clockwise direction. This makes rendering the faces easier, and allows quick culling of faces that face away from the viewpoint.
///
///Texinfo is an index into the Texinfo array (see below), and represents the texture to be drawn on the face. Dispinfo is an index into the Dispinfo array is the face is a displacement surface (in which case, the face defines the boundaries of the surface); otherwise, it is -1. SurfaceFogVolumeID appears to be related to drawing fogging when the player's viewpoint is underwater or looking through water.
///
///OrigFace is the index of the original face which was split to produce this face. NumPrims and firstPrimID are related to the drawing of "Non-polygonal primitives" (see below). The other members of the structure are used to reference face-lighting info (see the Lighting lump, below).
///
///The face array is limited to 65536 (MAX_MAP_FACES) entries.
///
///The original face lump (Lump 27) has the same structure as the face lump, but contains the array of faces before the BSP splitting process is done. These faces are therefore closer to the original brush faces present in the precompile map than the face array, and there are less of them. The origFace entry for all original faces is zero. The maximum size of the original face array is also 65536 entries.
///
///Both the face and original face arrays are culled; that is, many faces present before compilation of the map (primarily those that face towards the "void" outside the map) are removed from the array.

#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct dface_t {
    planenum: u16,                         // the plane number
    side: i8,                              // faces opposite to the node's plane direction
    onNode: i8,                            // 1 of on node, 0 if in leaf
    firstedge: i32,                        // index into surfedges
    numedges: i16,                         // number of surfedges
    texinfo: i16,                          // texture info
    dispinfo: i16,                         // displacement info
    surfaceFogVolumeID: i16,               // ?
    styles: [i8; 4],                       // switchable lighting info
    lightofs: i32,                         // offset into lightmap lump
    area: f32,                             // face area in units^2
    LightmapTextureMinsInLuxels: [i32; 2], // texture lighting info
    LightmapTextureSizeInLuxels: [i32; 2], // texture lighting info
    origFace: i32,                         // original face this was split from
    numPrims: u16,                         // primitives
    firstPrimID: u16,                      //
    smoothingGroups: u32,                  // lightmap smoothing group
}

impl Lump for dface_t {
    fn max() -> usize {
        MAX_MAP_FACES
    }

    fn validate(lump: &Box<[Self]>) {
        for face in lump.iter() {
            assert!((0..=1).contains(&face.side));
        }
        println!("validated face lump!");
    }
}
