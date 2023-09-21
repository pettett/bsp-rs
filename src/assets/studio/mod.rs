use std::{
    fmt::Debug,
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
    pub fn seek_start<R: Read + Seek>(
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

        // Remove trailing 0
        data.pop();

        Ok(String::from_utf8(data).unwrap())
    }
    //TODO: choose name
    pub fn read_array_f<
        T: Sized + bytemuck::Zeroable + bytemuck::Pod + BinaryData,
        R: Read + Seek,
    >(
        &self,
        buffer: &mut BufReader<R>,
        start: i64,
        pos: &mut i64,
        count: usize,
    ) -> io::Result<Box<[T]>> {
        self.seek_start(buffer, start, pos)?;

        //let mut b = [0, 0, 0, 0];
        //buffer.read_exact(&mut b)?;
        //println!("{b:?}");

        let b = T::read_array(buffer, count, None)?;

        *pos += (count * mem::size_of::<T>()) as i64;
        Ok(b)
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
        v.reserve_exact(self.index as usize);

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
impl<T: Sized + BinaryData + bytemuck::Pod> BinArray<T> {
    pub fn read_f<R: Read + Seek>(
        &self,
        buffer: &mut BufReader<R>,
        start: i64,
        pos: &mut i64,
    ) -> io::Result<Box<[T]>> {
        self.offset
            .read_array_f(buffer, start, pos, self.count as usize)
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
                let Ok(mdl) = data.load_mdl(&dir) else {
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

        let vtx = dir
            .load_vtx(&VGlobalPath::from("models/props_c17/bench01a.dx90.vtx"))
            .unwrap();
        println!("{:#?}", vtx);
    }

    #[test]
    fn test_single_vvd_misc_dir() {
        let dir = VPKDirectory::load(PathBuf::from(
            "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_misc_dir.vpk",
        ))
        .unwrap();

        let vvd = dir
            .load_vvd(&VGlobalPath::from("models/props_c17/bench01a.vvd"))
            .unwrap();

        for t in vvd.tangents.iter() {
            assert!(t.w == 0.0 || t.w == -1.0 || t.w == 1.0)
        }
        println!("Tangents all good!");
    }

    #[test]
    fn test_single_misc_dir() {
        let dir = VPKDirectory::load(PathBuf::from(
            "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_misc_dir.vpk",
        ))
        .unwrap();

        let vtx = dir
            .load_vtx(&VGlobalPath::from("models/props_c17/bench01a.dx90.vtx"))
            .unwrap();

        let mdl = dir
            .load_mdl(&VGlobalPath::from("models/props_c17/bench01a.mdl"))
            .unwrap();

        let vvd = dir
            .load_vvd(&VGlobalPath::from("models/props_c17/bench01a.vvd"))
            .unwrap();

        assert_eq!(mdl.body.len(), vtx.body[0].0.len());

        let vtx_lod0 = &vtx.body[0].0[0].0;

        for m in vtx_lod0 {
            for sg in &m.0 {
                //println!("{:?}", sg.indices);
            }
        }

        println!("{:#?}", mdl);
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
