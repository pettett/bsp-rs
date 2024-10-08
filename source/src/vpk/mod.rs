// // Valve Packfile. Only handles newest VPK version.

// https://developer.valvesoftware.com/wiki/VPK_(file_format)#Tree

// import ArrayBufferSlice from "../ArrayBufferSlice.js";
// import { assert, readString, leftPad, nullify } from "../util.js";
// import { DataFetcher, AbortedCallback } from "../DataFetcher.js";

// interface VPKFileEntryChunk {
//     packFileIdx: number;
//     chunkOffset: number;
//     chunkSize: number;
// }

// interface VPKFileEntry {
//     path: string;
//     crc: number;
//     chunks: VPKFileEntryChunk[];
//     metadataChunk: ArrayBufferSlice | null;
// }

// interface VPKDirectory {
//     entries: Map<string, VPKFileEntry>;
//     maxPackFile: number;
// }

pub mod pak;

use common::{vfile::VFileSystem, vpath::VPath};

use crate::binaries::BinaryData;
use ahash::{AHasher, RandomState};
use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, Cursor, Read, Seek},
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

#[cfg(target_arch = "x86_64")]
use std::fs::File;

use super::{
    studio::{vtx::VTX, vvd::VVD, MDL},
    vmt::VMT,
    vtf::VTF,
};

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VPKHeaderV1 {
    signature: u32,
    // = 0x55aa1234;
    version: u32, // = 2;

    // The size, in bytes, of the directory tree
    tree_size: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VPKHeaderV2 {
    // How many bytes of file content are stored in this VPK file (0 in CSGO)
    file_data_section_size: u32,

    // The size, in bytes, of the section containing MD5 checksums for external archive content
    archive_md5_section_size: u32,

    // The size, in bytes, of the section containing MD5 checksums for content in this file (should always be 48)
    other_md5_section_size: u32,

    // The size, in bytes, of the section containing the public key and signature. This is either 0 (CSGO & The Ship) or 296 (HL2, HL2:DM, HL2:EP1, HL2:EP2, HL2:LC, TF2, DOD:S & CS:S)
    signature_section_size: u32,
}

impl VPKHeaderV1 {
    pub fn pak_header() -> Self {
        Self {
            signature: 0,
            version: 0,
            tree_size: 0,
        }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct VPKDirectoryEntry {
    crc: u32,
    // A 32bit CRC of the file's data.
    preload_bytes: u16, // The number of bytes contained in the index file.

    // A zero based index of the archive this file's data is contained in.
    // If 0x7fff, the data follows the directory.
    archive_index: u16,

    // If ArchiveIndex is 0x7fff, the offset of the file data relative to the end of the directory (see the header for more details).
    // Otherwise, the offset of the data from the start of the specified archive.
    entry_offset: u32,

    // If zero, the entire file is stored in the preload data.
    // Otherwise, the number of bytes stored starting at EntryOffset.
    entry_length: u32,

    terminator: u16, //  = 0xffff
}

pub enum VPKFileType {
    VTF(VTF),
    VMT(VMT),
    MDL(MDL),
    VVD(VVD),
    VTX(VTX),
}

pub struct VPKFile {
    entry: VPKDirectoryEntry,
    preload: Option<Vec<u8>>,
    vtf: OnceLock<io::Result<Arc<VTF>>>,
    vmt: OnceLock<io::Result<Arc<VMT>>>,
    mdl: OnceLock<io::Result<Arc<MDL>>>,
    vvd: OnceLock<io::Result<Arc<VVD>>>,
    vtx: OnceLock<io::Result<Arc<VTX>>>,
}

impl VPKFile {
    pub fn load_vmt(&self, vpk: &VPKDirectory) -> io::Result<&Arc<VMT>> {
        self.load_file(vpk, |f| &f.vmt)
    }

    pub fn load_vtf(&self, vpk: &VPKDirectory) -> io::Result<&Arc<VTF>> {
        self.load_file(vpk, |f| &f.vtf)
    }

    pub fn load_mdl(&self, vpk: &VPKDirectory) -> io::Result<&Arc<MDL>> {
        self.load_file(vpk, |f| &f.mdl)
    }

    fn load_file<'a, T: BinaryData, F: FnOnce(&'a VPKFile) -> &'a OnceLock<io::Result<Arc<T>>>>(
        &'a self,
        vpk: &VPKDirectory,
        get_cell: F,
    ) -> io::Result<&'a Arc<T>> {
        match get_cell(self).get_or_init(|| vpk.load_file::<T>(self).map(|f| Arc::new(f))) {
            Ok(x) => Ok(x),
            Err(x) => {
                log::error!("Error loading data, {}", x.to_string());
                Err(io::Error::new(x.kind(), ""))
            }
        }
    }

    pub fn vtx(&self) -> &OnceLock<io::Result<Arc<VTX>>> {
        &self.vtx
    }

    pub fn vvd(&self) -> &OnceLock<io::Result<Arc<VVD>>> {
        &self.vvd
    }

    pub fn mdl(&self) -> &OnceLock<io::Result<Arc<MDL>>> {
        &self.mdl
    }

    pub fn vmt(&self) -> &OnceLock<io::Result<Arc<VMT>>> {
        &self.vmt
    }

    pub fn vtf(&self) -> &OnceLock<io::Result<Arc<VTF>>> {
        &self.vtf
    }

    pub fn len(&self) -> u32 {
        self.entry.entry_length
    }
    pub fn offset(&self) -> u32 {
        self.entry.entry_offset
    }
    pub fn archive(&self) -> u16 {
        self.entry.archive_index
    }

    pub fn preload(&self) -> Option<&Vec<u8>> {
        self.preload.as_ref()
    }
}

