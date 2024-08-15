use flagset::flags;
use num_derive::FromPrimitive;

pub const HEADER_LUMPS: usize = 64;

// upper design bounds
pub const MIN_MAP_DISP_POWER: usize = 2; // Minimum and maximum power a displacement can be.
pub const MAX_MAP_DISP_POWER: usize = 4;

// Max # of neighboring displacement touching a displacement's corner.
pub const MAX_DISP_CORNER_NEIGHBORS: usize = 4;

pub const fn num_disp_power_verts(power: usize) -> usize {
    ((1 << (power)) + 1) * ((1 << (power)) + 1)
}
pub const fn num_disp_power_tris(power: usize) -> usize {
    (1 << (power)) * (1 << (power)) * 2
}

pub const MAX_MAP_MODELS: usize = 1024;
pub const MAX_MAP_BRUSHES: usize = 8192;
pub const MAX_MAP_ENTITIES: usize = 8192;
pub const MAX_MAP_TEXINFO: usize = 12288;
pub const MAX_MAP_TEXDATA: usize = 2048;
pub const MAX_MAP_DISPINFO: usize = 2048;
pub const MAX_MAP_DISP_VERTS: usize =
    MAX_MAP_DISPINFO * ((1 << MAX_MAP_DISP_POWER) + 1) * ((1 << MAX_MAP_DISP_POWER) + 1);
pub const MAX_MAP_DISP_TRIS: usize = (1 << MAX_MAP_DISP_POWER) * (1 << MAX_MAP_DISP_POWER) * 2;
pub const MAX_DISPVERTS: usize = num_disp_power_verts(MAX_MAP_DISP_POWER);
pub const MAX_DISPTRIS: usize = num_disp_power_tris(MAX_MAP_DISP_POWER);
pub const MAX_MAP_AREAS: usize = 256;
pub const MAX_MAP_AREA_BYTES: usize = MAX_MAP_AREAS / 8;
pub const MAX_MAP_AREAPORTALS: usize = 1024;
// Planes come in pairs, thus an even number.
pub const MAX_MAP_PLANES: usize = 65536;
pub const MAX_MAP_NODES: usize = 65536;
pub const MAX_MAP_BRUSHSIDES: usize = 65536;
pub const MAX_MAP_LEAFS: usize = 65536;
pub const MAX_MAP_VERTS: usize = 65536;
pub const MAX_MAP_VERTNORMALS: usize = 256000;
pub const MAX_MAP_VERTNORMALINDICES: usize = 256000;
pub const MAX_MAP_FACES: usize = 65536;
pub const MAX_MAP_LEAFFACES: usize = 65536;
pub const MAX_MAP_LEAFBRUSHES: usize = 65536;
pub const MAX_MAP_PORTALS: usize = 65536;
pub const MAX_MAP_CLUSTERS: usize = 65536;
pub const MAX_MAP_LEAFWATERDATA: usize = 32768;
pub const MAX_MAP_PORTALVERTS: usize = 128000;
pub const MAX_MAP_EDGES: usize = 256000;
pub const MAX_MAP_SURFEDGES: usize = 512000;
pub const MAX_MAP_LIGHTING: usize = 0x1000000;
pub const MAX_MAP_VISIBILITY: usize = 0x1000000; // increased BSPVERSION 7
pub const MAX_MAP_TEXTURES: usize = 1024;
pub const MAX_MAP_WORLDLIGHTS: usize = 8192;
pub const MAX_MAP_CUBEMAPSAMPLES: usize = 1024;
pub const MAX_MAP_OVERLAYS: usize = 512;
pub const MAX_MAP_WATEROVERLAYS: usize = 16384;
pub const MAX_MAP_TEXDATA_STRING_DATA: i32 = 256000;
pub const MAX_MAP_TEXDATA_STRING_TABLE: usize = 65536;
// this is stuff for trilist/tristrips, etc.
pub const MAX_MAP_PRIMITIVES: usize = 32768;
pub const MAX_MAP_PRIMVERTS: usize = 65536;
pub const MAX_MAP_PRIMINDICES: usize = 65536;

pub const TEXTURE_NAME_LENGTH: usize = 128;

#[derive(Copy, Clone, FromPrimitive, Debug)]
pub enum LumpType {
    Entities = 0,
    Places = 1,
    TexData = 2,
    Vertexes = 3,
    Visibility = 4,
    Nodes = 5,
    TexInfo = 6,
    Faces = 7,
    Lighting = 8,
    Leafs = 10,
    Edges = 12,
    SurfEdges = 13,
    Models = 14,
    WorldLights = 15,
    LeafFaces = 16,
    DispInfo = 26,
    OriginalFaces = 27,
    VertNormals = 30,
    VertNormalIndices = 31,
    DispVerts = 33,
    GameLump = 35,
    LeafWaterData = 36,
    Primitives = 37,
    PrimIndices = 39,
    PakFile = 40,
    Cubemaps = 42,
    TexDataStringData = 43,
    TexDataStringTable = 44,
    Overlays = 45,
    LeafAmbientIndexHdr = 51,
    LeafAmbientIndex = 52,
    LightingHdr = 53,
    WorldLightsHdr = 54,
    LeafAmbientLightingHdr = 55,
    LeafAmbientLighting = 56,
    FacesHdr = 58,
}
flags! {
    enum Contents: i32 {
        EMPTY = 0,             //N.o contents
        SOLID = 0x1,           //an eye is never valid in a solid
        WINDOW = 0x2,          //translucent, but not watery (glass)
        AUX = 0x4,             //
        GRATE = 0x8, //alpha-tested "grate" textures. Bullets/sight pass through, but solids don't
        SLIME = 0x10, //
        WATER = 0x20, //
        MIST = 0x40, //
        OPAQUE = 0x80, //	block AI line of sight
        TESTFOGVOLUME = 0x100, //things that cannot be seen through (may be non-solid though)
        UNUSED = 0x200, //unused
        UNUSED6 = 0x400, //unused
        TEAM1 = 0x800, //per team contents used to differentiate collisions between players and objects on different teams
        TEAM2 = 0x1000,
        IgnoreNodrawOpaque = 0x2000, //ignore CONTENTS_OPAQUE on surfaces that have SURF_NODRAW
        MOVEABLE = 0x4000,             //hits entities which are MOVETYPE_PUSH (doors, plats, etc.)
        AREAPORTAL = 0x8000,           //remaining contents are non-visible, and don't eat brushes
        PLAYERCLIP = 0x10000,          //
        MONSTERCLIP = 0x20000,         //
        Current0 = 0x40000,           //currents can be added to any other contents, and may be mixed
        Current90 = 0x80000,
        Current180 = 0x100000,
        Current270 = 0x200000,
        CurrentUp = 0x400000,
        CurrentDown = 0x800000,
        ORIGIN = 0x1000000,       //	removed before bsping an entity
        MONSTER = 0x2000000,      //	should never be on a brush, only in game
        DEBRIS = 0x4000000,       //
        DETAIL = 0x8000000,       //	brushes to be added after vis leafs
        TRANSLUCENT = 0x10000000, // 	auto set if any surface has trans
        LADDER = 0x20000000,      //
        HITBOX = 0x40000000,      // 	use accurate hitboxes on trace
    }
}
