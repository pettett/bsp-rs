// This article's documentation is for anything that uses the Source engine. Click here for more information.
// MDL (Source)
// Quake
// GoldSrc
// Source Engine

// MDL is the extension for Source's proprietary model format. It defines the structure of the model along with animation, bounding box, hit box, materials, mesh and LOD information. It does not, however, contain all the information needed for the model. Additional data is stored in PHY, ANI, VTX and VVD files, and sometimes, usually for shared animations, other .mdl files.
// Contents

//     1 File format
//         1.1 Main header
//         1.2 Secondary header
//         1.3 Texture data
//         1.4 Skin replacement tables
//     2 See also

// File format

// Some details of the file format may be gleaned from the source code in Valve's studio.h, specifically the struct studiohdr_t. The early header defines a series of offsets and lengths for various sub-sections within the file, along with some key scalar information. The MDL also contains the names of materials (VMT), which may be used and referenced in various ways.
// Main header

// To get the latest header for specific game, please use the studio.h file in the Valve's SDK instead.

use std::{
    io::{self, BufRead, BufReader, Read, Seek, SeekFrom},
    marker::PhantomData,
    mem,
};

use fixedstr::zstr;
use glam::Vec3;

use crate::binaries::BinaryData;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct MDLOffset {
    pub index: i32,
}

