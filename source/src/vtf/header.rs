use super::consts::ImageFormat;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
pub struct VTFHeader {
    pub signature: [i8; 4], // File signature ("VTF\0"). (or as little-endian integer, 0x00465456)
    pub version: [u32; 2],  // version[0].version[1] (currently 7.2).
    pub header_size: u32, // Size of the header struct  (16 byte aligned, currently 80 bytes) + size of the resources dictionary (7.3+).
    pub width: u16,       // Width of the largest mipmap in pixels. Must be a power of 2.
    pub height: u16,      // Height of the largest mipmap in pixels. Must be a power of 2.
    pub flags: u32,       // VTF flags.
    pub frames: u16,      // Number of frames, if animated (1 for no animation).
    pub first_frame: u16, // First frame in animation (0 based). Can be -1 in environment maps older than 7.5, meaning there are 7 faces, not 6.
    padding0: [u8; 4],    // reflectivity padding (16 byte alignment).
    pub reflectivity: [f32; 3], // reflectivity vector.
    padding1: [u8; 4],    // reflectivity padding (8 byte packing).
    pub bumpmap_scale: f32, // Bumpmap scale.
    pub high_res_image_format: ImageFormat, // High resolution image format.
    pub mipmap_count: u8, // Number of mipmaps.
    pub low_res_image_format: ImageFormat, // Low resolution image format (always DXT1).
    pub low_res_image_width: u8, // Low resolution image width.
    pub low_res_image_height: u8, // Low resolution image height.
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
pub struct VTFHeader73 {
    // 7.2+
    pub depth: i16, // Depth of the largest mipmap in pixels. Must be a power of 2. Is 1 for a 2D texture.

    // 7.3+
    padding2: [u8; 3],      // depth padding (4 byte alignment).
    pub num_resources: u32, // Number of resources this vtf has. The max appears to be 32.

    padding3: [u8; 8], // Necessary on certain compilers
}

///Tags
///    { '\x01', '\0', '\0' } - Low-res (thumbnail) image data.
///    { '\x30', '\0', '\0' } - High-res image data.
///    { '\x10', '\0', '\0' } - Animated particle sheet data.
///    { 'C', 'R', 'C' } - CRC data.
///    { 'L', 'O', 'D' } - Texture LOD control information.
///    { 'T', 'S', 'O' } - Game-defined "extended" VTF flags.
///    { 'K', 'V', 'D' } - Arbitrary KeyValues data.
#[repr(C)]
#[derive(Copy, Clone, Default, Debug, bytemuck::Zeroable)]
pub struct ResourceEntryInfo {
    pub tag: [u8; 3], // A three-byte "tag" that identifies what this resource is.
    pub flags: u8, // Resource entry flags. The only known flag is 0x2, which indicates that no data chunk corresponds to this resource.
    pub offset: u32, // The offset of this resource's data in the file.
}
