use std::{
    io::{self, BufReader, Cursor, Read, Seek},
    sync::OnceLock,
};

use stream_unzip::ZipReader;

use crate::{binaries::BinaryData, vmt::VMT, vtf::VTF};

use super::{consts::LumpType, Lump};

pub struct PakEntry {
    pub filename: String,
    pub bytes: Vec<u8>,
    pub vtf: OnceLock<Option<VTF>>,
    pub vmt: OnceLock<Option<VMT>>,
}

impl PakEntry {
    pub fn get_vtf(&self) -> Option<&VTF> {
        self.vtf
            .get_or_init(|| {
                if self.filename.ends_with(".vtf") {
                    let mut b = BufReader::new(Cursor::new(&self.bytes[..]));
                    VTF::read(&mut b, None).ok()
                } else {
                    None
                }
            })
            .as_ref()
    }

    pub fn get_vmt(&self) -> Option<&VMT> {
        self.vmt
            .get_or_init(|| {
                if self.filename.ends_with(".vmt") {
                    let mut b = BufReader::new(Cursor::new(&self.bytes[..]));
                    VMT::read(&mut b, None).ok()
                } else {
                    None
                }
            })
            .as_ref()
    }
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

    fn validate(_lump: &Box<[Self]>) {}
}

impl BinaryData for BSPPak {
    fn read<R: Read + Seek>(
        buffer: &mut BufReader<R>,
        max_size: Option<usize>,
    ) -> io::Result<Self> {
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
                vtf: OnceLock::new(),
                vmt: OnceLock::new(),
            })
            .collect();

        Ok(Self { entries })
    }
}
