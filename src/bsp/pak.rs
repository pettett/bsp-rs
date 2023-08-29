use std::{
    fs::File,
    io::{self, BufReader, Read},
};

use stream_unzip::ZipReader;

use crate::binaries::BinaryData;

use super::{consts::LumpType, Lump};

#[derive(Clone)]
pub struct PakEntry {
    pub filename: String,
    pub bytes: Vec<u8>,
}

pub struct BSPPak {
    pub entries: Vec<PakEntry>,
}

impl Lump for BSPPak {
    fn max() -> usize {
        1
    }

    fn lump_type() -> super::consts::LumpType {
        LumpType::PAKFILE
    }

    fn validate(lump: &Box<[Self]>) {}
}

impl BinaryData for BSPPak {
    fn read(buffer: &mut BufReader<File>, max_size: Option<usize>) -> io::Result<Self> {
        let mut pakfile_data = bytemuck::zeroed_slice_box(max_size.unwrap());
        buffer.read_exact(&mut pakfile_data)?;
        let mut zip_reader = ZipReader::default();

        zip_reader.update(pakfile_data.into());

        // Or read the whole file and deal with the entries
        // at the end.
        zip_reader.finish();

        let entries: Vec<PakEntry> = zip_reader
            .drain_entries()
            .iter()
            .map(|e| PakEntry {
                filename: e.header().filename.clone(),
                bytes: e.compressed_data().to_vec(),
            })
            .collect();

        Ok(Self { entries })
    }
}
