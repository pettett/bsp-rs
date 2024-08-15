use bytemuck::{Pod, Zeroable};
use flagset::flags;
use num_derive::FromPrimitive;

#[derive(Copy, Clone, FromPrimitive, Debug, PartialEq)]
#[repr(i32)]
pub enum ImageFormat {
    NONE = -1,
    RGBA8888 = 0,
    ABGR8888,
    RGB888,
    BGR888,
    RGB565,
    I8,
    IA88,
    P8,
    A8,
    RGB888BLUESCREEN,
    BGR888BLUESCREEN,
    ARGB8888,
    BGRA8888,
    DXT1,
    DXT3,
    DXT5,
    BGRX8888,
    BGR565,
    BGRX5551,
    BGRA4444,
    DXT1ONEBITALPHA,
    BGRA5551,
    UV88,
    UVWQ8888,
    RGBA16161616F,
    RGBA16161616,
    UVLX8888,
}
impl ImageFormat {
    pub fn bytes_for_size(&self, width: usize, height: usize, mip_level: usize) -> usize {
        let width = width >> mip_level;
        let height = height >> mip_level;

        let block_width = width.max(4);
        let block_height = height.max(4);
        let block_count = block_width * block_height / 16;

        match self {
            ImageFormat::NONE => 0,
            ImageFormat::UVLX8888
            | ImageFormat::UVWQ8888
            | ImageFormat::BGRA8888
            | ImageFormat::ARGB8888
            | ImageFormat::RGBA8888
            | ImageFormat::ABGR8888
            | ImageFormat::BGRX8888 => width * height * 4,
            ImageFormat::RGB888BLUESCREEN
            | ImageFormat::BGR888BLUESCREEN
            | ImageFormat::RGB888
            | ImageFormat::BGR888 => width * height * 3,
            ImageFormat::I8 | ImageFormat::P8 | ImageFormat::A8 => width * height,
            ImageFormat::DXT1 => block_count * 8, // should be 1/2?
            // 4x4 block has 64bits of color and 64 bits of alpha
            ImageFormat::DXT3 | ImageFormat::DXT5 => block_count * 16,
            ImageFormat::IA88
            | ImageFormat::RGB565
            | ImageFormat::UV88
            | ImageFormat::BGRA5551
            | ImageFormat::BGRX5551
            | ImageFormat::BGR565
            | ImageFormat::BGRA4444 => width * height * 2,
            ImageFormat::DXT1ONEBITALPHA => todo!(),
            ImageFormat::RGBA16161616F | ImageFormat::RGBA16161616 => width * height * 8,
        }
    }
    // function imageFormatIsBlockCompressed(fmt: ImageFormat): boolean {
    //     if (fmt === ImageFormat.DXT1)
    //         return true;
    //     if (fmt === ImageFormat.DXT3)
    //         return true;
    //     if (fmt === ImageFormat.DXT5)
    //         return true;

    //     return false;
    // }

    // function imageFormatCalcLevelSize(fmt: ImageFormat, width: number, height: number, depth: number): number {
    //     if (imageFormatIsBlockCompressed(fmt)) {
    //         width = Math.max(width, 4);
    //         height = Math.max(height, 4);
    //         const count = ((width * height) / 16) * depth;
    //         if (fmt === ImageFormat.DXT1)
    //             return count * 8;
    //         else if (fmt === ImageFormat.DXT3)
    //             return count * 16;
    //         else if (fmt === ImageFormat.DXT5)
    //             return count * 16;
    //         else
    //             throw "whoops";
    //     } else {
    //         return (width * height * depth) * imageFormatGetBPP(fmt);
    //     }
    // }

    // function imageFormatGetBPP(fmt: ImageFormat): number {
    //     if (fmt === ImageFormat.RGBA16161616F)
    //         return 8;
    //     if (fmt === ImageFormat.RGBA8888)
    //         return 4;
    //     if (fmt === ImageFormat.ABGR8888)
    //         return 4;
    //     if (fmt === ImageFormat.ARGB8888)
    //         return 4;
    //     if (fmt === ImageFormat.BGRA8888)
    //         return 4;
    //     if (fmt === ImageFormat.BGRX8888)
    //         return 4;
    //     if (fmt === ImageFormat.RGB888)
    //         return 3;
    //     if (fmt === ImageFormat.BGR888)
    //         return 3;
    //     if (fmt === ImageFormat.BGRA5551)
    //         return 2;
    //     if (fmt === ImageFormat.UV88)
    //         return 2;
    //     if (fmt === ImageFormat.I8)
    //         return 1;
    //     throw "whoops";
    // }

