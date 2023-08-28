use std::{
    fs::File,
    io::{BufRead, BufReader, Seek},
};

use glam::{Vec3, Vec4};

use crate::bsp::consts::MAX_MAP_TEXDATA_STRING_DATA;

use super::{
    consts::{LumpType, MAX_MAP_TEXDATA, MAX_MAP_TEXINFO},
    lump::lump_t,
    Lump,
};

// Texinfo
//
// The texinfo lump (Lump 6) contains an array of texinfo_t structures:
//
// struct texinfo_t
// {
// 	float   textureVecs[2][4];    // [s/t][xyz offset]
// 	float   lightmapVecs[2][4];   // [s/t][xyz offset] - length is in units of texels/area
// 	int     flags;                // miptex flags overrides
// 	int     texdata;              // Pointer to texture name, size, etc.
// }
//
// Each texinfo is 72 bytes long.
//
// The first array of floats is in essence two vectors that represent how the texture is orientated and scaled when rendered on the world geometry. The two vectors, s and t, are the mapping of the left-to-right and down-to-up directions in the texture pixel coordinate space, onto the world. Each vector has an x, y, and z component, plus an offset which is the "shift" of the texture in that direction relative to the world. The length of the vectors represent the scaling of the texture in each direction.
//
// The 2D coordinates (u, v) of a texture pixel (or texel) are mapped to the world coordinates (x, y, z) of a point on a face by:
//
// u = tv0,0 * x + tv0,1 * y + tv0,2 * z + tv0,3
//
// v = tv1,0 * x + tv1,1 * y + tv1,2 * z + tv1,3
//
// (ie. The dot product of the vectors with the vertex plus the offset in that direction. Where tvA,B is textureVecs[A][B].
//
// Furthermore, after calculating (u, v), to convert them to texture coordinates which you would send to your graphics card, divide u and v by the width and height of the texture respectively.
//
// The lightmapVecs float array performs a similar mapping of the lightmap samples of the texture onto the world. It is the same formula but with lightmapVecs instead of textureVecs, and then subtracting the [0] and [1] values of LightmapTextureMinsInLuxels for u and v respectively. LightmapTextureMinsInLuxels is referenced in dface_t;
//
// The flags entry contains bitflags which are defined in bspflags.h:
// Name 	Value 	Notes
// SURF_LIGHT 	0x1 	value will hold the light strength
// SURF_SKY2D 	0x2 	don't draw, indicates we should skylight + draw 2d sky but not draw the 3D skybox
// SURF_SKY 	0x4 	don't draw, but add to skybox
// SURF_WARP 	0x8 	turbulent water warp
// SURF_TRANS 	0x10 	texture is translucent
// SURF_NOPORTAL 	0x20 	the surface can not have a portal placed on it
// SURF_TRIGGER 	0x40 	FIXME: This is an xbox hack to work around elimination of trigger surfaces, which breaks occluders
// SURF_NODRAW 	0x80 	don't bother referencing the texture
// SURF_HINT 	0x100 	make a primary bsp splitter
// SURF_SKIP 	0x200 	completely ignore, allowing non-closed brushes
// SURF_NOLIGHT 	0x400 	Don't calculate light
// SURF_BUMPLIGHT 	0x800 	calculate three lightmaps for the surface for bumpmapping
// SURF_NOSHADOWS 	0x1000 	Don't receive shadows
// SURF_NODECALS 	0x2000 	Don't receive decals
// SURF_NOCHOP 	0x4000 	Don't subdivide patches on this surface
// SURF_HITBOX 	0x8000 	surface is part of a hitbox
//
// The flags seem to be derived from the texture's .vmt file contents, and specify special properties of that texture.

#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct texinfo_t {
    /// [s/t][xyz offset]
    pub tex_s: Vec4,
    /// [s/t][xyz offset]
    pub tex_t: Vec4,
    pub lightmap_s: Vec4, // [s/t][xyz offset] - length is in units of texels/area
    pub lightmap_t: Vec4, // [s/t][xyz offset] - length is in units of texels/area
    pub flags: i32,       // miptex flags overrides
    pub tex_data: i32,    // Pointer to texture name, size, etc.
}
impl Lump for texinfo_t {
    fn max() -> usize {
        MAX_MAP_TEXINFO
    }
    fn lump_type() -> LumpType {
        LumpType::TEXINFO
    }
    fn validate(lump: &Box<[Self]>) {
        assert!(lump.len() < MAX_MAP_TEXINFO);

        println!("Validated texinfo lump!")
    }
}

///Texdata
///
///Finally the texdata entry is an index into the Texdata array, and specifies the actual texture.
///
///The index of a Texinfo (referenced from a face or brushside) may be given as -1; this indicates that no texture information is associated with this face. This occurs on compiling brush faces given the SKIP, CLIP, or INVISIBLE type textures in the editor.
///
///The texdata array (Lump 2) consists of the structures:
/// The reflectivity vector corresponds to the RGB components of the reflectivity of the texture, as derived from the material's .vtf file. This is probably used in radiosity (lighting) calculations of what light bounces from the texture's surface. The nameStringTableID is an index into the TexdataStringTable array (below). The other members relate to the texture's source image.
/// TexdataStringData and TexdataStringTable
///
#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct texdata_t {
    pub reflectivity: Vec3,     // RGB reflectivity
    pub nameStringTableID: i32, // index into TexdataStringTable
    pub width: i32,
    pub height: i32, // source image
    pub view_width: i32,
    pub view_height: i32,
}

impl Lump for texdata_t {
    fn max() -> usize {
        MAX_MAP_TEXDATA
    }
    fn lump_type() -> LumpType {
        LumpType::TEXDATA
    }
    fn validate(lump: &Box<[Self]>) {
        assert!(lump.len() < MAX_MAP_TEXINFO);

        println!("Validated dtexdata_t lump!")
    }
}
/// The TexdataStringTable (Lump 44) is an array of integers which are offsets into the TexdataStringData (lump 43). The TexdataStringData lump consists of concatenated null-terminated strings giving the texture name.

#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct texdatastringtable_t {
    pub index: i32,
}
impl texdatastringtable_t {
    pub fn get_filename(
        &self,
        buffer: &mut BufReader<File>,
        tex_data_string_data: &lump_t,
    ) -> String {
        let index = self.index;

        let seek_index = index + tex_data_string_data.fileofs;

        buffer
            .seek(std::io::SeekFrom::Start(seek_index as u64))
            .unwrap();

        let mut string_buf = Vec::new();

        buffer.read_until(0, &mut string_buf).unwrap();

        // remove trailing \0
        string_buf.pop();

        let mut str = unsafe { String::from_utf8_unchecked(string_buf) };
        str.make_ascii_lowercase();
        str
    }
}
impl Lump for texdatastringtable_t {
    fn max() -> usize {
        0
    }
    fn lump_type() -> LumpType {
        LumpType::TEXDATA_STRING_TABLE
    }
    fn validate(lump: &Box<[Self]>) {
        println!("Validated dtexdatastringdata_t lump!")
    }
}
//There can be a maximum of 12288 texinfos in a map (MAX_MAP_TEXINFO).
//There is a limit of 2048 texdatas in the array (MAX_MAP_TEXDATA) and up to 256000 bytes in the TexdataStringData data block (MAX_MAP_TEXDATA_STRING_DATA).
//Texture name strings are limited to 128 characters (TEXTURE_NAME_LENGTH).
