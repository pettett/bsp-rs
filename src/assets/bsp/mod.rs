pub mod consts;
pub mod displacement;
pub mod edges;
pub mod face;
pub mod gamelump;
pub mod header;
pub mod lightmap;
pub mod loader;
pub mod lump;
pub mod model;
pub mod plane;
pub mod textures;
pub mod vert;

pub use consts::LumpType;
pub use lump::Lump;

// https://developer.valvesoftware.com/wiki/BSP_(Source)
//
// https://github.com/ValveSoftware/source-sdk-2013/blob/master/mp/src/public/bspfile.h
//
// The BSP file contains the vast majority of the information needed by the Source engine to render and play a map.
// This includes the geometry of all the polygons in the level; references to the names and orientation of the textures
// to be drawn on those polygons; the data used to simulate the physical behaviour of the player and other items during
// the game; the location and properties of all brush-based, model (prop) based, and non-visible (logical) entities in
// the map; and the BSP tree and visibility table used to locate the player location in the map geometry and to render
// the visible map as efficiently as possible. Optionally, the map file can also contain any custom textures and models
// used on the level, embedded inside the map's Pakfile lump (see below).
//
// Information not stored in the BSP file includes the map description text displayed by multiplayer games (such as
// Counter-Strike: Source or Half-Life 2: Deathmatch) after loading the map (stored in the file mapname.txt) and the
// AI navigation file used by non-player characters (NPCs) which need to navigate the map (stored in the file mapname.nav).
// Because of the way the Source engine file system works, these external files may also be embedded in the BSP file's Pakfile lump,
// though usually they are not.
//
// Historically, map files were stored in the game's corresponding Steam Game Cache File (GCF), but these are no longer used
// since 2013. In current versions of all of Valve's games, maps are stored directly in the OS file system. Rarely, such as in
// some third-party games or mods (especially
// Black Mesa and most Source 2004/Source 2006 games that have been updated with SteamPipe), maps may be stored in VPK files;
// these can be extracted using Nemesis' GCFScape.
//
// The data in the BSP file can be stored in little-endian for
// PC or in big-endian for consoles such as the PlayStation 3 and Xbox 360. Byte-swapping is required when loading a
// little-endian file on a big-endian format platform such as Java and vice versa.

#[cfg(test)]
mod bsp_tests {
    use std::path::Path;

    use stream_unzip::ZipReader;

    use bytemuck::Zeroable;
    use glam::Vec3;

    use crate::assets::bsp::consts::{num_disp_power_verts, MAX_MAP_TEXDATA_STRING_DATA};

    use super::{
        consts::LumpType,
        displacement::{BSPDispInfo, BSPDispVert},
        edges::{BSPEdge, BSPSurfEdge},
        face::BSPFace,
        header::BSPHeader,
        model::BSPModel,
        plane::BSPPlane,
        textures::{BSPTexData, BSPTexDataStringTable, BSPTexInfo},
        Lump,
    };

    const PATH : &str = "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\maps\\d1_trainstation_02.bsp";

    #[test]
    fn test_header() {
        let (header, _buffer) = BSPHeader::load(Path::new(PATH)).unwrap();
        header.validate();
    }

    #[test]
    fn planes() {
        let lump = test_lump::<BSPPlane>();

        for plane in lump.iter() {
            let axis = plane.axis;
            assert!((0..=5).contains(&axis));
        }

        println!("Validated planes lump!")
    }
    #[test]
    fn edges() {
        test_lump::<BSPEdge>();
    }
    #[test]
    fn surfedges() {
        test_lump::<BSPSurfEdge>();
    }
    #[test]
    fn verts() {
        test_lump::<Vec3>();
    }
    #[test]
    fn faces() {
        test_lump::<BSPFace>();
    }

    #[test]
    fn texdata() {
        test_lump::<BSPTexData>();
    }
    #[test]
    fn texinfo() {
        test_lump::<BSPTexInfo>();
    }

    #[test]
    fn models() {
        test_lump::<BSPModel>();
    }

