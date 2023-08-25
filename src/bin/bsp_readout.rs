use bsp_explorer::bsp::{
    consts::*, edge::dedge_t, face::dface_t, header::dheader_t, plane::dplane_t, Lump,
};
use glam::Vec3;
use num_traits::FromPrimitive;
use std::{
    fmt,
    fs::File,
    io::{self, BufReader, Read, Seek},
    mem, slice,
};

pub fn main() -> io::Result<()> {
    let (header, mut buffer) = dheader_t::load("D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\maps\\d1_trainstation_01.bsp")?;
    header.validate();

    let planes: Box<[dplane_t]> = header.get_lump(LumpType::PLANES).decode(&mut buffer);
    Lump::validate(&planes);

    let faces: Box<[dface_t]> = header.get_lump(LumpType::FACES).decode(&mut buffer);
    Lump::validate(&faces);

    let edges: Box<[dedge_t]> = header.get_lump(LumpType::EDGES).decode(&mut buffer);
    Lump::validate(&edges);

    let verts: Box<[Vec3]> = header.get_lump(LumpType::VERTEXES).decode(&mut buffer);
    Lump::validate(&verts);

    Ok(())
}
