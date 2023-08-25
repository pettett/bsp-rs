use std::{
    fmt,
    fs::File,
    io::{self, BufReader, Read},
    mem, slice,
};

const HEADER_LUMPS: usize = 64;
// https://developer.valvesoftware.com/wiki/BSP_(Source)
#[repr(C, packed)]
#[derive(Debug, Default, Copy, Clone)]
struct lump_t {
    fileofs: i32,    // offset into file (bytes)
    filelen: i32,    // length of lump (bytes)
    version: i32,    // lump format version
    fourCC: [u8; 4], // lump ident code
}
#[repr(C, packed)]
#[derive(Copy, Clone)]
struct dheader_t {
    ident: [u8; 4],                // BSP file identifier
    version: i32,                  // BSP file version
    lumps: [lump_t; HEADER_LUMPS], // lump directory array
    mapRevision: i32,              // the map's revision (iteration, version) number
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct Vector {
    x: i32,
    y: i32,
    z: i32,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct dplane_t {
    normal: Vector, // normal vector
    dist: f32,      // distance from origin
    axis: i32,      // plane axis identifier
}

#[derive(Copy, Clone)]
enum LumpType {
    ENTITIES = 0,
    PLANES = 1,
    TEXDATA = 2,
    VERTEXES = 3,
    VISIBILITY = 4,
    NODES = 5,
    TEXINFO = 6,
    FACES = 7,
    LIGHTING = 8,
    LEAFS = 10,
    EDGES = 12,
    SURFEDGES = 13,
    MODELS = 14,
    WORLDLIGHTS = 15,
    LEAFFACES = 16,
    DISPINFO = 26,
    VERTNORMALS = 30,
    VERTNORMALINDICES = 31,
    DISP_VERTS = 33,
    GAME_LUMP = 35,
    LEAFWATERDATA = 36,
    PRIMITIVES = 37,
    PRIMINDICES = 39,
    PAKFILE = 40,
    CUBEMAPS = 42,
    TEXDATA_STRING_DATA = 43,
    TEXDATA_STRING_TABLE = 44,
    OVERLAYS = 45,
    LEAF_AMBIENT_INDEX_HDR = 51,
    LEAF_AMBIENT_INDEX = 52,
    LIGHTING_HDR = 53,
    WORLDLIGHTS_HDR = 54,
    LEAF_AMBIENT_LIGHTING_HDR = 55,
    LEAF_AMBIENT_LIGHTING = 56,
    FACES_HDR = 58,
}

impl Default for dheader_t {
    fn default() -> Self {
        Self {
            ident: Default::default(),
            version: Default::default(),
            lumps: [lump_t::default(); 64],
            mapRevision: Default::default(),
        }
    }
}

impl fmt::Debug for dheader_t {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version = self.version;
        let mapRevision = self.mapRevision;
        f.debug_struct("dheader_t")
            .field("ident", &self.ident)
            .field("version", &version)
            .field("mapRevision", &mapRevision)
            .finish()
    }
}

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

    // Check the magic number
    // This way around means little endian, PSBV is big endian
    let text = "VBSP";
    let magic_number: [u8; 4] = text
        .chars()
        .map(|c| c as u8)
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();

    assert_eq!(header.ident, magic_number);

    //let mut bytes = [0, 0, 0, 0];
    //buffer.read_exact(&mut bytes).unwrap();
    //header.version = i32::from_le_bytes(bytes);

    println!("{header:?}");
    for lump in &header.lumps {
        let ofs = lump.fileofs;
        let len = lump.filelen;
        println!("{ofs:?} {len:?}");
    }
    Ok(())
}