    #[test]
    fn displacements() {
        let (header, mut buffer) = BSPHeader::load(Path::new(PATH)).unwrap();
        let infos = header.get_lump::<BSPDispInfo>(&mut buffer);
        let verts = header.get_lump::<BSPDispVert>(&mut buffer);
        let faces = header.get_lump::<BSPFace>(&mut buffer);

        //ensure every vertex is accounted for
        let mut vert_marks = vec![0; verts.len()];

        for i in 0..infos.len() {
            let info = infos[i];
            println!("{:#?}", info);
            let disp_vert_count = num_disp_power_verts(info.power as usize);
            assert_eq!(
                disp_vert_count,
                (2u32.pow(info.power) + 1u32).pow(2) as usize
            );

            let face = faces[info.map_face as usize];

            let disp_info = face.disp_info;
            assert_eq!(disp_info, i as i16);

            for n in &info.edge_neighbours {
                for sn in n.sub_neighbours.iter() {
                    if sn.i_neighbour != 0xFFFF {
                        assert!((sn.i_neighbour as usize) < verts.len())
                    }
                }
            }
            //ensure every vertex has been mapped to
            for i in 0..disp_vert_count {
                vert_marks[(i + info.disp_vert_start as usize).min(verts.len() - 1)] += 1;
            }
        }
        assert!(vert_marks.iter().all(|&x| x == 1));
    }

    #[test]
    fn pakfile() {
        let (header, mut buffer) = BSPHeader::load(Path::new(PATH)).unwrap();
        let pakfile = header.get_lump_header(LumpType::PakFile);

        let pakfile_data = pakfile.read_bytes(&mut buffer).unwrap();

        let mut zip_reader = ZipReader::default();

        zip_reader.update(pakfile_data.into());

        // Or read the whole file and deal with the entries
        // at the end.
        zip_reader.finish();
        let entries = zip_reader.drain_entries();
        for entry in entries {
            println!("entry: {:?}", entry.header().filename);
            // write to disk or whatever you need.
            if entry.header().filename.contains(".vmt") {
                println!("{}", std::str::from_utf8(entry.compressed_data()).unwrap());
            }
        }
    }

    #[test]
    fn texdatastringtable() {
        test_lump::<BSPTexDataStringTable>();
    }
    #[test]
    fn tex_data_string_data() {
        let (header, mut buffer) = BSPHeader::load(Path::new(PATH)).unwrap();
        let tex_data_string_data = header.get_lump_header(LumpType::TexDataStringData);

        assert!(tex_data_string_data.file_len <= MAX_MAP_TEXDATA_STRING_DATA);

        let strings = tex_data_string_data.read_bytes(&mut buffer).unwrap();

        // ensure it's utf8
        let all_textures = std::str::from_utf8(&strings).unwrap();

        println!("{}", all_textures);
    }

    #[test]
    fn textures() {
        let (header, mut buffer) = BSPHeader::load(Path::new(PATH)).unwrap();
        let tex_info = header.get_lump::<BSPTexInfo>(&mut buffer);
        let tex_data = header.get_lump::<BSPTexData>(&mut buffer);
        let tex_data_string_table = header.get_lump::<BSPTexDataStringTable>(&mut buffer);
        let tex_data_string_data = header.get_lump_header(LumpType::TexDataStringData);

        // test data relation
        for info in tex_info.iter() {
            let data = tex_data[info.tex_data as usize];
            println!("{:?}", data);
        }
        // test data itself
        for data in tex_data.iter() {
            let string = tex_data_string_table[data.name_string_table_id as usize]
                .get_filename(&mut buffer, tex_data_string_data);

            println!("{}", string);
        }
    }

    fn test_lump<T: Lump + Clone + Zeroable>() -> Box<[T]> {
        let (header, mut buffer) = BSPHeader::load(Path::new(PATH)).unwrap();
        let lump = header
            .get_lump_header(T::lump_type())
            .decode(&mut buffer)
            .unwrap();

        assert!(lump.len() < BSPPlane::max());
        return lump;
    }
}
