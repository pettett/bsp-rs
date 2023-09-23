use std::{
    fmt::Debug,
    fs::File,
    io::{self, BufReader, Read, Seek},
    mem, slice,
};

use crate::binaries::BinaryData;

use super::consts::LumpType;

pub trait Lump
where
    Self: Sized,
{
    fn max() -> usize;
    fn lump_type() -> LumpType;
}

// https://developer.valvesoftware.com/wiki/BSP_(Source)
#[repr(C, packed)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BSPLump {
    pub file_ofs: i32,    // offset into file (i8s)
    pub file_len: i32,    // length of lump (i8s)
    pub version: i32,     // lump format version
    pub four_cc: [u8; 4], // lump ident code
}
impl BSPLump {
    pub fn decode<T: bytemuck::Zeroable>(
        &self,
        buffer: &mut BufReader<File>,
    ) -> io::Result<Box<[T]>> {
        let item_size = mem::size_of::<T>();

        assert_eq!(
            self.file_len as usize % item_size,
            0,
            "Structure given does not fit nicely into lump data"
        );

        let len = self.file_len as usize / item_size;

        let mut table = bytemuck::zeroed_slice_box(len);

        if len > 0 {
            unsafe {
                let header_slice =
                    slice::from_raw_parts_mut(&mut table[0] as *mut _ as *mut u8, len * item_size);
                buffer.seek(io::SeekFrom::Start(self.file_ofs as u64))?;
                // `read_exact()` comes from `Read` impl for `&[u8]`
                buffer.read_exact(header_slice)?;
            }
        }

        Ok(table)
    }

    pub fn read_binary<T: BinaryData>(&self, buffer: &mut BufReader<File>) -> io::Result<T> {
        buffer.seek(std::io::SeekFrom::Start(self.file_ofs as u64))?;
        Ok(T::read(buffer, Some(self.file_len as usize))?)
    }

    pub fn read_bytes(&self, buffer: &mut BufReader<File>) -> io::Result<Box<[u8]>> {
        buffer.seek(std::io::SeekFrom::Start(self.file_ofs as u64))?;
        let mut bytes = bytemuck::zeroed_slice_box(self.file_len as usize);
        buffer.read_exact(&mut bytes)?;
        Ok(bytes)
    }
}
