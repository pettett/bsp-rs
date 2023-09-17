// // Valve Texture File

pub mod binary_data;
pub mod consts;
pub mod gui;
mod header;
pub mod vtf;

pub use vtf::VTF;

#[cfg(test)]
mod vtf_tests {
    use std::path::PathBuf;

    use crate::{assets::vpk::VPKDirectory, util::v_path::VGlobalPath};

    const PATH: &str =
        "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_textures_dir.vpk";

    #[test]
    fn test_load() {
        let _dir = VPKDirectory::load(PathBuf::from(PATH)).unwrap();
        // for file in dir.get_file_names() {
        //     if file.contains(".vtf") {
        //         let data = dir.load_vtf(file).unwrap().unwrap();
        //         let _lr = data.header.low_res_image_format;
        //         let hr = data.header.high_res_image_format;
        //         if hr != ImageFormat::DXT5 && hr != ImageFormat::DXT1 {
        //             println!("{} {:?}", file, hr);
        //         }
        //     }
        // }
    }
    #[test]
    fn test_load_materials_metal_metalfence001a() {
        let dir = VPKDirectory::load(PathBuf::from(PATH)).unwrap();

        let data = dir
            .load_vtf(&Into::<VGlobalPath>::into(
                "materials/metal/metalfence001a.vtf",
            ))
            .unwrap()
            .unwrap();
        println!("{:?}", data.header());
    }
}
