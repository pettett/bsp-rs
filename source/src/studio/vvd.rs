use crate::binaries::{BinOffset, BinaryData};
use glam::{Vec2, Vec3, Vec4};
use std::{io::Seek, mem};

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
pub struct VertexFileHeader {
    id: i32,                       // MODEL_VERTEX_FILE_ID
    version: i32,                  // MODEL_VERTEX_FILE_VERSION
    checksum: i32,                 // same as studiohdr_t, ensures sync
    num_lods: u32,                 // num of valid lods
    num_lod_vertexes: [u32; 8],    // num verts for desired root lod
    num_fixups: u32,               // num of vertexFileFixup_t
    fixup_table_start: BinOffset,  // offset from base to fixup table
    vertex_data_start: BinOffset,  // offset from base to vertex block
    tangent_data_start: BinOffset, // offset from base to tangent block
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct ModelVertex {
    pub bone_weight: Vec3,
    pub bone_id: [u8; 3],
    pub num_bones: u8,
    pub pos: Vec3,
    pub norm: Vec3,
    pub uv: Vec2,
}

#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
pub struct Fixup {
    pub lod: i32,
    pub dst: i32,
    pub src: i32,
    pub count: i32,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
struct VVDFixup {
    pub lod: i32,
    pub src: i32,
    pub count: i32,
}

pub struct VVD {
    pub header: VertexFileHeader,
    pub verts: Box<[ModelVertex]>,
    pub tangents: Box<[Vec4]>,
    pub fixups: Box<[Fixup]>,
}

impl BinaryData for VVD {
    fn read<R: std::io::Read + std::io::Seek>(
        buffer: &mut std::io::BufReader<R>,
        _max_size: Option<usize>,
    ) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let _s = buffer.stream_position()?;
        let header = VertexFileHeader::read(buffer, None)?;

        let mut pos = mem::size_of::<VertexFileHeader>() as i64;

        let total_verts = header.num_lod_vertexes[0] as usize;

        let v = header.vertex_data_start.index;
        let v1 = v + header.num_lod_vertexes[0] * 0x30;
        let t = header.tangent_data_start.index;
        assert_eq!(v1, t);

        let verts: Box<[ModelVertex]> =
            header
                .vertex_data_start
                .read_array_f(buffer, 0, &mut pos, total_verts)?;

        let tangents: Box<[Vec4]> =
            header
                .tangent_data_start
                .read_array_f(buffer, 0, &mut pos, total_verts)?;

        let vvd_fixups: Box<[VVDFixup]> = header.fixup_table_start.read_array_f(
            buffer,
            0,
            &mut pos,
            header.num_fixups as usize,
        )?;

        let mut fixups: Box<[Fixup]> = bytemuck::zeroed_slice_box(vvd_fixups.len());
        let mut dst = 0;

        for i in 0..fixups.len() {
            fixups[i].dst = dst;
            fixups[i].count = vvd_fixups[i].count;
            fixups[i].lod = vvd_fixups[i].lod;
            fixups[i].src = vvd_fixups[i].src;
            dst += fixups[i].count;
        }

        // if header.num_fixups > 0 {
        //     println!("Fixups: {:?}", fixups);
        // }

        Ok(Self {
            header,
            verts,
            tangents,
            fixups,
        })
    }
}
