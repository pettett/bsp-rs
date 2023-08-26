use bsp_explorer::bsp::{
    consts::*,
    edges::{dedge_t, dsurfedge_t},
    face::dface_t,
    header::dheader_t,
    plane::dplane_t,
    Lump,
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

    let planes = header.get_lump::<dplane_t>(&mut buffer);
    Lump::validate(&planes);

    let faces = header.get_lump::<dface_t>(&mut buffer);
    Lump::validate(&faces);

    let edges = header.get_lump::<dedge_t>(&mut buffer);
    Lump::validate(&edges);

    let surfedges = header.get_lump::<dsurfedge_t>(&mut buffer);
    Lump::validate(&surfedges);

    let verts = header.get_lump::<Vec3>(&mut buffer);
    Lump::validate(&verts);

    Ok(())
}