pub struct VPKDirectory {
    pub dir_path: PathBuf,
    header1: VPKHeaderV1,
    header2: Option<VPKHeaderV2>,
    max_pack_file: u16,
    /// Files map, mapped by extension, then directory, then filename
    pub files: HashMap<
        String,
        HashMap<String, HashMap<String, VPKFile, RandomState>, RandomState>,
        RandomState,
    >,
    pak_archives: Vec<PathBuf>,
    data: VFileSystem,
}

impl VPKDirectory {
    //pub fn get_file_names(&self) -> std::collections::hash_map::Keys<'_, String, VPKFile> {
    //    self.files.keys()
    //}

    pub fn insert(&mut self, ext: String, dir: String, filename: String, file: VPKFile) {
        let ext_files = self.files.entry(ext).or_default();
        let dir_files = ext_files.entry(dir).or_default();

        dir_files.insert(filename, file);
    }

    pub fn new(header1: VPKHeaderV1, header2: Option<VPKHeaderV2>) -> Self {
        Self {
            dir_path: Default::default(),
            header1,
            header2,
            max_pack_file: 0,
            files: Default::default(),
            data: Default::default(),
            pak_archives: Default::default(),
			
        }
    }
    pub fn load(file_load: VFileSystem, dir_path: PathBuf) -> io::Result<Self> {
        match file_load.clone().get(&dir_path) {
            Some(mut buffer) => Self::read(&mut buffer, file_load, dir_path),
            None => {
                #[cfg(target_arch = "x86_64")]
                {
                    let file = File::open(&dir_path)?;
                    let mut buffer = BufReader::new(file);
                    return Self::read(&mut buffer, file_load, dir_path);
                }
                panic!("{:?} not found", dir_path);
            }
        }
    }

    pub fn read<R: Read + Seek>(
        buffer: &mut BufReader<R>,
        file_load: VFileSystem,
        dir_path: PathBuf,
    ) -> io::Result<Self> {
        let header1 = VPKHeaderV1::read(buffer, None)?;
        let header2 = if header1.version == 2 {
            Some(VPKHeaderV2::read(buffer, None)?)
        } else {
            None
        };

        {
            let v = header1.version;
            println!("Loading VPK version {}", v);
        }

        let mut max_pack_file = 0;
        let mut files = HashMap::<_, HashMap<_, HashMap<_, _, _>, _>, _>::default();

        loop {
            let ext = read_string(buffer);
            if ext.len() == 0 {
                break;
            }

            let ext_files = files.entry(ext).or_default();

            loop {
                let dir = read_string(buffer);
                if dir.len() == 0 {
                    break;
                }

                let dir_files = ext_files.entry(dir).or_default();

                loop {
                    let filename = read_string(buffer);

                    if filename.len() == 0 {
                        break;
                    }

                    let entry = VPKDirectoryEntry::read(buffer, None).unwrap();
                    let terminator = entry.terminator;

                    assert_eq!(terminator, 0xffff);

                    if entry.archive_index != 0x7fff {
                        // 0x7fff means contained in this same file
                        max_pack_file = u16::max(entry.archive_index, max_pack_file);
                    }

                    // Read metadata.
                    let preload = if entry.preload_bytes != 0 {
                        let mut buf = vec![0; entry.preload_bytes as usize];
                        buffer.read_exact(&mut buf[..]).unwrap();
                        Some(buf)
                    } else {
                        None
                    };

                    dir_files.insert(
                        filename,
                        VPKFile {
                            entry,
                            preload,
                            vtf: OnceLock::new(),
                            vmt: OnceLock::new(),
                            mdl: OnceLock::new(),
                            vvd: OnceLock::new(),
                            vtx: OnceLock::new(),
                        },
                    );

                    //entries.set(path, { crc, path, chunks, metadataChunk });
                }
            }
        }

		let mut pak_archives = Vec::new();
        for i in 0..=max_pack_file {
            // replace dir with number
            let mut header_pak_path =  dir_path.to_path_buf();
            let dir_file = dir_path.file_name().unwrap().to_string_lossy();
            let name = dir_file.replace("_dir", &format!("_{i:0>3}"));

            header_pak_path.set_file_name(name);
            pak_archives.push(header_pak_path);
        }


        Ok(Self {
            dir_path,
            header1,
            header2,
            max_pack_file,
            files,
			pak_archives,
            data: file_load,
        })
    }

