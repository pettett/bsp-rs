use std::{
    fmt::Debug,
    io::{self, BufReader, Read, Seek},
    mem, slice,
};

use glam::{Vec2, Vec3, Vec4};

pub trait BinaryData {
    fn read<R: Read + Seek>(buffer: &mut BufReader<R>, _max_size: Option<usize>) -> io::Result<Self>
    where
        Self: Sized,
    {
        let mut data = unsafe { mem::zeroed() };

        let size = mem::size_of::<Self>();
        unsafe {
            let slice = slice::from_raw_parts_mut(&mut data as *mut _ as *mut u8, size);
            // `read_exact()` comes from `Read` impl for `&[u8]`
            buffer.read_exact(slice)?;
        }
        Ok(data)
    }

    fn read_array<R: Read + Seek>(
        buffer: &mut BufReader<R>,
        count: usize,
        _max_size: Option<usize>,
    ) -> io::Result<Box<[Self]>>
    where
        Self: Sized + bytemuck::Zeroable + bytemuck::Pod + Debug,
    {
        let mut header = bytemuck::zeroed_slice_box(count);

        unsafe {
            let size = count * mem::size_of::<Self>();
            let slice = slice::from_raw_parts_mut(&mut header[0] as *mut _ as *mut u8, size);
            // `read_exact()` comes from `Read` impl for `&[u8]`
            buffer.read_exact(slice)?;
        }

        Ok(header)
    }
}

impl BinaryData for u16 {}
impl BinaryData for u32 {}

impl BinaryData for Vec2 {}
impl BinaryData for Vec3 {}
impl BinaryData for Vec4 {}
