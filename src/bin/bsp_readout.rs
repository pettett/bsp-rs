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
    let file = File::open("D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\maps\\d1_canals_01.bsp")?;
    let mut buffer = BufReader::new(file);

    let mut header: dheader_t = unsafe { mem::zeroed() };

    let header_size = mem::size_of::<dheader_t>();
    unsafe {
        let header_slice = slice::from_raw_parts_mut(&mut header as *mut _ as *mut u8, header_size);
        // `read_exact()` comes from `Read` impl for `&[u8]`
        buffer.read_exact(header_slice).unwrap();
    }
    //buffer.read_exact(&mut header.ident).unwrap();

    header.validate();

    let planes: Vec<dplane_t> = header.get_lump(LumpType::PLANES).decode(&mut buffer);
    Lump::validate(&planes);

    let faces: Vec<dface_t> = header.get_lump(LumpType::FACES).decode(&mut buffer);
    Lump::validate(&faces);

    let edges: Vec<dedge_t> = header.get_lump(LumpType::EDGES).decode(&mut buffer);
    Lump::validate(&edges);

    let verts: Vec<Vec3> = header.get_lump(LumpType::VERTEXES).decode(&mut buffer);
    Lump::validate(&verts);

    Ok(())
}
