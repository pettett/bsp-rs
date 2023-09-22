use std::{
    io::{self, BufReader, Read, Seek},
    mem, slice,
};

pub trait BinaryData {
    fn read<R: Read + Seek>(buffer: &mut BufReader<R>, _max_size: Option<usize>) -> io::Result<Self>
    where
        Self: Sized,
    {
        let mut header = unsafe { mem::zeroed() };

        let header_size = mem::size_of::<Self>();
        unsafe {
            let header_slice =
                slice::from_raw_parts_mut(&mut header as *mut _ as *mut u8, header_size);
            // `read_exact()` comes from `Read` impl for `&[u8]`
            buffer.read_exact(header_slice)?;
        }
        Ok(header)
    }
}

impl<T: bytemuck::Pod> BinaryData for T {}
