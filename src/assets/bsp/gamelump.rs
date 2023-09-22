use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader, Seek},
};

use fixedstr::zstr;
use glam::Vec3;

use crate::binaries::BinaryData;

use super::{lump::BSPLump, Lump};

#[derive(Debug, bytemuck::Zeroable)]
#[repr(C, packed)]
pub struct StaticPropLumpV5_t {
    pub m_Origin: Vec3,
    pub m_Angles: Vec3,
    pub m_PropType: u16,
    pub m_FirstLeaf: u16,
    pub m_LeafCount: u16,
    pub m_Solid: u8,
    pub m_Flags: u8,
    pub m_Skin: i32,
    pub m_FadeMinDist: f32,
    pub m_FadeMaxDist: f32,
    pub m_LightingOrigin: Vec3,
    pub m_flForcedFadeScale: f32, //	int				m_Lighting;			// index into the GAMELUMP_STATIC_PROP_LIGHTING lump
}

#[derive(Debug, bytemuck::Zeroable)]
#[repr(C, packed)]
struct dgamelump_t {
    id: [u8; 4],  // gamelump ID
    flags: u16,   // flags
    version: u16, // gamelump version
    fileofs: i32, // offset to this gamelump
    filelen: i32, // length
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct PropDictEntry {
    name: zstr<128>, // model name
}

impl BinaryData for PropDictEntry {}

#[derive(Debug)]
pub struct GameLump {
    pub static_prop_names: Vec<String>,
    pub props: Vec<StaticPropLumpV5_t>,
}

pub fn load_gamelump(lump: &BSPLump, buffer: &mut BufReader<File>) -> io::Result<GameLump> {
    buffer.seek(std::io::SeekFrom::Start(lump.file_ofs as u64))?;

    let lump_count = i32::read(buffer, None)?;

    let mut lumps = HashMap::new();
    for i in 0..lump_count {
        let e = dgamelump_t::read(buffer, None)?;

        lumps.insert(e.id.clone(), e);
    }

    let static_props_lump = lumps.get(b"prps").unwrap();
    //TODO: Support more versions
    assert!(static_props_lump.version == 5);

    buffer.seek(std::io::SeekFrom::Start(static_props_lump.fileofs as u64))?;

    let dict_entries = i32::read(buffer, None)?;

    let mut static_prop_names = Vec::new();

    for i in 0..dict_entries {
        let e = PropDictEntry::read(buffer, None)?;

        static_prop_names.push(e.name.to_ascii_lowercase());
    }
    let leafs = i32::read(buffer, None)?;
    for i in 0..leafs {
        u16::read(buffer, None)?;
    }

    let prop_lumps = i32::read(buffer, None)?;
    let mut props = Vec::new();
    for p in 0..prop_lumps {
        let prop = StaticPropLumpV5_t::read(buffer, None)?;

        assert!(0 <= prop.m_PropType && (prop.m_PropType as usize) < static_prop_names.len());

        props.push(prop);
    }

    Ok(GameLump {
        static_prop_names,
        props,
    })
}

#[cfg(test)]
mod gamelump_tests {
    use std::{collections::HashMap, io::Seek, path::Path};

    use crate::assets::bsp::{header::BSPHeader, LumpType};

    use super::*;

    const PATH : &str = "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\maps\\d1_trainstation_02.bsp";

    #[test]
    fn static_props() {
        let (header, mut buffer) = BSPHeader::load(Path::new(PATH)).unwrap();

        let h = header.get_lump_header(LumpType::GameLump);

        let gamelump = load_gamelump(h, &mut buffer).unwrap();
    }
}
