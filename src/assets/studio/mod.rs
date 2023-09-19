use std::{
    io::{self, BufRead, BufReader, Read, Seek},
    marker::PhantomData,
    mem,
};

use crate::{
    assets::studio::mdl_headers::{mstudiomesh_t, mstudiomodel_t},
    binaries::BinaryData,
};

use self::mdl_headers::{mstudiobodyparts_t, mstudiotexture_t};

pub mod mdl;
pub mod mdl_headers;
pub mod vertex;
pub mod vtx;
pub mod vvd;

pub use mdl::MDL;
pub use vtx::VTX;
pub use vvd::VVD;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
pub struct BinOffset {
    pub index: u32,
}

impl BinOffset {
    fn seek_start<R: Read + Seek>(
        &self,
        buffer: &mut BufReader<R>,
        start: i64,
        pos: &mut i64,
    ) -> io::Result<()> {
        let p = self.index as i64 + start;
        buffer.seek_relative(p - *pos)?;
        *pos = p;
        Ok(())
    }
    pub fn read_str<R: Read + Seek>(
        &self,
        buffer: &mut BufReader<R>,
        start: i64,
        pos: &mut i64,
    ) -> io::Result<String> {
        self.seek_start(buffer, start, pos)?;
        //let mut b = [0, 0, 0, 0];
        //buffer.read_exact(&mut b)?;
        //println!("{b:?}");
        let mut data = Default::default();

        *pos += buffer.read_until(0, &mut data)? as i64;

        Ok(String::from_utf8(data).unwrap())
    }

    pub fn read_array<T: Sized + BinaryData, R: Read + Seek>(
        &self,
        buffer: &mut BufReader<R>,
        start: i64,
        pos: &mut i64,
        count: u32,
    ) -> io::Result<Vec<(i64, T)>> {
        self.seek_start(buffer, start, pos)?;

        //let mut b = [0, 0, 0, 0];
        //buffer.read_exact(&mut b)?;
        //println!("{b:?}");

        let mut v = Vec::new();
        v.reserve(self.index as usize);

        for _ in 0..count {
            v.push((*pos, T::read(buffer, None)?));
            *pos += mem::size_of::<T>() as i64;
        }

        Ok(v)
    }
}

/// Struct of (count, offset) for reading an array of items from an mdl
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
pub struct BinArray<T: Sized + BinaryData> {
    pub count: u32,
    pub offset: BinOffset,
    _p: PhantomData<T>,
}

impl<T: Sized + BinaryData> BinArray<T> {
    pub fn read<R: Read + Seek>(
        &self,
        buffer: &mut BufReader<R>,
        start: i64,
        pos: &mut i64,
    ) -> io::Result<Vec<(i64, T)>> {
        self.offset.read_array(buffer, start, pos, self.count)
    }
}

#[cfg(test)]
mod mdl_tests {
    use crate::{assets::vpk::VPKDirectory, util::v_path::VGlobalPath};
    use std::path::PathBuf;

    const PATH: &str = "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Portal 2\\portal2\\maps\\sp_a2_laser_intro.bsp";

    #[test]
    fn test_misc_dir() {
        let dir = VPKDirectory::load(PathBuf::from(
            "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_misc_dir.vpk",
        ))
        .unwrap();
        for (d, files) in &dir.files["mdl"] {
            for (_file, data) in files {
                let Ok(Some(mdl)) = data.load_mdl(&dir) else {
                    continue;
                };
                assert!(mdl.header.version < 100);
            }
        }
    }

    #[test]
    fn test_single_vtx_misc_dir() {
        let dir = VPKDirectory::load(PathBuf::from(
            "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_misc_dir.vpk",
        ))
        .unwrap();

        let Ok(Some(vtx)) = dir.load_vtx(&VGlobalPath::from("models/props_c17/bench01a.dx90.vtx"))
        else {
            panic!()
        };

        println!("{:#?}", vtx);
    }
    #[test]
    fn test_single_vvd_misc_dir() {
        let dir = VPKDirectory::load(PathBuf::from(
            "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_misc_dir.vpk",
        ))
        .unwrap();

        let Ok(Some(vvd)) = dir.load_vvd(&VGlobalPath::from("models/props_c17/bench01a.dx90.vtx"))
        else {
            panic!()
        };

        println!("{:?}", vvd);
    }

    #[test]
    fn test_single_misc_dir() {
        let dir = VPKDirectory::load(PathBuf::from(
            "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_misc_dir.vpk",
        ))
        .unwrap();

        let Ok(Some(vtx)) = dir.load_vtx(&VGlobalPath::from("models/props_c17/bench01a.dx90.vtx"))
        else {
            panic!()
        };
        let Ok(Some(mdl)) = dir.load_mdl(&VGlobalPath::from("models/props_c17/bench01a.dx90.vtx"))
        else {
            panic!()
        };
        let Ok(Some(vvd)) = dir.load_vvd(&VGlobalPath::from("models/props_c17/bench01a.dx90.vtx"))
        else {
            panic!()
        };
    }

    // #[test]
    // fn test_misc_dir_p2() {
    //     let dir = VPKDirectory::load(PathBuf::from(
    //         "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Portal 2\\portal2\\pak01_dir.vpk",
    //     ))
    //     .unwrap();

    //     let mut shaders = HashSet::new();

    //     for (d, files) in &dir.files["vmt"] {
    //         for (_file, data) in files {
    //             let Ok(Some(vmt)) = data.load_vmt(&dir) else {
    //                 continue;
    //             };
    //             shaders.insert(vmt.shader.to_ascii_lowercase());
    //         }
    //     }
    //     println!("{:?}", shaders);
    // }

    // #[test]
    // fn test_maps() {
    //     let (header, mut buffer) = BSPHeader::load(Path::new(PATH)).unwrap();

    //     let pak_header = header.get_lump_header(LumpType::PakFile);

    //     let dir: VPKDirectory = pak_header.read_binary(&mut buffer).unwrap();

    //     for (d, files) in &dir.files["vmt"] {
    //         println!("DIR: {}", d);
    //         for (_file, data) in files {
    //             let Ok(Some(vmt)) = data.load_vmt(&dir) else {
    //                 continue;
    //             };
    //             println!("{:?}", vmt.data);
    //         }
    //     }
    // }
}
