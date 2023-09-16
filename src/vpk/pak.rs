use std::collections::HashMap;
use std::{
    io::{self, BufReader, Cursor, Read, Seek},
    sync::OnceLock,
};

use bevy_ecs::system::Resource;
use stream_unzip::ZipReader;

use crate::bsp::consts::LumpType;
use crate::bsp::Lump;
use crate::vpk::{VPKDirectory, VPKDirectoryEntry, VPKFile, VPKHeader_v2};
use crate::{binaries::BinaryData, bsp, vmt::VMT, vtf::VTF};

impl Lump for VPKDirectory {
    fn max() -> usize {
        1
    }

    fn lump_type() -> LumpType {
        LumpType::PAKFILE
    }

    fn validate(_lump: &Box<[Self]>) {}
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

        let mut vpk = VPKDirectory::new(VPKHeader_v2::pak_header());

        for e in zip_reader.drain_entries() {
            let filename_sep = e.header().filename.rfind('/').unwrap();
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
                        CRC: e.header().crc32,
                        PreloadBytes: e.header().uncompressed_size as u16,
                        ArchiveIndex: 0,
                        EntryOffset: 0,
                        EntryLength: 0,
                        Terminator: 0xffff,
                    },
                    preload: Some(e.compressed_data().to_vec()),
                    vtf: OnceLock::new(),
                    vmt: OnceLock::new(),
                },
            )
        }

        Ok(vpk)
    }
}