impl MDLOffset {
    pub fn read_str<R: Read + Seek>(
        &self,
        buffer: &mut BufReader<R>,
        start: i64,
        pos: &mut i64,
    ) -> io::Result<String> {
        buffer.seek_relative((self.index as i64 + start) - *pos)?;
        *pos += self.index as i64 + start;

        //let mut b = [0, 0, 0, 0];
        //buffer.read_exact(&mut b)?;
        //println!("{b:?}");
        let mut data = Default::default();

        *pos += buffer.read_until(0, &mut data)? as i64;

        Ok(String::from_utf8(data).unwrap())
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct MDLArray<T: Sized + BinaryData> {
    pub count: i32,
    pub offset: i32,
    _p: PhantomData<T>,
}

impl<T: Sized + BinaryData> MDLArray<T> {
    pub fn read<R: Read + Seek>(
        &self,
        buffer: &mut BufReader<R>,
        pos: &mut i64,
    ) -> io::Result<Vec<(i64, T)>> {
        buffer.seek_relative(self.offset as i64 - *pos)?;
        *pos = self.offset as i64;

        //let mut b = [0, 0, 0, 0];
        //buffer.read_exact(&mut b)?;
        //println!("{b:?}");

        let mut v = Vec::new();
        v.reserve(self.count as usize);

        for _ in 0..self.count {
            v.push((*pos, T::read(buffer, None)?));
            *pos += mem::size_of::<T>() as i64;
        }

        Ok(v)
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct MDLHeader {
    pub id: [u8; 4],    // Model format ID, such as "IDST" (0x49 0x44 0x53 0x54)
    pub version: i32,   // Format version number, such as 48 (0x30,0x00,0x00,0x00)
    pub checksum: i32,  // This has to be the same in the phy and vtx files to load!
    pub name: zstr<64>, // The internal name of the model, padding with null bytes.
    // Typically "my_model.mdl" will have an internal name of "my_model"
    pub data_length: i32, // Data size of MDL file in bytes.

    // A Vec3 is 12 bytes, three 4-byte f32-values in a row.
    pub eyeposition: Vec3, // Position of player viewpoint relative to model origin
    pub illumposition: Vec3, // Position (relative to model origin) used to calculate ambient light contribution and cubemap reflections for the entire model.
    pub hull_min: Vec3,      // Corner of model hull box with the least X/Y/Z values
    pub hull_max: Vec3,      // Opposite corner of model hull box
    pub view_bbmin: Vec3,    // TODO: what's this, how is it different from hull_min/max?
    pub view_bbmax: Vec3,

    pub flags: i32, // Binary flags in little-endian order.
    // ex (00000001,00000000,00000000,11000000) means flags for position 0, 30, and 31 are set.
    // Set model flags section for more information

    /*
     * After this point, the header contains many references to offsets
     * within the MDL file and the number of items at those offsets.
     *
     * Offsets are from the very beginning of the file.
     *
     * Note that indexes/counts are not always paired and ordered consistently.
     */
    // mstudiobone_t
    pub bone: MDLArray<mstudionill_t>, // Number of data sections (of type mstudiobone_t)

    // mstudiobonecontroller_t
    pub bonecontroller: MDLArray<mstudionill_t>,

    // mstudiohitboxset_t
    pub hitbox: MDLArray<mstudionill_t>,

    // mstudioanimdesc_t
    pub localanim: MDLArray<mstudionill_t>,

    // mstudioseqdesc_t
    pub localseq: MDLArray<mstudionill_t>,

    pub activitylistversion: i32, // ??
    pub eventsindexed: i32,       // ??

    // VMT texture filenames
    // mstudiotexture_t
    pub texture: MDLArray<mstudiotexture_t>,

    // This offset points to a series of i32s.
    // Each i32 value, in turn, is an offset relative to the start of this header/the-file,
    // At which there is a null-terminated string.
    pub texturedir: MDLArray<mstudionill_t>,

    // Each skin-family assigns a texture-id to a skin location
    pub skinreference_count: i32,
    pub skinrfamily_count: i32,
    pub skinreference_index: i32,

    // mstudiobodyparts_t
    pub bodypart: MDLArray<mstudiobodyparts_t>,

    // Local attachment points
    // mstudioattachment_t
    pub attachment: MDLArray<mstudionill_t>,

    // Node values appear to be single bytes, while their names are null-terminated strings.
    pub localnode: MDLArray<mstudionill_t>,
    pub localnode_name_index: MDLOffset,

    // mstudioflexdesc_t
    pub flexdesc: MDLArray<mstudionill_t>,

    // mstudioflexcontroller_t
    pub flexcontroller: MDLArray<mstudionill_t>,

    // mstudioflexrule_t
    pub flexrules: MDLArray<mstudionill_t>,

    // IK probably referse to inverse kinematics
    // mstudioikchain_t
    pub ikchain: MDLArray<mstudionill_t>,

    // Information about any "mouth" on the model for speech animation
    // More than one sounds pretty creepy.
    // mstudiomouth_t
    pub mouths: MDLArray<mstudionill_t>,

    // mstudioposeparamdesc_t
    pub localposeparam: MDLArray<mstudionill_t>,

    /*
     * For anyone trying to follow along, as of this writing,
     * the next "surfaceprop_index" value is at position 0x0134 (308)
     * from the start of the file.
     */
    // Surface property value (single null-terminated string)
    pub surfaceprop_index: MDLOffset,

    // Unusual: In this one index comes first, then count.
    // Key-value data is a series of strings. If you can't find
    // what you're i32erested in, check the associated PHY file as well.
    pub keyvalue_index: MDLArray<mstudionill_t>,

    // More inverse-kinematics
    // mstudioiklock_t
    pub iklock: MDLArray<mstudionill_t>,

    pub mass: f32,     // Mass of object (4-bytes)
    pub contents: i32, // ??

    // Other models can be referenced for re-used sequences and animations
    // (See also: The $includemodel QC option.)
    // mstudiomodelgroup_t
    pub includemodel: MDLArray<mstudionill_t>,

    pub virtual_model: i32, // Placeholder for mutable-void*
    // Note that the SDK only compiles as 32-bit, so an i32 and a pointer are the same size (4 bytes)

    // mstudioanimblock_t
    pub animblocks_name_index: MDLOffset,
    pub animblocks: MDLArray<mstudionill_t>,

    pub animblock_model: i32, // Placeholder for mutable-void*

    // points to a series of bytes?
    pub bonetablename_index: i32,

    pub vertex_base: i32, // Placeholder for void*
    pub offset_base: i32, // Placeholder for void*

    // Used with $constantdirectionallight from the QC
    // Model should have flag #13 set if enabled
    pub directionaldotproduct: i8,

    pub root_lod: i8, // Preferred rather than clamped

    // 0 means any allowed, N means Lod 0 -> (N-1)
    pub num_allowed_root_lods: i8,

    unused0: i8,  // ??
    unused1: i32, // ??

    // mstudioflexcontrollerui_t
    pub flexcontrollerui: MDLArray<mstudionill_t>,

    pub vertAnimFixedpointScale: f32, // ??
    unused2: i32,

    /**
     * Offset for additional header information.
     * May be zero if not present, or also 408 if it immediately
     * follows this studiohdr_t
     */
    // studiohdr2_t
    pub tudiohdr2index: i32,

    pub unused3: i32, // ??

                      // /**
                      //  * As of this writing, the header is 408 bytes long in total
                      //  */
}

impl BinaryData for MDLHeader {}
unsafe impl bytemuck::Zeroable for MDLHeader {}

// body part index

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct mstudiobodyparts_t {
    pub name_index: u32, // index into models array
    pub nummodels: u32,
    pub base: u32,
    pub modelindex: u32,
}

impl BinaryData for mstudiobodyparts_t {}
unsafe impl bytemuck::Zeroable for mstudiobodyparts_t {}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct mstudiotexture_t {
    // Number of bytes past the beginning of this structure
    // where the first character of the texture name can be found.
    pub name_offset: MDLOffset, // Offset for null-terminated string
    flags: i32,

    used: i32,   // Padding?
    unused: i32, // Padding.

    material: i32,        // Placeholder for IMaterial
    client_material: i32, // Placeholder for void*

    unused2: [i32; 10], // Final padding
                        // Struct is 64 bytes long
}

impl BinaryData for mstudiotexture_t {}
unsafe impl bytemuck::Zeroable for mstudiotexture_t {}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct mstudiomodel_t {
    name: zstr<64>,
    t: i32,
    boundingradius: f32,
    nummeshes: i32,
    meshindex: i32,
    numvertices: i32,   // number of unique vertices/normals/texcoords
    vertexindex: i32,   // vertex Vector
    tangentsindex: i32, // tangents Vector
    numattachments: i32,
    attachmentindex: i32,
    numeyeballs: i32,
    eyeballindex: i32,
    pVertexData: i32,
    // base of external vertex data stores
    pTangentData: i32,
    unused: [i32; 8], // remove as appropriate
}

impl BinaryData for mstudiomodel_t {}
unsafe impl bytemuck::Zeroable for mstudiomodel_t {}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct mstudionill_t;

impl BinaryData for mstudionill_t {}
unsafe impl bytemuck::Zeroable for mstudionill_t {}

// Known flags Name 	Position 	Details
// STUDIOHDR_FLAGS_AUTOGENERATED_HITBOX 	0 	This flag is set if no hitbox information was specified
// STUDIOHDR_FLAGS_USES_ENV_CUBEMAP 	1 	This flag is set at loadtime, not mdl build time so that we don't have to rebuild models when we change materials.
// STUDIOHDR_FLAGS_FORCE_OPAQUE 	2 	Use this when there are translucent parts to the model but we're not going to sort it.
// STUDIOHDR_FLAGS_TRANSLUCENT_TWOPASS 	3 	Use this when we want to render the opaque parts during the opaque pass and the translucent parts during the translucent pass. Added using $mostlyopaque to the QC.
// STUDIOHDR_FLAGS_STATIC_PROP 	4 	This is set any time the .qc files has $staticprop in it. Means there's no bones and no transforms.
// STUDIOHDR_FLAGS_USES_FB_TEXTURE 	5 	This flag is set at loadtime, not mdl build time so that we don't have to rebuild models when we change materials.
// STUDIOHDR_FLAGS_HASSHADOWLOD 	6 	This flag is set by studiomdl.exe if a separate "$shadowlod" entry was present for the .mdl (the shadow lod is the last entry in the lod list if present).
// STUDIOHDR_FLAGS_USES_BUMPMAPPING 	7 	This flag is set at loadtime, not mdl build time so that we don't have to rebuild models when we change materials.
// STUDIOHDR_FLAGS_USE_SHADOWLOD_MATERIALS 	8 	This flag is set when we should use the actual materials on the shadow LOD instead of overriding them with the default one (necessary for translucent shadows).
// STUDIOHDR_FLAGS_OBSOLETE 	9 	This flag is set when we should use the actual materials on the shadow LOD instead of overriding them with the default one (necessary for translucent shadows).
// STUDIOHDR_FLAGS_UNUSED 	10
// STUDIOHDR_FLAGS_NO_FORCED_FADE 	11 	This flag is set at mdl build time.
// STUDIOHDR_FLAGS_FORCE_PHONEME_CROSSFADE 	12 	The npc will lengthen the viseme check to always include two phonemes.
// STUDIOHDR_FLAGS_CONSTANT_DIRECTIONAL_LIGHT_DOT 	13 	This flag is set when the .qc has $constantdirectionallight in it. If set, we use constantdirectionallightdot to calculate light i32ensity rather than the normal directional dot product. Only valid if STUDIOHDR_FLAGS_STATIC_PROP is also set.
// STUDIOHDR_FLAGS_FLEXES_CONVERTED 	14 	Flag to mark delta flexes as already converted from disk format to memory format.
// STUDIOHDR_FLAGS_BUILT_IN_PREVIEW_MODE 	15 	Indicates the studiomdl was built in preview mode (added with the -preview flag).
// STUDIOHDR_FLAGS_AMBIENT_BOOST 	16 	Ambient boost (runtime flag).
// STUDIOHDR_FLAGS_DO_NOT_CAST_SHADOWS 	17 	Don't cast shadows from this model (useful on first-person models).
// STUDIOHDR_FLAGS_CAST_TEXTURE_SHADOWS 	18 	Alpha textures should cast shadows in vrad on this model (ONLY prop_static!). Requires setup in the lights.rad file.
// Secondary header

// This header section is optional, and is found via the studiohdr2index value.

// struct studiohdr2_t
// {
//         // ??
//         i32    srcbonetransform_count:
//         i32    srcbonetransform_index:

//         i32    illumpositionattachmentindex:

//         f32  flMaxEyeDeflection:    //  If set to 0, then equivalent to cos(30)

//         // mstudiolinearbone_t
//         i32    linearbone_index:

//         i32    unknown[64]:
// }:

// Texture data

// struct mstudiotexture_t
// {
//         // Number of bytes past the beginning of this structure
//         // where the first byteacter of the texture name can be found.
//         i32    name_offset: // Offset for null-terminated string
//         i32    flags:

//         i32    used:        // Padding?
//         i32    unused:      // Padding.

//         i32    material:        // Placeholder for IMaterial
//         i32    client_material: // Placeholder for void*

//         i32    unused2[10]: // Final padding
//         // Struct is 64 bytes long
// }:

// Skin replacement tables

// Each "skin" that a model has (as seen in the Model Viewer and choose-able for prop entities) is actually referred to as a skin "family" in the MDL code. (Additional skins may be created with the $texturegroup compile option.) For the purposes of this section, a "skin" an area where a single material (aka texture) may be applied, while a skin-family defines a list of materials to use on skin zones 0,1,2, etc.

// The skin-family section of the MDL is a sequence of short (2-byte) values, which should be broken up i32o a table. To illustrate how it works, we will consider a model and then examine how it would be represented on the byte-level. For this example, imagine a crate-model with three skin-families labeled (fam0,fam1,fam2) two skins-zones (mainbody,trimming) and three textures (lightwood,darkwood,metal).
// Skin table 	mainbody 	trimming
// fam0 	lightwood 	metal
// fam1 	darkwood 	metal
// fam2 	metal 	metal

// Let's assume that the various textures are given ID values in the MDL like so:
// Texture IDs ID 	Name
// 0 	lightwood
// 1 	darkwood
// 2 	metal
// Note:
// This ordering frequently matches the "VMTs Loaded" display of HLMV

// In the MDL data, this table relationship is broken down i32o a stream of bytes, with two-byte texture IDs replacing the literal names. Thus the final series of bytes that corresponds to these skin-family relationships would (as little-endian short values) be:

// 00 00   02 00
// 01 00   02 00
// 02 00   02 00

// The total number of bytes is numskinfamilies*numskins*2.
// Note:
// MDLs may often have a texture-replacement table which is larger than necessary, with additional columns which are never used.
// Todo:
// Discover how to accurately detect which columns are meaningful in too-large replacement tables.
