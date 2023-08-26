use crate::bsp::consts::HEADER_LUMPS;
use std::{
    fmt,
    fs::File,
    io::{self, BufReader, Read},
    mem, slice,
};

use crate::bsp::consts::LumpType;
use num_traits::FromPrimitive;

use super::{lump::lump_t, Lump};

#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct dheader_t {
    pub ident: [u8; 4],                // BSP file identifier
    pub version: i32,                  // BSP file version
    pub lumps: [lump_t; HEADER_LUMPS], // lump directory array
    pub mapRevision: i32,              // the map's revision (iteration, version) number
}

impl Default for dheader_t {
    fn default() -> Self {
        Self {
            ident: Default::default(),
            version: Default::default(),
            lumps: [lump_t::default(); 64],
            mapRevision: Default::default(),
        }
    }
}

impl fmt::Debug for dheader_t {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version = self.version;
        let mapRevision = self.mapRevision;
        f.debug_struct("dheader_t")
            .field("ident", &self.ident)
            .field("version", &version)
            .field("mapRevision", &mapRevision)
            .finish()
    }
}

impl dheader_t {
    pub fn load(path: &str) -> io::Result<(Self, BufReader<File>)> {
        let file = File::open(path)?;
        let mut buffer = BufReader::new(file);

        let mut header: dheader_t = unsafe { mem::zeroed() };

        let header_size = mem::size_of::<dheader_t>();
        unsafe {
            let header_slice =
                slice::from_raw_parts_mut(&mut header as *mut _ as *mut u8, header_size);
            // `read_exact()` comes from `Read` impl for `&[u8]`
            buffer.read_exact(header_slice).unwrap();
        }
        //buffer.read_exact(&mut header.ident).unwrap();
        Ok((header, buffer))
    }

    pub fn get_lump_header(&self, lump: LumpType) -> &lump_t {
        &self.lumps[lump as usize]
    }
    pub fn get_lump<T: Lump>(&self, buffer: &mut BufReader<File>) -> Box<[T]> {
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

        println!("{self:?}");
        for i in 0..self.lumps.len() {
            println!("{:?} {:?}", LumpType::from_usize(i), self.lumps[i]);
        }
    }
}
