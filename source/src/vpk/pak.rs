use std::{
    io::{self, BufReader, Read, Seek},
    sync::OnceLock,
};

use stream_unzip::ZipReader;

use crate::binaries::BinaryData;
use crate::bsp::consts::LumpType;
use crate::bsp::Lump;
use crate::vpk::{VPKDirectory, VPKDirectoryEntry, VPKFile};

use super::VPKHeaderV1;

impl Lump for VPKDirectory {
    fn max() -> usize {
        1
    }

    fn lump_type() -> LumpType {
        LumpType::PakFile
    }
}

impl BinaryData for VPKDirectory {
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

        let mut vpk = VPKDirectory::new(VPKHeaderV1::pak_header(), None);

        for e in zip_reader.drain_entries() {
            let Some(filename_sep) = e.header().filename.rfind('/') else {
                println!("Cannot work with {:?}", e.header().filename);
                continue;
            };
            let ext_sep = e.header().filename.find('.').unwrap();

            let ext = e.header().filename[ext_sep + 1..].to_owned();
            let dir = e.header().filename[..filename_sep].to_owned();
            let filename = e.header().filename[filename_sep + 1..ext_sep].to_owned();

            vpk.insert(
                ext,
                dir,
                filename,
                VPKFile {
                    entry: VPKDirectoryEntry {
                        crc: e.header().crc32,
                        preload_bytes: e.header().uncompressed_size as u16,
                        archive_index: 0,
                        entry_offset: 0,
                        entry_length: 0,
                        terminator: 0xffff,
                    },
                    preload: Some(e.compressed_data().to_vec()),
                    vtf: OnceLock::new(),
                    vmt: OnceLock::new(),
                    mdl: OnceLock::new(),
                    vvd: OnceLock::new(),
                    vtx: OnceLock::new(),
                },
            )
        }

        Ok(vpk)
    }
}
