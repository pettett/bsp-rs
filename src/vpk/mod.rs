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

pub mod file;
pub mod gui;
pub mod pak;

use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader, Cursor, Read},
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use crate::{binaries::BinaryData, util::v_path::VPath, vmt::VMT, vtf::VTF};

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

impl BinaryData for VPKHeaderV1 {}
impl BinaryData for VPKHeaderV2 {}
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
    CRC: u32,
    // A 32bit CRC of the file's data.
    PreloadBytes: u16, // The number of bytes contained in the index file.

    // A zero based index of the archive this file's data is contained in.
    // If 0x7fff, the data follows the directory.
    ArchiveIndex: u16,

    // If ArchiveIndex is 0x7fff, the offset of the file data relative to the end of the directory (see the header for more details).
    // Otherwise, the offset of the data from the start of the specified archive.
    EntryOffset: u32,

    // If zero, the entire file is stored in the preload data.
    // Otherwise, the number of bytes stored starting at EntryOffset.
    EntryLength: u32,

    Terminator: u16, //  = 0xffff
}

impl BinaryData for VPKDirectoryEntry {}

#[derive(Debug)]
pub struct VPKFile {
    entry: VPKDirectoryEntry,
    preload: Option<Vec<u8>>,
    vtf: OnceLock<Option<Arc<VTF>>>,
    vmt: OnceLock<Option<Arc<VMT>>>,
}

impl VPKFile {
    pub fn load_vmt(&self, vpk: &VPKDirectory) -> io::Result<Option<&Arc<VMT>>> {
        self.load_file(vpk, |f| &f.vmt)
    }

    pub fn load_vtf(&self, vpk: &VPKDirectory) -> io::Result<Option<&Arc<VTF>>> {
        self.load_file(vpk, |f| &f.vtf)
    }

    fn load_file<'a, T: BinaryData, F: FnOnce(&'a VPKFile) -> &'a OnceLock<Option<Arc<T>>>>(
        &'a self,
        vpk: &VPKDirectory,
        get_cell: F,
    ) -> io::Result<Option<&'a Arc<T>>> {
        Ok(get_cell(self)
            .get_or_init(|| vpk.load_file::<T>(self).map(|f| Arc::new(f)))
            .as_ref())
    }
}

#[derive(Debug)]
pub enum VPKDirectoryTree {
    Leaf(String),
    Node(HashMap<String, VPKDirectoryTree>),
}

pub struct VPKDirectory {
    dir_path: PathBuf,
    header1: VPKHeaderV1,
    header2: Option<VPKHeaderV2>,
    max_pack_file: u16,
    root: VPKDirectoryTree,
    /// Files map, mapped by extension, then directory, then filename
    pub files: HashMap<String, HashMap<String, HashMap<String, VPKFile>>>,
}

impl VPKDirectoryTree {
    pub fn add_entry(&mut self, _prefix: &str, dir: &str, _ext: &str) {
        self.add_entry_inner(dir, dir);
    }

    fn add_entry_inner(&mut self, parsed_entry: &str, entry: &str) {
        if parsed_entry.len() == 0 {
            //most likely added a folder "etc/", don't add a "" leaf
            return;
        }
        match self {
            VPKDirectoryTree::Leaf(..) => {}
            VPKDirectoryTree::Node(tree) => match parsed_entry.find('/') {
                Some(pos) => match tree.get_mut(&parsed_entry[..pos]) {
                    Some(subtree) => subtree.add_entry_inner(&parsed_entry[pos + 1..], entry),
                    None => {
                        let mut dir = VPKDirectoryTree::Node(HashMap::new());

                        dir.add_entry_inner(&parsed_entry[pos + 1..], entry);

                        tree.insert(parsed_entry[..pos].to_owned(), dir);
                    }
                },
                None => {
                    tree.insert(parsed_entry.to_owned(), Self::Leaf(entry.to_owned()));
                }
            },
        };
    }
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

