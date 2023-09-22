pub mod mdl;
pub mod mdl_headers;
pub mod vtx;
pub mod vvd;

pub use mdl::MDL;
pub use vtx::VTX;
pub use vvd::VVD;

#[cfg(test)]
mod mdl_tests {
    use crate::{assets::vpk::VPKDirectory, v::vpath::VGlobalPath};
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
