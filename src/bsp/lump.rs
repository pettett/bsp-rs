use std::{
    fs::File,
    io::{self, BufReader, Read, Seek},
    mem, slice,
};

pub trait Lump
where
    Self: Sized,
{
    fn max() -> usize;
    fn validate(lump: &Vec<Self>);
}

// https://developer.valvesoftware.com/wiki/BSP_(Source)
#[repr(C, packed)]
#[derive(Debug, Default, Copy, Clone)]
pub struct lump_t {
    fileofs: i32,    // offset into file (i8s)
    filelen: i32,    // length of lump (i8s)
    version: i32,    // lump format version
    fourCC: [u8; 4], // lump ident code
}
impl lump_t {
    pub fn decode<T: Clone>(&self, buffer: &mut BufReader<File>) -> Vec<T> {
        let item_size = mem::size_of::<T>();

        assert_eq!(self.filelen as usize % item_size, 0);

        let planes_count = self.filelen as usize / item_size;

        let mut planes = vec![unsafe { mem::zeroed() }; planes_count];

        unsafe {
            let header_slice = slice::from_raw_parts_mut(
                &mut planes[0] as *mut _ as *mut u8,
                planes_count * item_size,
            );
            buffer
                .seek(io::SeekFrom::Start(self.fileofs as u64))
                .unwrap();
            // `read_exact()` comes from `Read` impl for `&[u8]`
            buffer.read_exact(header_slice).unwrap();
        }

        planes
    }
}