    pub fn load_vtf(&self, path: &dyn VPath) -> io::Result<&Arc<VTF>> {
        self.load_file_once(path, |f| &f.vtf)
    }
    /// Load material from global path (materials/x/y.vmt)
    pub fn load_vmt(&self, path: &dyn VPath) -> io::Result<&Arc<VMT>> {
        self.load_file_once(path, |f| &f.vmt)
    }
    pub fn load_mdl(&self, path: &dyn VPath) -> io::Result<&Arc<MDL>> {
        self.load_file_once(path, |f| &f.mdl)
    }
    pub fn load_vvd(&self, path: &dyn VPath) -> io::Result<&Arc<VVD>> {
        self.load_file_once(path, |f| &f.vvd)
    }
    pub fn load_vtx(&self, path: &dyn VPath) -> io::Result<&Arc<VTX>> {
        self.load_file_once(path, |f| &f.vtx)
    }

    pub fn file_data<'a>(&'a self, path: &dyn VPath) -> io::Result<&'a VPKFile> {
        let ext_files = self.files.get(path.ext()).ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Extension {} not present", path.ext()),
        ))?;

        let dir = ext_files.get(&path.dir()).ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Directory Prefix {} not present while loading {}",
                path.dir(),
                path.filename()
            ),
        ))?;

        let file_data = dir.get(path.filename()).ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("File {} not present", path.filename()),
        ))?;

        Ok(file_data)
    }

    pub fn load_file_once<
        'a,
        T: BinaryData,
        F: FnOnce(&'a VPKFile) -> &'a OnceLock<io::Result<Arc<T>>>,
    >(
        &'a self,
        path: &dyn VPath,
        get_cell: F,
    ) -> io::Result<&'a Arc<T>> {
        let file_data = self.file_data(path)?;

        file_data.load_file(self, get_cell)
    }

    fn load_file<F: BinaryData>(&self, file_data: &VPKFile) -> io::Result<F> {
        if let Some(preload) = &file_data.preload {
            // Load from preload data
            //TODO: delete preload data after

            let c = Cursor::new(preload);
            let mut buffer = BufReader::new(c);

            F::read(&mut buffer, Some(preload.len()))
        } else {
            // Attempt to load
            let index = file_data.entry.archive_index;

            // replace dir with number
            let mut header_pak_path = self.dir_path.to_path_buf();
            let dir_file = self.dir_path.file_name().unwrap().to_string_lossy();
            let name = dir_file.replace("_dir", &format!("_{index:0>3}"));

            #[cfg(target_arch = "wasm32")]
            if let Some(mut buffer) = self.data.get_str(&name) {
                // seek and load
                buffer.seek_relative(file_data.entry.entry_offset as i64)?;

                return F::read(&mut buffer, Some(file_data.entry.entry_length as usize));
            }

            #[cfg(target_arch = "x86_64")]
            {
                header_pak_path.set_file_name(name);

                // open file
                let file = File::open(header_pak_path)?;
                let mut buffer = BufReader::new(file);
                // seek and load
                buffer.seek_relative(file_data.entry.entry_offset as i64)?;

                return F::read(&mut buffer, Some(file_data.entry.entry_length as usize));
            }

            panic!("Could not find {} without desktop support", name)
        }
    }

    pub fn max_pack_file(&self) -> u16 {
        self.max_pack_file
    }
	
	pub fn pak_archive(&self, archive:u16) -> &Path {
		&self.pak_archives[archive as usize]
	}
}

pub fn read_string<R: Seek + Read>(buffer: &mut BufReader<R>) -> String {
    let mut string_buf = Vec::new();

    buffer.read_until(0, &mut string_buf).unwrap();
    string_buf.pop();

    unsafe { std::str::from_utf8_unchecked(&string_buf[..]) }.to_owned()
}

pub fn read_u32<R: Seek + Read>(buffer: &mut BufReader<R>) -> u32 {
    let mut string_buf = [0; 4];

    buffer.read_exact(&mut string_buf).unwrap();

    u32::from_le_bytes(string_buf)
}

pub fn read_u16<R: Seek + Read>(buffer: &mut BufReader<R>) -> u16 {
    let mut string_buf = [0; 2];

    buffer.read_exact(&mut string_buf).unwrap();

    u16::from_le_bytes(string_buf)
}

#[cfg(test)]
mod vpk_tests {
    use super::*;

    const PATH: &str =
        "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_textures_dir.vpk";

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_header() {
        let file = File::open(PATH).unwrap();
        let mut buffer = BufReader::new(file);

        let header = VPKHeaderV2::read(&mut buffer, None).unwrap();

        println!("{:?}", header);
    }

    #[test]
    fn test_dir() {
        let dir = VPKDirectory::load(Default::default(), PathBuf::from(PATH)).unwrap();

        for (ext, dirs) in &dir.files {
            println!("EXT: {}", ext);
            for (dir, files) in dirs {
                println!("DIR: {}", dir);
                for (file, _data) in files {
                    println!("FILE: {}", file);
                }
            }
        }
    }
}
