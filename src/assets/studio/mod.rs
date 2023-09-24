pub mod mdl;
pub mod mdl_headers;
pub mod vtx;
pub mod vvd;

use std::sync::Arc;

pub use mdl::MDL;
pub use vtx::VTX;
pub use vvd::VVD;

use crate::{
    assets::vpk::VPKFile,
    game_data::GameData,
    v::{
        vpath::{VPath, VSplitPath},
        vshader::VShader,
        VMesh,
    },
    vertex::UVAlphaVertex,
};

use self::vvd::Fixup;
pub fn fixup_remapping_search(fixup_table: &Box<[Fixup]>, dst_idx: u16) -> u16 {
    for i in 0..fixup_table.len() {
        let map = fixup_table[i];
        let idx = dst_idx as i32 - map.dst;
        if idx >= 0 && idx < map.count {
            return (map.src + idx) as u16;
        }
    }

    // remap did not copy over this vertex, return as is.
    return dst_idx;
}
pub fn load_vmesh(
    mdl_path: &dyn VPath,
    device: &wgpu::Device,
    shader_tex: Arc<VShader>,
    game_data: &GameData,
) -> Result<VMesh, &'static str> {
    let mdl = game_data
        .load(mdl_path, VPKFile::mdl)
        .ok_or("No mdl file")?;

    let dir = mdl_path.dir();

    let mut vtx_filename = mdl_path.filename().to_owned();
    vtx_filename.push_str(".dx90");
    //println!("{vtx_filename}");
    let vtx_path = VSplitPath::new(&dir, &vtx_filename, "vtx");
    let vvd_path = VSplitPath::new(&dir, mdl_path.filename(), "vvd");

    let vtx = game_data
        .load(&vtx_path, VPKFile::vtx)
        .ok_or("No VTX File")?;
    let vvd = game_data
        .load(&vvd_path, VPKFile::vvd)
        .ok_or("No VVD File")?;

    let l = vtx.header.num_lods as usize;

    assert_eq!(l, vtx.body[0].0[0].0.len());

    let lod0 = &vtx.body[0].0[0].0[0];

    let verts = vvd
        .verts
        .iter()
        .map(|v| UVAlphaVertex {
            position: v.pos,
            uv: v.uv,
            alpha: 1.0,
        })
        .collect::<Vec<_>>();

    for m in &lod0.0 {
        //println!("Mesh {:?}", m.flags);

        for strip_group in &m.strip_groups {
            let mut indices = strip_group.indices.clone();
            if vvd.fixups.len() > 0 {
                let mut map_dsts = vec![0; vvd.fixups.len()];

                for i in 1..vvd.fixups.len() {
                    map_dsts[i] = map_dsts[i - 1] + vvd.fixups[i - 1].count;
                }
                //println!("{:?}", map_dsts);
                //println!("{:?}", vvd.fixups[0]);

                for index in indices.iter_mut() {
                    *index = fixup_remapping_search(
                        &vvd.fixups,
                        strip_group.verts[*index as usize].orig_mesh_vert_id,
                    );
                }
            } else {
                for index in indices.iter_mut() {
                    *index = strip_group.verts[*index as usize].orig_mesh_vert_id;
                }
            }

            for s in &strip_group.strips {
                let ind_start = s.header.index_offset as usize;
                let ind_count = s.header.num_indices as usize;

                let m = VMesh::new(
                    device,
                    &verts[..],
                    &indices[ind_start..ind_start + ind_count],
                    shader_tex,
                );

                return Ok(m);
            }
        }
    }
    Err("No mesh in LODs")
}

#[cfg(test)]
mod mdl_tests {
    use crate::{assets::vpk::VPKDirectory, v::vpath::VGlobalPath};
    use std::path::PathBuf;

    //const PATH: &str = "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Portal 2\\portal2\\maps\\sp_a2_laser_intro.bsp";

    #[test]
    fn test_misc_dir() {
        let dir = VPKDirectory::load(PathBuf::from(
            "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_misc_dir.vpk",
        ))
        .unwrap();
        for (_d, files) in &dir.files["mdl"] {
            for (_file, data) in files {
                let Ok(mdl) = data.load_mdl(&dir) else {
                    continue;
                };
                assert!(mdl.version < 100);
            }
        }
    }

    #[test]
    fn test_single_vtx_misc_dir() {
        let dir = VPKDirectory::load(PathBuf::from(
            "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_misc_dir.vpk",
        ))
        .unwrap();

        let _vtx = dir
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

        let _vvd = dir
            .load_vvd(&VGlobalPath::from("models/props_c17/bench01a.vvd"))
            .unwrap();

        //print!("{:?}", mdl.text);

        assert_eq!(mdl.body.len(), vtx.body[0].0.len());

        let vtx_lod0 = &vtx.body[0].0[0].0;

        for m in vtx_lod0 {
            for _sg in &m.0 {
                //5!("{:?}", sg.indices);
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
