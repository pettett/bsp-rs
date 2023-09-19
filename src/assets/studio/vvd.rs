use std::{io::Seek, mem};

use crate::binaries::BinaryData;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
pub struct vertexFileHeader_t {
    id: i32,                     // MODEL_VERTEX_FILE_ID
    version: i32,                // MODEL_VERTEX_FILE_VERSION
    checksum: i32,               // same as studiohdr_t, ensures sync
    numLODs: u32,                // num of valid lods
    numLODVertexes: [i32; 8],    // num verts for desired root lod
    numFixups: i32,              // num of vertexFileFixup_t
    fixupTableStart: BinOffset,  // offset from base to fixup table
    vertexDataStart: BinOffset,  // offset from base to vertex block
    tangentDataStart: BinOffset, // offset from base to tangent block
}
use glam::{Vec2, Vec3, Vec4};

use super::BinOffset;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct ModelVertex {
    bone_weight: Vec3,
    bone_id: [u8; 3],
    num_bones: u8,
    pos: Vec3,
    norm: Vec3,
    uv: Vec2,
}

impl BinaryData for ModelVertex {}
impl BinaryData for vertexFileHeader_t {}

#[repr(C, packed)]
pub struct VVD {
    pub header: vertexFileHeader_t,
    pub verts: Box<[ModelVertex]>,
    pub tangents: Box<[Vec4]>,
}

impl BinaryData for VVD {
    fn read<R: std::io::Read + std::io::Seek>(
        buffer: &mut std::io::BufReader<R>,
        max_size: Option<usize>,
    ) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let s = buffer.stream_position()?;
        let header = vertexFileHeader_t::read(buffer, None)?;

        let mut pos = mem::size_of::<vertexFileHeader_t>() as i64;

        let total_verts = header.numLODVertexes[0] as usize;

        header.tangentDataStart.seek_start(buffer, 0, &mut pos)?;

        let tangents: Box<[Vec4]> = header
            .tangentDataStart
            .read_array_f(buffer, 0, &mut pos, total_verts)
            .unwrap();

        let verts: Box<[ModelVertex]> = header
            .vertexDataStart
            .read_array_f(buffer, 0, &mut pos, total_verts)
            .unwrap();

        Ok(Self {
            header,
            verts,
            tangents,
        })
    }
}
