use bsp_explorer::bsp::{
    edges::{BSPEdge, BSPSurfEdge},
    face::BSPFace,
    header::BSPHeader,
    plane::BSPPlane,
    Lump,
};
use glam::Vec3;

use std::{
    io::{self},
    path::Path,
};

pub fn main() -> io::Result<()> {
    let (header, mut buffer) = BSPHeader::load(Path::new("D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\maps\\d1_trainstation_01.bsp"))?;
    header.validate();

    let planes = header.get_lump::<BSPPlane>(&mut buffer);
    Lump::validate(&planes);

    let faces = header.get_lump::<BSPFace>(&mut buffer);
    Lump::validate(&faces);

    let edges = header.get_lump::<BSPEdge>(&mut buffer);
    Lump::validate(&edges);

    let surfedges = header.get_lump::<BSPSurfEdge>(&mut buffer);
    Lump::validate(&surfedges);

    let verts = header.get_lump::<Vec3>(&mut buffer);
    Lump::validate(&verts);

    Ok(())
}
