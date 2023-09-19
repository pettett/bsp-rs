use std::{io::Seek, mem};

use crate::binaries::BinaryData;

use self::headers::{mstudiobodyparts_t, mstudiotexture_t};

pub mod headers;

#[derive(Debug)]
pub struct StudioModel {
    pub header: headers::MDLHeader,
    pub body: Vec<(i64, mstudiobodyparts_t)>,
    pub text: Vec<(i64, mstudiotexture_t)>,
}

impl BinaryData for StudioModel {
    fn read<R: std::io::Read + std::io::Seek>(
        buffer: &mut std::io::BufReader<R>,
        max_size: Option<usize>,
    ) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let header = headers::MDLHeader::read(buffer, None)?;

        let mut pos = mem::size_of::<headers::MDLHeader>() as i64;

        let body = header.bodypart.read(buffer, &mut pos)?;
        let text = header.texture.read(buffer, &mut pos)?;

        for (i, t) in &text {
            println!("{}", t.name_offset.read_str(buffer, *i, &mut pos)?)
        }

        Ok(Self { header, body, text })
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
    fn test_single_misc_dir() {
        let dir = VPKDirectory::load(PathBuf::from(
            "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_misc_dir.vpk",
        ))
        .unwrap();

        let Ok(Some(mdl)) = dir.load_mdl(&VGlobalPath::from("models/props_c17/bench01a.mdl"))
        else {
            panic!()
        };
        // https://github.com/magcius/noclip.website/blob/70232e5e4a08bac6a242a6b74270e67c6f06fcca/src/SourceEngine/Studio.ts#L1140
        assert_eq!(mdl.header.id, [b'I', b'D', b'S', b'T']);
        assert!(mdl.header.version < 100);

        println!("{mdl:#?}");

        return;
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
