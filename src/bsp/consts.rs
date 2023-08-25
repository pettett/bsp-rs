use flagset::{flags, FlagSet};
use num_derive::FromPrimitive;

pub const MAX_MAP_PLANES: usize = 65536;
pub const HEADER_LUMPS: usize = 64;
pub const MAX_MAP_VERTS: usize = 65536;
pub const MAX_MAP_EDGES: usize = 256000;
pub const MAX_MAP_SURFEDGES: usize = 512000;
pub const MAX_MAP_FACES: usize = 65536;

#[derive(Copy, Clone, FromPrimitive, Debug)]
pub enum LumpType {
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
    ORIGINAL_FACES = 27,
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
        IGNORE_NODRAW_OPAQUE = 0x2000, //ignore CONTENTS_OPAQUE on surfaces that have SURF_NODRAW
        MOVEABLE = 0x4000,             //hits entities which are MOVETYPE_PUSH (doors, plats, etc.)
        AREAPORTAL = 0x8000,           //remaining contents are non-visible, and don't eat brushes
        PLAYERCLIP = 0x10000,          //
        MONSTERCLIP = 0x20000,         //
        CURRENT_0 = 0x40000,           //currents can be added to any other contents, and may be mixed
        CURRENT_90 = 0x80000,
        CURRENT_180 = 0x100000,
        CURRENT_270 = 0x200000,
        CURRENT_UP = 0x400000,
        CURRENT_DOWN = 0x800000,
        ORIGIN = 0x1000000,       //	removed before bsping an entity
        MONSTER = 0x2000000,      //	should never be on a brush, only in game
        DEBRIS = 0x4000000,       //
        DETAIL = 0x8000000,       //	brushes to be added after vis leafs
        TRANSLUCENT = 0x10000000, // 	auto set if any surface has trans
        LADDER = 0x20000000,      //
        HITBOX = 0x40000000,      // 	use accurate hitboxes on trace
    }
}