    pub fn layout(&self, size: wgpu::Extent3d) -> wgpu::ImageDataLayout {
        match self {
            ImageFormat::NONE => wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: None,
                rows_per_image: None,
            },

            ImageFormat::ARGB8888
            | ImageFormat::BGRA8888
            | ImageFormat::RGBA8888
            | ImageFormat::ABGR8888
            | ImageFormat::RGB888BLUESCREEN
            | ImageFormat::BGR888BLUESCREEN
            | ImageFormat::RGB888
            | ImageFormat::BGR888 => wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width),
                rows_per_image: None,
            },
            ImageFormat::RGB565 => todo!(),
            ImageFormat::I8 => todo!(),
            ImageFormat::IA88 => todo!(),
            ImageFormat::P8 => todo!(),
            ImageFormat::A8 => todo!(),
            ImageFormat::DXT1 => wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(16 * size.width / 8),
                rows_per_image: None,
            },
            ImageFormat::DXT3 => wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(16 * size.width / 4),
                rows_per_image: None,
            },
            ImageFormat::DXT5 => wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(16 * size.width / 4),
                rows_per_image: None,
            },
            ImageFormat::BGRX8888 => todo!(),
            ImageFormat::BGR565 => todo!(),
            ImageFormat::BGRX5551 => todo!(),
            ImageFormat::BGRA4444 => todo!(),
            ImageFormat::DXT1ONEBITALPHA => todo!(),
            ImageFormat::BGRA5551 => todo!(),
            ImageFormat::UV88 => todo!(),
            ImageFormat::UVWQ8888 => todo!(),
            ImageFormat::RGBA16161616F => todo!(),
            ImageFormat::RGBA16161616 => todo!(),
            ImageFormat::UVLX8888 => todo!(),
        }
    }
}

// function imageFormatToGfxFormat(device: GfxDevice, fmt: ImageFormat, srgb: boolean): GfxFormat {
//     // TODO(jstpierre): Software decode BC1 if necessary.
//     if (fmt === ImageFormat.DXT1)
//         return srgb ? GfxFormat.BC1_SRGB : GfxFormat.BC1;
//     else if (fmt === ImageFormat.DXT3)
//         return srgb ? GfxFormat.BC2_SRGB : GfxFormat.BC2;
//     else if (fmt === ImageFormat.DXT5)
//         return srgb ? GfxFormat.BC3_SRGB : GfxFormat.BC3;
//     else if (fmt === ImageFormat.RGBA8888)
//         return srgb ? GfxFormat.U8_RGBA_SRGB : GfxFormat.U8_RGBA_NORM;
//     else if (fmt === ImageFormat.RGB888)
//         return srgb ? GfxFormat.U8_RGBA_SRGB : GfxFormat.U8_RGBA_NORM;
//     else if (fmt === ImageFormat.BGR888)
//         return srgb ? GfxFormat.U8_RGBA_SRGB : GfxFormat.U8_RGBA_NORM;
//     else if (fmt === ImageFormat.BGRA8888)
//         return srgb ? GfxFormat.U8_RGBA_SRGB : GfxFormat.U8_RGBA_NORM;
//     else if (fmt === ImageFormat.ABGR8888)
//         return srgb ? GfxFormat.U8_RGBA_SRGB : GfxFormat.U8_RGBA_NORM;
//     else if (fmt === ImageFormat.BGRX8888)
//         return srgb ? GfxFormat.U8_RGBA_SRGB : GfxFormat.U8_RGBA_NORM;
//     else if (fmt === ImageFormat.BGRA5551)
//         return GfxFormat.U16_RGBA_5551; // TODO(jstpierre): sRGB?
//     else if (fmt === ImageFormat.UV88)
//         return GfxFormat.S8_RG_NORM;
//     else if (fmt === ImageFormat.I8)
//         return GfxFormat.U8_RGBA_NORM;
//     else if (fmt === ImageFormat.RGBA16161616F)
//         return GfxFormat.F16_RGBA;
//     else
//         throw "whoops";
// }
 
