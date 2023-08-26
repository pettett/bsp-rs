pub mod consts;
pub mod edges;
pub mod edges_debug_mesh;
pub mod face;
pub mod header;
pub mod lump;
pub mod plane;
pub mod textures;
pub mod vert;

pub use lump::Lump;

// https://developer.valvesoftware.com/wiki/BSP_(Source)
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
    use std::io::{BufRead, Read, Seek};
    use stream_unzip::ZipReader;

    use bytemuck::Zeroable;
    use glam::Vec3;

    use crate::bsp::consts::MAX_MAP_TEXDATA_STRING_DATA;

    use super::{
        consts::LumpType,
        edges::{dedge_t, dsurfedge_t},
        face::dface_t,
        header::dheader_t,
        plane::dplane_t,
        textures::{texdata_t, texdatastringtable_t, texinfo_t},
        Lump,
    };

    const PATH : &str = "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\maps\\d1_trainstation_01.bsp";

    #[test]
    fn test_header() {
        let (header, mut buffer) = dheader_t::load(PATH).unwrap();
        header.validate();
    }

    #[test]
    fn planes() {
        test_lump::<dplane_t>();
    }
    #[test]
    fn edges() {
        test_lump::<dedge_t>();
    }
    #[test]
    fn surfedges() {
        test_lump::<dsurfedge_t>();
    }
    #[test]
    fn verts() {
        test_lump::<Vec3>();
    }
    #[test]
    fn faces() {
        test_lump::<dface_t>();
    }

    #[test]
    fn texdata() {
        test_lump::<texdata_t>();
    }
    #[test]
    fn texinfo() {
        test_lump::<texinfo_t>();
    }

    #[test]
    fn pakfile() {
        let (header, mut buffer) = dheader_t::load(PATH).unwrap();
        let pakfile = header.get_lump_header(LumpType::PAKFILE);

        buffer
            .seek(std::io::SeekFrom::Start(pakfile.fileofs as u64))
            .unwrap();

        let mut pakfile_data = vec![0; pakfile.filelen as usize];

        buffer.read_exact(&mut pakfile_data).unwrap();

        let mut zip_reader = ZipReader::default();

        zip_reader.update(pakfile_data.into());

        // Or read the whole file and deal with the entries
        // at the end.
        zip_reader.finish();
        let entries = zip_reader.drain_entries();
        for entry in entries {
            println!("entry: {:?}", entry);
            // write to disk or whatever you need.
        }
    }

    #[test]
    fn texdatastringtable() {
        test_lump::<texdatastringtable_t>();
    }
    #[test]
    fn texdatastringdata() {
        let (header, mut buffer) = dheader_t::load(PATH).unwrap();
        let texdatastringdata = header.get_lump_header(LumpType::TEXDATA_STRING_DATA);

        assert!(texdatastringdata.filelen <= MAX_MAP_TEXDATA_STRING_DATA);

        buffer
            .seek(std::io::SeekFrom::Start(texdatastringdata.fileofs as u64))
            .unwrap();

        let mut strings = vec![0; texdatastringdata.filelen as usize];

        buffer.read_exact(&mut strings).unwrap();
        // ensure it's utf8
        let all_textures = std::str::from_utf8(&strings).unwrap();

        //println!("{}", all_textures);
    }

    #[test]
    fn textures() {
        let (header, mut buffer) = dheader_t::load(PATH).unwrap();
        let texinfo = header.get_lump::<texinfo_t>(&mut buffer);
        let texdata = header.get_lump::<texdata_t>(&mut buffer);
        let texdatastringtable = header.get_lump::<texdatastringtable_t>(&mut buffer);
        let texdatastringdata = header.get_lump_header(LumpType::TEXDATA_STRING_DATA);

        // test data relation
        for info in texinfo.iter() {
            let data = texdata[info.texdata as usize];
        }
        // test data itself
        for data in texdata.iter() {
            let string = texdatastringtable[data.nameStringTableID as usize];
            let index = string.index;

            let seek_index = index + texdatastringdata.fileofs;

            buffer
                .seek(std::io::SeekFrom::Start(seek_index as u64))
                .unwrap();

            let mut string_buf = Vec::new();

            buffer.read_until(0, &mut string_buf).unwrap();

            let tex_name = unsafe { std::str::from_utf8_unchecked(&string_buf[..]) };

            println!("{} {}", index, tex_name);
        }
    }

    fn test_lump<T: Lump + Clone + Zeroable>() {
        let (header, mut buffer) = dheader_t::load(PATH).unwrap();
        let lump: Box<[T]> = header.get_lump_header(T::lump_type()).decode(&mut buffer);
        Lump::validate(&lump);
    }
}
