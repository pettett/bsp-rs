use std::mem;

use crate::binaries::BinaryData;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
pub struct vertexFileHeader_t {
    id: i32,                  // MODEL_VERTEX_FILE_ID
    version: i32,             // MODEL_VERTEX_FILE_VERSION
    checksum: i32,            // same as studiohdr_t, ensures sync
    numLODs: i32,             // num of valid lods
    numLODVertexes: [i32; 8], // num verts for desired root lod
    numFixups: i32,           // num of vertexFileFixup_t
    fixupTableStart: i32,     // offset from base to fixup table
    vertexDataStart: i32,     // offset from base to vertex block
    tangentDataStart: i32,    // offset from base to tangent block
}

impl BinaryData for vertexFileHeader_t {}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
pub struct VVD {
    pub header: vertexFileHeader_t,
}

impl BinaryData for VVD {
    fn read<R: std::io::Read + std::io::Seek>(
        buffer: &mut std::io::BufReader<R>,
        max_size: Option<usize>,
    ) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let header = vertexFileHeader_t::read(buffer, None)?;

        let mut pos = mem::size_of::<vertexFileHeader_t>() as i64;

        Ok(Self { header })
    }
}