impl TryFrom<ImageFormat> for wgpu::TextureFormat {
    type Error = ();

    fn try_from(value: ImageFormat) -> Result<Self, Self::Error> {
        match value {
            ImageFormat::NONE => Err(()),
            ImageFormat::RGBA8888
            | ImageFormat::ABGR8888
            | ImageFormat::RGB888
            | ImageFormat::ARGB8888
            | ImageFormat::BGRA8888
            | ImageFormat::RGB888BLUESCREEN
            | ImageFormat::BGR888BLUESCREEN
            | ImageFormat::BGRX8888
            | ImageFormat::BGR888 => Ok(wgpu::TextureFormat::Rgba8UnormSrgb),
            ImageFormat::RGB565 => todo!(),
            ImageFormat::I8 => Ok(wgpu::TextureFormat::R8Unorm),
            ImageFormat::IA88 => todo!(),
            ImageFormat::P8 => todo!(),
            ImageFormat::A8 => todo!(),
            ImageFormat::DXT1 => Ok(wgpu::TextureFormat::Bc1RgbaUnormSrgb),
            ImageFormat::DXT3 => Ok(wgpu::TextureFormat::Bc2RgbaUnormSrgb),
            ImageFormat::DXT5 => Ok(wgpu::TextureFormat::Bc3RgbaUnorm),
            ImageFormat::BGR565 => todo!(),
            ImageFormat::BGRX5551 => todo!(),
            ImageFormat::BGRA4444 => todo!(),
            ImageFormat::DXT1ONEBITALPHA => todo!(),
            ImageFormat::BGRA5551 => todo!(),
            ImageFormat::UV88 => Ok(wgpu::TextureFormat::Rg8Unorm),
            ImageFormat::UVWQ8888 => todo!(),
            ImageFormat::RGBA16161616F => todo!(),
            ImageFormat::RGBA16161616 => todo!(),
            ImageFormat::UVLX8888 => todo!(),
        }
    }
}

unsafe impl Zeroable for ImageFormat {}
unsafe impl Pod for ImageFormat {}

flags! {
    #[repr(u32)]
    pub enum CompiledVtfFlags: u32 {
        // Flags from the *.txt config file
        POINTSAMPLE = 0x00000001,
        TRILINEAR = 0x00000002,
        CLAMPS = 0x00000004,
        CLAMPT = 0x00000008,
        ANISOTROPIC = 0x00000010,
        HINTDXT5 = 0x00000020,
        PWLCORRECTED = 0x00000040,
        NORMAL = 0x00000080,
        NOMIP = 0x00000100,
        NOLOD = 0x00000200,
        ALLMIPS = 0x00000400,
        PROCEDURAL = 0x00000800,

        // These are automatically generated by vtex from the texture data.
        ONEBITALPHA = 0x00001000,
        EIGHTBITALPHA = 0x00002000,

        // Newer flags from the *.txt config file
        ENVMAP = 0x00004000,
        RENDERTARGET = 0x00008000,
        DEPTHRENDERTARGET = 0x00010000,
        NODEBUGOVERRIDE = 0x00020000,
        SINGLECOPY	= 0x00040000,
        PRESRGB = 0x00080000,

        UNUSED00100000 = 0x00100000,
        UNUSED00200000 = 0x00200000,
        UNUSED00400000 = 0x00400000,

        NODEPTHBUFFER = 0x00800000,

        UNUSED01000000 = 0x01000000,

        CLAMPU = 0x02000000,
        VERTEXTEXTURE = 0x04000000,
        SSBUMP = 0x08000000,

        UNUSED10000000 = 0x10000000,

        BORDER = 0x20000000,

        TUNUSED40000000 = 0x40000000,
        TUNUSED80000000 = 0x80000000,
    }
}
unsafe impl Zeroable for CompiledVtfFlags {}
unsafe impl Pod for CompiledVtfFlags {}