    pub fn get_root(&self) -> &VPKDirectoryTree {
        &self.root
    }
    pub fn new(header1: VPKHeaderV1, header2: Option<VPKHeaderV2>) -> Self {
        Self {
            dir_path: Default::default(),
            header1,
            header2,
            max_pack_file: 0,
            root: VPKDirectoryTree::Node(HashMap::new()),
            files: Default::default(),
        }
    }
    pub fn load(dir_path: PathBuf) -> io::Result<Self> {
        let file = File::open(&dir_path)?;
        let mut buffer = BufReader::new(file);

        let header1 = VPKHeaderV1::read(&mut buffer, None)?;
        let header2 = if header1.version == 2 {
            Some(VPKHeaderV2::read(&mut buffer, None)?)
        } else {
            None
        };

        {
            let v = header1.version;
            println!("Loading {:?} version {}", dir_path, v);
        }

        let mut root = VPKDirectoryTree::Node(HashMap::new());
        let mut max_pack_file = 0;
        let mut files = HashMap::<String, HashMap<String, HashMap<String, VPKFile>>>::new();

        loop {
            let ext = read_string(&mut buffer);
            if ext.len() == 0 {
                break;
            }
            loop {
                let dir = read_string(&mut buffer);
                if dir.len() == 0 {
                    break;
                }
                loop {
                    let filename = read_string(&mut buffer);

                    if filename.len() == 0 {
                        break;
                    }

                    let entry = VPKDirectoryEntry::read(&mut buffer, None).unwrap();
                    let terminator = entry.Terminator;

                    assert_eq!(terminator, 0xffff);

                    if entry.ArchiveIndex != 0x7fff {
                        // 0x7fff means contained in this same file
                        max_pack_file = u16::max(entry.ArchiveIndex, max_pack_file);
                    }

                    // Read metadata.
                    let preload = if entry.PreloadBytes != 0 {
                        let mut buf = vec![0; entry.PreloadBytes as usize];
                        buffer.read_exact(&mut buf[..]).unwrap();
                        Some(buf)
                    } else {
                        None
                    };

                    root.add_entry(&dir, &filename, &ext);

                    let ext_files = files.entry(ext.clone()).or_default();
                    let dir_files = ext_files.entry(dir.clone()).or_default();

                    dir_files.insert(
                        filename,
                        VPKFile {
                            entry,
                            preload,
                            vtf: OnceLock::new(),
                            vmt: OnceLock::new(),
                        },
                    );

                    //entries.set(path, { crc, path, chunks, metadataChunk });
                }
            }
        }

        Ok(Self {
            dir_path,
            header1,
            header2,
            max_pack_file,
            root,
            files,
        })
    }

    pub fn load_vtf(&self, path: &dyn VPath) -> io::Result<Option<&Arc<VTF>>> {
        self.load_file_once(path, |f| &f.vtf)
    }

    /// Load material from global path (materials/x/y.vmt)
    pub fn load_vmt(&self, path: &dyn VPath) -> io::Result<Option<&Arc<VMT>>> {
        self.load_file_once(path, |f| &f.vmt)
    }

    pub fn load_file_once<
        'a,
        T: BinaryData,
        F: FnOnce(&'a VPKFile) -> &'a OnceLock<Option<Arc<T>>>,
    >(
        &'a self,
        path: &dyn VPath,
        get_cell: F,
    ) -> io::Result<Option<&'a Arc<T>>> {
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

        file_data.load_file(self, get_cell)
    }

