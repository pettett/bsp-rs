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

use std::{
    cell::{OnceCell, RefCell},
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader, Read},
    mem,
    path::{Path, PathBuf},
    slice,
};

use bytemuck::Zeroable;

use crate::{binaries::BinaryData, vtf::VTF};

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct VPKHeader_v2 {
    Signature: u32, // = 0x55aa1234;
    Version: u32,   // = 2;

    // The size, in bytes, of the directory tree
    TreeSize: u32,

    // How many bytes of file content are stored in this VPK file (0 in CSGO)
    FileDataSectionSize: u32,

    // The size, in bytes, of the section containing MD5 checksums for external archive content
    ArchiveMD5SectionSize: u32,

    // The size, in bytes, of the section containing MD5 checksums for content in this file (should always be 48)
    OtherMD5SectionSize: u32,

    // The size, in bytes, of the section containing the public key and signature. This is either 0 (CSGO & The Ship) or 296 (HL2, HL2:DM, HL2:EP1, HL2:EP2, HL2:LC, TF2, DOD:S & CS:S)
    SignatureSectionSize: u32,
}

impl BinaryData for VPKHeader_v2 {}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct VPKDirectoryEntry {
    CRC: u32,          // A 32bit CRC of the file's data.
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
    vtf: OnceCell<VTF>,
}
#[derive(Debug)]
pub enum VPKDirectoryTree {
    Leaf(String),
    Node(HashMap<String, VPKDirectoryTree>),
}

pub struct VPKDirectory {
    dir_path: PathBuf,
    header: VPKHeader_v2,
    maxPackFile: u16,
    root: VPKDirectoryTree,
    files: HashMap<String, VPKFile>,
}

impl VPKDirectoryTree {
    pub fn add_entry(&mut self, entry: &str) {
        self.add_entry_inner(entry, entry);
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
    pub fn get_file_names(&self) -> std::collections::hash_map::Keys<'_, String, VPKFile> {
        self.files.keys()
    }

    pub fn get_root(&self) -> &VPKDirectoryTree {
        &self.root
    }
    pub fn load(dir_path: PathBuf) -> io::Result<Self> {
        let file = File::open(&dir_path)?;
        let mut buffer = BufReader::new(file);

        let header = VPKHeader_v2::read(&mut buffer)?;
        let mut root = VPKDirectoryTree::Node(HashMap::new());
        let mut maxPackFile = 0;
        let mut files = HashMap::<String, VPKFile>::new();

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
                    let dirPrefix = if (dir == "" || dir == " ") {
                        "".to_owned()
                    } else {
                        format!("{dir}/")
                    };
                    let path = format!("{dirPrefix}{filename}.{ext}");

                    let entry = VPKDirectoryEntry::read(&mut buffer).unwrap();
                    let terminator = entry.Terminator;

                    assert_eq!(terminator, 0xffff);

                    if (entry.ArchiveIndex != 0x7fff) {
                        // 0x7fff means contained in this same file
                        maxPackFile = u16::max(entry.ArchiveIndex, maxPackFile);
                    }

                    // Read metadata.
                    let preload = if entry.PreloadBytes != 0 {
                        let mut buf = vec![0; entry.PreloadBytes as usize];
                        buffer.read_exact(&mut buf[..]).unwrap();
                        Some(buf)
                    } else {
                        None
                    };

                    root.add_entry(&path);

                    files.insert(
                        path,
                        VPKFile {
                            entry,
                            preload,
                            vtf: OnceCell::new(),
                        },
                    );

                    //entries.set(path, { crc, path, chunks, metadataChunk });
                }
            }
        }

        Ok(Self {
            dir_path,
            header,
            maxPackFile,
            root,
            files,
        })
    }
}

impl VPKDirectory {
    pub fn load_vtf(&self, vpk_path: &str) -> io::Result<&VTF> {
        let file_data = self.files.get(vpk_path).ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("File path {} not present", vpk_path),
        ))?;

        if let Some(vtf) = file_data.vtf.get() {
            return Ok(vtf);
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
            buffer.seek_relative(file_data.entry.EntryOffset as i64)?;

            file_data.vtf.set(VTF::read(&mut buffer)?).unwrap();

            return Ok(file_data.vtf.get().unwrap());
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

        let header = VPKHeader_v2::read(&mut buffer).unwrap();

        println!("{:?}", header);
    }

    #[test]
    fn test_dir() {
        let dir = VPKDirectory::load(PathBuf::from(PATH)).unwrap();

        for file in dir.files.keys() {
            if file.contains("floor") {
                println!("{}", file);
            }
        }
    }

    #[test]
    fn test_tree() {
        let mut root = VPKDirectoryTree::Node(HashMap::new());

        root.add_entry("test/file/please/ignore.txt");
        root.add_entry("test/file/please/receive.txt");
        root.add_entry("test/folder/");
        root.add_entry("test/folder/please/receive.txt");

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
