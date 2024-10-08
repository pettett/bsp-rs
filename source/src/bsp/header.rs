use crate::bsp::consts::HEADER_LUMPS;
use std::{
    fmt,
    io::{self, BufReader, Cursor, Read, Seek},
    mem,
    path::Path,
    slice,
};

#[cfg(target_arch = "x86_64")]
use std::fs::File;

use bytemuck::Zeroable;
use common::vfile::VFileSystem;

use super::{
    lump::{BSPLump, Lump},
    LumpType,
};

#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BSPHeader {
    pub ident: [u8; 4],                 // BSP file identifier
    pub version: i32,                   // BSP file version
    pub lumps: [BSPLump; HEADER_LUMPS], // lump directory array
    pub map_revision: i32,              // the map's revision (iteration, version) number
}

impl Default for BSPHeader {
    fn default() -> Self {
        Self {
            ident: Default::default(),
            version: Default::default(),
            lumps: [BSPLump::default(); 64],
            map_revision: Default::default(),
        }
    }
}

impl fmt::Debug for BSPHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version = self.version;
        let map_revision = self.map_revision;
        f.debug_struct("dheader_t")
            .field("ident", &self.ident)
            .field("version", &version)
            .field("mapRevision", &map_revision)
            .finish()
    }
}

impl BSPHeader {
    pub fn load_file<'a>(
        path: &Path,
        data: &'a VFileSystem,
    ) -> io::Result<(Self, BufReader<Cursor<&'a [u8]>>)> {
        let mut buffer = data.get(path).unwrap();

        Ok((Self::load_buf(&mut buffer)?, buffer))
    }

    #[cfg(target_arch = "x86_64")]
    pub fn load(path: &Path) -> io::Result<(Self, BufReader<File>)> {
        let file = File::open(path)?;
        let mut buffer = BufReader::new(file);

        Ok((Self::load_buf(&mut buffer)?, buffer))
    }

    pub fn load_buf<F: Read + Seek>(buffer: &mut BufReader<F>) -> io::Result<Self> {
        let mut header = Self::zeroed();

        let header_size = mem::size_of::<Self>();
        unsafe {
            let header_slice =
                slice::from_raw_parts_mut(&mut header as *mut _ as *mut u8, header_size);
            // `read_exact()` comes from `Read` impl for `&[u8]`
            buffer.read_exact(header_slice).unwrap();
        }
        //buffer.read_exact(&mut header.ident).unwrap();
        Ok(header)
    }
    pub fn get_lump_header(&self, lump: LumpType) -> &BSPLump {
        &self.lumps[lump as usize]
    }
    pub fn get_lump<T: Lump + bytemuck::Zeroable>(
        &self,
        buffer: &mut BufReader<impl Seek + Read>,
    ) -> Box<[T]> {
        self.get_lump_header(T::lump_type()).decode(buffer).unwrap()
    }
    pub fn validate(&self) {
        // Check the magic number
        // This way around means little endian, PSBV is big endian
        let text = "VBSP";
        let magic_number: [u8; 4] = text
            .chars()
            .map(|c| c as u8)
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();

        assert_eq!(self.ident, magic_number);

        //let mut i8s = [0, 0, 0, 0];
        //buffer.read_exact(&mut i8s).unwrap();
        //header.version = i32::from_le_i8s(i8s);

        //println!("{self:?}");
        //for i in 0..self.lumps.len() {
        //    println!("{:?} {:?}", LumpType::from_usize(i), self.lumps[i]);
        //}
    }
}