    fn load_file<F: BinaryData>(&self, file_data: &VPKFile) -> Option<F> {
        if let Some(preload) = &file_data.preload {
            // Load from preload data
            //TODO: delete preload data after

            let c = Cursor::new(preload);
            let mut buffer = BufReader::new(c);

            F::read(&mut buffer, Some(preload.len())).ok()
        } else {
            // Attempt to load
            let index = file_data.entry.ArchiveIndex;
            // replace dir with number
            let mut header_pak_path = self.dir_path.to_path_buf();
            let dir_file = self.dir_path.file_name().unwrap().to_string_lossy();
            header_pak_path.set_file_name(dir_file.replace("_dir", &format!("_{index:0>3}")));

            // open file
            let file = File::open(header_pak_path).unwrap();
            let mut buffer = BufReader::new(file);
            // seek and load
            if buffer
                .seek_relative(file_data.entry.EntryOffset as i64)
                .is_ok()
            {
                F::read(&mut buffer, Some(file_data.entry.EntryLength as usize)).ok()
            } else {
                None
            }
        }
    }
}

pub fn read_string(buffer: &mut BufReader<File>) -> String {
    let mut string_buf = Vec::new();

    buffer.read_until(0, &mut string_buf).unwrap();
    string_buf.pop();

    unsafe { std::str::from_utf8_unchecked(&string_buf[..]) }.to_owned()
}

pub fn read_u32(buffer: &mut BufReader<File>) -> u32 {
    let mut string_buf = [0; 4];

    buffer.read_exact(&mut string_buf).unwrap();

    u32::from_le_bytes(string_buf)
}

pub fn read_u16(buffer: &mut BufReader<File>) -> u16 {
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
    fn test_header() {
        let file = File::open(PATH).unwrap();
        let mut buffer = BufReader::new(file);

        let header = VPKHeaderV2::read(&mut buffer, None).unwrap();

        println!("{:?}", header);
    }

    #[test]
    fn test_dir() {
        let dir = VPKDirectory::load(PathBuf::from(PATH)).unwrap();

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

    #[test]
    fn test_tree() {
        let root = VPKDirectoryTree::Node(HashMap::new());

        //root.add_entry("test/file/please/ignore.txt");
        //root.add_entry("test/file/please/receive.txt");
        //root.add_entry("test/folder/");
        //root.add_entry("test/folder/please/receive.txt");

        println!("{:?}", root);
    }
}

// export function parseVPKDirectory(buffer: ArrayBufferSlice): VPKDirectory {
//     const view = buffer.createDataView();
//     assert(view.getUint32(0x00, true) === 0x55AA1234);
//     const version = view.getUint32(0x04, true);
//     const directorySize = view.getUint32(0x08, true);

//     let idx: number;
//     if (version === 0x01) {
//         idx = 0x0C;
//     } else if (version === 0x02) {
//         const embeddedChunkSize = view.getUint32(0x0C, true);
//         assert(embeddedChunkSize === 0);
//         const chunkHashesSize = view.getUint32(0x10, true);
//         const selfHashesSize = view.getUint32(0x14, true);
//         const signatureSize = view.getUint32(0x18, true);
//         idx = 0x1C;
//     } else {
//         throw "whoops";
//     }

//     // Parse directory.

//     let maxPackFile = 0;

//     const entries = new Map<string, VPKFileEntry>();
//     while (true) {
//         const ext = readString(buffer, idx);
//         idx += ext.length + 1;
//         if (ext.length === 0)
//             break;

//         while (true) {
//             const dir = readString(buffer, idx);
//             idx += dir.length + 1;
//             if (dir.length === 0)
//                 break;

//             while (true) {
//                 const filename = readString(buffer, idx);
//                 idx += filename.length + 1;
//                 if (filename.length === 0)
//                     break;

//                 const dirPrefix = (dir === '' || dir === ' ') ? '' : `${dir}/`;

//                 const path = `${dirPrefix}${filename}.${ext}`;
//                 const crc = view.getUint32(idx, true);
//                 idx += 0x04;
//                 const metadataSize = view.getUint16(idx, true);
//                 idx += 0x02;

//                 // Parse file chunks.
//                 const chunks: VPKFileEntryChunk[] = [];
//                 while (true) {
//                     const packFileIdx = view.getUint16(idx + 0x00, true);
//                     idx += 0x02;
//                     if (packFileIdx === 0xFFFF)
//                         break;

//                     if (packFileIdx !== 0x07FF)
//                         maxPackFile = Math.max(maxPackFile, packFileIdx);

//                     const chunkOffset = view.getUint32(idx + 0x00, true);
//                     const chunkSize = view.getUint32(idx + 0x04, true);
//                     idx += 0x08;

//                     if (chunkSize === 0)
//                         continue;

//                     chunks.push({ packFileIdx, chunkOffset, chunkSize });
//                 }

//                 // Read metadata.
//                 const metadataChunk = metadataSize !== 0 ? buffer.subarray(idx, metadataSize) : null;
//                 idx += metadataSize;

//                 entries.set(path, { crc, path, chunks, metadataChunk });
//             }
//         }
//     }

//     return { entries, maxPackFile };
// }

// export class VPKMount {
//     private fileDataPromise = new Map<string, Promise<ArrayBufferSlice>>();

//     constructor(private basePath: string, private dir: VPKDirectory) {
//     }

//     private fetchChunk(dataFetcher: DataFetcher, chunk: VPKFileEntryChunk, abortedCallback: AbortedCallback, debugName: string): Promise<ArrayBufferSlice> {
//         const packFileIdx = chunk.packFileIdx, rangeStart = chunk.chunkOffset, rangeSize = chunk.chunkSize;
//         return dataFetcher.fetchData(`${this.basePath}_${leftPad('' + packFileIdx, 3, '0')}.vpk`, { debugName, rangeStart, rangeSize, abortedCallback });
//     }

//     public findEntry(path: string): VPKFileEntry | null {
//         return nullify(this.dir.entries.get(path));
//     }

//     private async fetchFileDataInternal(dataFetcher: DataFetcher, entry: VPKFileEntry, abortedCallback: AbortedCallback): Promise<ArrayBufferSlice> {
//         const promises = [];
//         let size = 0;

//         const metadataSize = entry.metadataChunk !== null ? entry.metadataChunk.byteLength : 0;
//         size += metadataSize;

//         for (let i = 0; i < entry.chunks.length; i++) {
//             const chunk = entry.chunks[i];
//             promises.push(this.fetchChunk(dataFetcher, chunk, abortedCallback, entry.path));
//             size += chunk.chunkSize;
//         }

//         if (promises.length === 0) {
//             assert(entry.metadataChunk !== null);
//             return entry.metadataChunk;
//         }

//         const chunks = await Promise.all(promises);
//         if (chunks.length === 1 && entry.metadataChunk === null)
//             return chunks[0];

//         const buf = new Uint8Array(metadataSize + size);

//         let offs = 0;

//         // Metadata comes first.
//         if (entry.metadataChunk !== null) {
//             buf.set(entry.metadataChunk.createTypedArray(Uint8Array), offs);
//             offs += entry.metadataChunk.byteLength;
//         }

//         for (let i = 0; i < chunks.length; i++) {
//             buf.set(chunks[i].createTypedArray(Uint8Array), offs);
//             offs += chunks[i].byteLength;
//         }

//         return new ArrayBufferSlice(buf.buffer);
//     }

//     public fetchFileData(dataFetcher: DataFetcher, entry: VPKFileEntry): Promise<ArrayBufferSlice> {
//         if (!this.fileDataPromise.has(entry.path)) {
//             this.fileDataPromise.set(entry.path, this.fetchFileDataInternal(dataFetcher, entry, () => {
//                 this.fileDataPromise.delete(entry.path);
//             }));
//         }
//         return this.fileDataPromise.get(entry.path)!;
//     }
// }

// export async function createVPKMount(dataFetcher: DataFetcher, basePath: string) {
//     const dir = parseVPKDirectory(await dataFetcher.fetchData(`${basePath}_dir.vpk`));
//     return new VPKMount(basePath, dir);
// }
