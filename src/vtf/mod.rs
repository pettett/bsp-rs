// // Valve Texture File

use std::{cell::OnceCell, fmt, io::Read, mem};


use wgpu::{Device, Queue};

use crate::{binaries::BinaryData, texture::Texture};

use self::consts::ImageFormat;

pub struct VTF {
    header: VTFHeader,
    header_7_3: Option<VTFHeader73>,
    low_res_data: [Vec<u8>; 1],
    high_res_data: Vec<Vec<u8>>,
    low_res: OnceCell<Texture>,
    high_res: OnceCell<Texture>,

    low_res_imgui: OnceCell<imgui::TextureId>,
    high_res_imgui: OnceCell<imgui::TextureId>,
}

impl fmt::Debug for VTF {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, ".vtf: {:?}", self.header)?;
        write!(f, "7.3 data: {:?}", self.header_7_3)
    }
}

impl VTF {
    //TODO: Currently 7.3 data is not parsed
    pub fn ready_for_use(&self) -> bool {
        self.header_7_3.is_none()
    }
    pub fn width(&self) -> u32 {
        self.header.width as u32
    }
    pub fn height(&self) -> u32 {
        self.header.height as u32
    }

    pub fn low_res_width(&self) -> u32 {
        self.header.low_res_image_width as u32
    }
    pub fn low_res_height(&self) -> u32 {
        self.header.low_res_image_height as u32
    }

    pub fn get_high_res(&self, device: &Device, queue: &Queue) -> &Texture {
        self.high_res
            .get_or_init(|| self.upload_high_res(device, queue))
    }

    pub fn get_low_res(&self, device: &Device, queue: &Queue) -> &Texture {
        self.low_res
            .get_or_init(|| self.upload_low_res(device, queue))
    }
    pub fn get_high_res_imgui(
        &self,
        device: &Device,
        queue: &Queue,
        renderer: &mut imgui_wgpu::Renderer,
    ) -> &imgui::TextureId {
        self.high_res_imgui.get_or_init(|| {
            renderer
                .textures
                .insert(self.get_high_res(device, queue).to_imgui(device, renderer))
        })
    }
    pub fn get_low_res_imgui(
        &self,
        device: &Device,
        queue: &Queue,
        renderer: &mut imgui_wgpu::Renderer,
    ) -> &imgui::TextureId {
        self.low_res_imgui.get_or_init(|| {
            renderer
                .textures
                .insert(self.get_low_res(device, queue).to_imgui(device, renderer))
        })
    }

    fn upload_high_res(&self, device: &Device, queue: &Queue) -> Texture {
        self.upload(
            device,
            queue,
            self.width(),
            self.height(),
            self.header.high_res_image_format,
            &self.high_res_data[..],
        )
    }

    fn upload_low_res(&self, device: &Device, queue: &Queue) -> Texture {
        self.upload(
            device,
            queue,
            self.low_res_width(),
            self.low_res_height(),
            self.header.low_res_image_format,
            &self.low_res_data,
        )
    }

    fn upload(
        &self,
        device: &Device,
        queue: &Queue,
        width: u32,
        height: u32,
        format: ImageFormat,
        mipped_data: &[Vec<u8>],
    ) -> Texture {
        let wgpu_format = format.try_into().unwrap();
        println!("{:?} {:?}", format, wgpu_format);
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            // All textures are stored as 3D, we represent our 2D texture
            // by setting depth to 1.
            size,
            mip_level_count: mipped_data.len() as u32,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Most images are stored using sRGB so we need to reflect that here.
            format: wgpu_format,
            // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
            // COPY_DST means that we want to copy data to this texture
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            // This is the same as with the SurfaceConfig. It
            // specifies what texture formats can be used to
            // create TextureViews for this texture. The base
            // texture format (Rgba8UnormSrgb in this case) is
            // always supported. Note that using a different
            // texture format is not supported on the WebGL2
            // backend.
            view_formats: &[],
        });
        for mip_level in 0..mipped_data.len() {
            let mip_size = wgpu::Extent3d {
                width: width >> mip_level,
                height: height >> mip_level,
                depth_or_array_layers: 1,
            };

            queue.write_texture(
                // Tells wgpu where to copy the pixel data
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: mip_level as u32,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                // The actual pixel data
                &mipped_data[mip_level],
                // The layout of the texture
                format.layout(mip_size),
                mip_size,
            );
        }
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Texture::new(texture, view, sampler)
    }
}

impl BinaryData for VTF {
    fn read(buffer: &mut std::io::BufReader<std::fs::File>) -> std::io::Result<Self> {
        let mut header_read = 0;

        let header = VTFHeader::read(buffer)?;
        let header_size = header.header_size as i64;
        header_read += mem::size_of::<VTFHeader>() as i64;

        //println!("Header size, {} used: {}", header_size, header_read);

        assert!(header.width < 4096);
        assert!(header.height < 4096);

        let mut low_res_data = Vec::new();
        let mut high_res_data: Vec<Vec<u8>> = Vec::new();
        let mut header_7_3 = None;
        if header.version[0] == 7 && header.version[1] == 3 {
            let h_7_3 = VTFHeader73::read(buffer)?;

            header_7_3 = Some(h_7_3);

            //println!("Loading entries");
            for _i in 0..h_7_3.num_resources {
                if let Ok(entry) = ResourceEntryInfo::read(buffer) {
                    match entry.tag {
                        [b'\x01', b'\0', b'\0'] => (), //  Low-res (thumbnail) image data
                        [b'\x30', b'\0', b'\0'] => (), //- High-res image data.
                        [b'\x10', b'\0', b'\0'] => (), //- Animated particle sheet data.
                        [b'C', b'R', b'C'] => (),      //- CRC data.
                        [b'L', b'O', b'D'] => (),      //- Texture LOD control information.
                        [b'T', b'S', b'O'] => (),      //- Game-defined "extended" VTF flags.
                        [b'K', b'V', b'D'] => (),      //- Arbitrary KeyValues data.
                        _ => {
                            break;
                        }
                    };
                } else {
                    break;
                }
            }
        } else {
            //println!("{:?}", header);

            let lowResImageFormat = header.low_res_image_format;
            let highResImageFormat = header.high_res_image_format;
            // load data
            let low_res_size = lowResImageFormat.bytes_for_size(
                header.low_res_image_width as usize,
                header.low_res_image_height as usize,
                0,
            );

            //TODO: There appears to be one byte of something
            let remaining_header = header_size - header_read;
            if remaining_header > 0 {
                log::warn!(
                    "Not all header has been read, skipping {} bytes",
                    remaining_header
                );
                buffer.seek_relative(remaining_header)?;
            }
            low_res_data = vec![0; low_res_size];

            let wanted_mips = (header.mipmap_count as usize).max(7) - 6;

            high_res_data = vec![Vec::new(); wanted_mips];
            buffer.read_exact(&mut low_res_data[..])?;

            // seek forward through mip maps we don't want
            let mut offset = 0;
            for mip_level in wanted_mips..header.mipmap_count as usize {
                offset += highResImageFormat.bytes_for_size(
                    header.width as usize,
                    header.height as usize,
                    mip_level,
                ) as i64;
            }
            buffer.seek_relative(offset)?;
            // have to operate in reverse to load correct data
            for mip_level in (0..wanted_mips as usize).rev() {
                high_res_data[mip_level] = vec![
                    0;
                    highResImageFormat.bytes_for_size(
                        header.width as usize,
                        header.height as usize,
                        mip_level,
                    )
                ];

                buffer.read_exact(&mut high_res_data[mip_level][..])?;

                println!(
                    "loaded mipmap level {} {}",
                    mip_level,
                    high_res_data[mip_level].len()
                );

                // Do things like add empty alpha channels
                image_format_convert_data(
                    highResImageFormat,
                    &mut high_res_data[mip_level],
                    header.width as usize >> mip_level,
                    header.height as usize >> mip_level,
                    header.frames as usize,
                );
            }
            //low res is always going to be dtx1
            assert_eq!(lowResImageFormat, ImageFormat::DXT1);
        }
        Ok(Self {
            header,
            header_7_3,
            low_res_data: [low_res_data],
            high_res_data,
            low_res: OnceCell::new(),
            high_res: OnceCell::new(),
            low_res_imgui: OnceCell::new(),
            high_res_imgui: OnceCell::new(),
        })
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
struct VTFHeader {
    signature: [i8; 4], // File signature ("VTF\0"). (or as little-endian integer, 0x00465456)
    version: [u32; 2],  // version[0].version[1] (currently 7.2).
    header_size: u32, // Size of the header struct  (16 byte aligned, currently 80 bytes) + size of the resources dictionary (7.3+).
    width: u16,       // Width of the largest mipmap in pixels. Must be a power of 2.
    height: u16,      // Height of the largest mipmap in pixels. Must be a power of 2.
    flags: u32,       // VTF flags.
    frames: u16,      // Number of frames, if animated (1 for no animation).
    first_frame: u16, // First frame in animation (0 based). Can be -1 in environment maps older than 7.5, meaning there are 7 faces, not 6.
    padding0: [u8; 4], // reflectivity padding (16 byte alignment).
    reflectivity: [f32; 3], // reflectivity vector.
    padding1: [u8; 4], // reflectivity padding (8 byte packing).
    bumpmap_scale: f32, // Bumpmap scale.
    high_res_image_format: ImageFormat, // High resolution image format.
    mipmap_count: u8, // Number of mipmaps.
    low_res_image_format: ImageFormat, // Low resolution image format (always DXT1).
    low_res_image_width: u8, // Low resolution image width.
    low_res_image_height: u8, // Low resolution image height.
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
struct VTFHeader73 {
    // 7.2+
    depth: i16, // Depth of the largest mipmap in pixels. Must be a power of 2. Is 1 for a 2D texture.

    // 7.3+
    padding2: [u8; 3],  // depth padding (4 byte alignment).
    num_resources: u32, // Number of resources this vtf has. The max appears to be 32.

    padding3: [u8; 8], // Necessary on certain compilers
}

impl BinaryData for VTFHeader {}
impl BinaryData for VTFHeader73 {}

///Tags
///    { '\x01', '\0', '\0' } - Low-res (thumbnail) image data.
///    { '\x30', '\0', '\0' } - High-res image data.
///    { '\x10', '\0', '\0' } - Animated particle sheet data.
///    { 'C', 'R', 'C' } - CRC data.
///    { 'L', 'O', 'D' } - Texture LOD control information.
///    { 'T', 'S', 'O' } - Game-defined "extended" VTF flags.
///    { 'K', 'V', 'D' } - Arbitrary KeyValues data.
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
struct ResourceEntryInfo {
    tag: [u8; 3], // A three-byte "tag" that identifies what this resource is.
    flags: u8, // Resource entry flags. The only known flag is 0x2, which indicates that no data chunk corresponds to this resource.
    offset: u32, // The offset of this resource's data in the file.
}

impl BinaryData for ResourceEntryInfo {}

mod vtf_tests {
    

    

    

    const PATH: &str =
        "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_textures_dir.vpk";

    #[test]
    fn test_load() {
        let dir = VPKDirectory::load(PathBuf::from(PATH)).unwrap();
        for file in dir.get_file_names() {
            if file.contains(".vtf") {
                let data = dir.load_vtf(file).unwrap();
                let lr = data.header.low_res_image_format;
                let hr = data.header.high_res_image_format;
                if hr != ImageFormat::DXT5 && hr != ImageFormat::DXT1 {
                    println!("{} {:?}", file, hr);
                }
            }
        }
    }
    #[test]
    fn test_load_materials_metal_metalfence001a() {
        let dir = VPKDirectory::load(PathBuf::from(PATH)).unwrap();

        let data = dir.load_vtf("materials/metal/metalfence001a.vtf").unwrap();
        println!("{:?}", data.header);
    }
}

// import ArrayBufferSlice from "../ArrayBufferSlice.js";
// import { GfxTexture, GfxDevice, GfxFormat, GfxSampler, GfxWrapMode, GfxTexFilterMode, GfxMipFilterMode, GfxTextureDescriptor, GfxTextureDimension, GfxTextureUsage } from "../gfx/platform/GfxPlatform.js";
// import { readString, assert, nArray, assertExists } from "../util.js";
// import { TextureMapping } from "../TextureHolder.js";
// import { GfxRenderCache } from "../gfx/render/GfxRenderCache.js";

// const enum ImageFormat {
//     RGBA8888      = 0x00,
//     ABGR8888      = 0x01,
//     RGB888        = 0x02,
//     BGR888        = 0x03,
//     I8            = 0x05,
//     ARGB8888      = 0x0B,
//     BGRA8888      = 0x0C,
//     DXT1          = 0x0D,
//     DXT3          = 0x0E,
//     DXT5          = 0x0F,
//     BGRX8888      = 0x10,
//     BGRA5551      = 0x15,
//     UV88          = 0x16,
//     RGBA16161616F = 0x18,
// }

fn image_format_convert_data(
    fmt: ImageFormat,
    data: &mut Vec<u8>,
    width: usize,
    height: usize,
    depth: usize,
) {
    let n = width * height * depth * 4;
    if fmt == ImageFormat::BGR888 {
        // BGR888 => RGBA8888
        let mut dst = vec![0; n];
        let mut p = 0;

        //ensure there is enough data: 4 to 3 ratio
        assert!(data.len() * 4 >= n * 3);

        for i in (0..n).step_by(4) {
            dst[i + 0] = data[p + 2]; //red
            dst[i + 1] = data[p + 1]; //green
            dst[i + 2] = data[p + 0]; //blue
            dst[i + 3] = 255;
            p += 3;
        }
        *data = dst;
    } else if fmt == ImageFormat::RGB888 {
        // RGB888 => RGBA8888
        let mut dst = vec![0; n];
        let mut p = 0;
        for i in (0..n).step_by(4) {
            dst[i + 0] = data[p + 0];
            dst[i + 1] = data[p + 1];
            dst[i + 2] = data[p + 2];
            dst[i + 3] = 255;
            p += 3;
        }
        *data = dst;
    } else if fmt == ImageFormat::ABGR8888 {
        // ABGR8888 => RGBA8888
        let mut dst = vec![0; n];
        for i in (0..n).step_by(4) {
            dst[i + 0] = data[i + 3];
            dst[i + 1] = data[i + 2];
            dst[i + 2] = data[i + 1];
            dst[i + 3] = data[i + 0];
        }
        *data = dst;
    } else if fmt == ImageFormat::BGRA8888 {
        // BGRA8888 => RGBA8888
        let mut dst = vec![0; n];
        for i in (0..n).step_by(4) {
            dst[i + 0] = data[i + 2];
            dst[i + 1] = data[i + 1];
            dst[i + 2] = data[i + 0];
            dst[i + 3] = data[i + 3];
        }
        *data = dst;
    } else if fmt == ImageFormat::BGRX8888 {
        // BGRX8888 => RGBA8888
        let mut dst = vec![0; n];
        let mut p = 0;
        for i in (0..n).step_by(4) {
            dst[i + 0] = data[p + 2];
            dst[i + 1] = data[p + 1];
            dst[i + 2] = data[p + 0];
            dst[i + 3] = 0xFF;
            p += 3;
        }
        *data = dst;
    } else if fmt == ImageFormat::I8 {
        // I8 => RGBA8888
        let mut dst = vec![0; n as usize];
        for i in (0..n).step_by(4) {
            let m = data[i / 4];
            dst[i + 0] = m;
            dst[i + 1] = m;
            dst[i + 2] = m;
            dst[i + 3] = 0xFF;
        }
        *data = dst;
    }
}

// export const enum VTFFlags {
//     NONE          = 0,
//     POINTSAMPLE   = 1 << 0,
//     TRILINEAR     = 1 << 1,
//     CLAMPS        = 1 << 2,
//     CLAMPT        = 1 << 3,
//     SRGB          = 1 << 6,
//     NOMIP         = 1 << 8,
//     ONEBITALPHA   = 1 << 12,
//     EIGHTBITALPHA = 1 << 13,
//     ENVMAP        = 1 << 14,
// }

// interface VTFResourceEntry {
//     rsrcID: number;
//     data: ArrayBufferSlice;
// }

// export class VTF {
//     public gfxTextures: GfxTexture[] = [];
//     public gfxSampler: GfxSampler | null = null;

//     public format: ImageFormat;
//     public flags: VTFFlags = VTFFlags.NONE;
//     public width: number = 0;
//     public height: number = 0;
//     public depth: number = 1;
//     public numFrames: number = 1;
//     public numLevels: number = 1;

//     public resources: VTFResourceEntry[] = [];

//     private versionMajor: number;
//     private versionMinor: number;

//     constructor(device: GfxDevice, cache: GfxRenderCache, buffer: ArrayBufferSlice | null, private name: string, srgb: boolean, public lateBinding: string | null = null) {
//         if (buffer == null)
//             return;

//         const view = buffer.createDataView();

//         assert(readString(buffer, 0x00, 0x04, false) == 'VTF\0');
//         this.versionMajor = view.getUint32(0x04, true);
//         assert(this.versionMajor == 7);
//         this.versionMinor = view.getUint32(0x08, true);
//         assert(this.versionMinor >= 0 && this.versionMinor <= 5);
//         const headerSize = view.getUint32(0x0C, true);

//         let dataIdx: number;
//         let imageDataIdx: number = 0;

//         if (this.versionMajor == 0x07) {
//             assert(this.versionMinor >= 0x00);

//             this.width = view.getUint16(0x10, true);
//             this.height = view.getUint16(0x12, true);
//             this.flags = view.getUint32(0x14, true);
//             this.numFrames = view.getUint16(0x18, true);
//             const startFrame = view.getUint16(0x1A, true);
//             const reflectivityR = view.getFloat32(0x20, true);
//             const reflectivityG = view.getFloat32(0x24, true);
//             const reflectivityB = view.getFloat32(0x28, true);
//             const bumpScale = view.getFloat32(0x30, true);
//             this.format = view.getUint32(0x34, true);
//             this.numLevels = view[0x38);
//             const lowresImageFormat = view.getUint32(0x39, true);
//             const lowresImageWidth = view[0x3D);
//             const lowresImageHeight = view[0x3E);

//             dataIdx = 0x40;

//             if (this.versionMinor >= 0x02) {
//                 this.depth = Math.max(view.getUint16(0x41, true), 1);
//                 dataIdx = 0x50;
//             } else {
//                 this.depth = 1;
//             }

//             const numResources = this.versionMinor >= 0x03 ? view.getUint32(0x44, true) : 0;
//             if (numResources > 0) {
//                 for (let i = 0; i < numResources; i++, dataIdx += 0x08) {
//                     const rsrcHeader = view.getUint32(dataIdx + 0x00, false);
//                     const rsrcID = (rsrcHeader & 0xFFFFFF00);
//                     const rsrcFlag = (rsrcHeader & 0x000000FF);
//                     const dataOffs = view.getUint32(dataIdx + 0x04, true);

//                     // RSRCFHAS_NO_DATA_CHUNK
//                     if (rsrcFlag == 0x02)
//                         continue;

//                     // Legacy resources don't have a size tag.

//                     if (rsrcID == 0x01000000) { // VTF_LEGACY_RSRC_LOW_RES_IMAGE
//                         // Skip.
//                         continue;
//                     }

//                     if (rsrcID == 0x30000000) { // VTF_LEGACY_RSRC_IMAGE
//                         imageDataIdx = dataOffs;
//                         continue;
//                     }

//                     const dataSize = view.getUint32(dataOffs + 0x00, true);
//                     const data = buffer.subarray(dataOffs + 0x04, dataSize);
//                     this.resources.push({ rsrcID, data });
//                 }
//             } else {
//                 if (lowresImageFormat !== 0xFFFFFFFF) {
//                     const lowresDataSize = imageFormatCalcLevelSize(lowresImageFormat, lowresImageWidth, lowresImageHeight, 1);
//                     const lowresData = buffer.subarray(dataIdx, lowresDataSize);
//                     dataIdx += lowresDataSize;
//                 }

//                 imageDataIdx = dataIdx;
//             }
//         } else {
//             throw "whoops";
//         }

//         const isCube = !!(this.flags & VTFFlags.ENVMAP);
//         // The srgb flag in the file does nothing :/, we have to know from the material system instead.
//         // const srgb = !!(this.flags & VTFFlags.SRGB);
//         const pixelFormat = imageFormatToGfxFormat(device, this.format, srgb);
//         const dimension = isCube ? GfxTextureDimension.Cube : GfxTextureDimension.n2D;
//         const faceCount = (isCube ? 6 : 1);
//         const hasSpheremap = this.versionMinor < 5;
//         const faceDataCount = (isCube ? (6 + (hasSpheremap ? 1 : 0)) : 1);
//         const descriptor: GfxTextureDescriptor = {
//             dimension, pixelFormat,
//             width: this.width,
//             height: this.height,
//             numLevels: this.numLevels,
//             depth: this.depth * faceCount,
//             usage: GfxTextureUsage.Sampled,
//         };

//         for (let i = 0; i < this.numFrames; i++) {
//             const texture = device.createTexture(descriptor);
//             device.setResourceName(texture, `${this.name} frame ${i}`);
//             this.gfxTextures.push(texture);
//         }

//         const levelDatas: ArrayBufferView[][] = nArray(this.gfxTextures.length, () => []);

//         // Mipmaps are stored from smallest to largest.
//         for (let i = this.numLevels - 1; i >= 0; i--) {
//             const mipWidth = Math.max(this.width >>> i, 1);
//             const mipHeight = Math.max(this.height >>> i, 1);
//             const faceSize = this.calcMipSize(i);
//             const size = faceSize * faceCount;
//             for (let j = 0; j < this.gfxTextures.length; j++) {
//                 const levelData = imageFormatConvertData(device, this.format, buffer.subarray(imageDataIdx, size), mipWidth, mipHeight, this.depth * faceCount);
//                 imageDataIdx += faceSize * faceDataCount;
//                 levelDatas[j].unshift(levelData);
//             }
//         }

//         for (let i = 0; i < this.gfxTextures.length; i++)
//             device.uploadTextureData(this.gfxTextures[i], 0, levelDatas[i]);

//         const wrapS = !!(this.flags & VTFFlags.CLAMPS) ? GfxWrapMode.Clamp : GfxWrapMode.Repeat;
//         const wrapT = !!(this.flags & VTFFlags.CLAMPT) ? GfxWrapMode.Clamp : GfxWrapMode.Repeat;

//         const texFilter = !!(this.flags & VTFFlags.POINTSAMPLE) ? GfxTexFilterMode.Point : GfxTexFilterMode.Bilinear;
//         const minFilter = texFilter;
//         const magFilter = texFilter;
//         const nomip = !!(this.flags & VTFFlags.NOMIP);
//         const maxLOD = nomip ? 0 : undefined;
//         const forceTrilinear = true;
//         const mipFilter = (!nomip && (forceTrilinear || !!(this.flags & VTFFlags.TRILINEAR))) ? GfxMipFilterMode.Linear : GfxMipFilterMode.Nearest;

//         const canSupportAnisotropy = texFilter == GfxTexFilterMode.Bilinear && mipFilter == GfxMipFilterMode.Linear;
//         const maxAnisotropy = canSupportAnisotropy ? 16 : 1;
//         this.gfxSampler = cache.createSampler({
//             wrapS, wrapT, minFilter, magFilter, mipFilter,
//             minLOD: 0, maxLOD, maxAnisotropy,
//         });
//     }

//     private calcMipSize(i: number, depth: number = this.depth): number {
//         const mipWidth = Math.max(this.width >>> i, 1);
//         const mipHeight = Math.max(this.height >>> i, 1);
//         const mipDepth = Math.max(depth >>> i, 1);
//         return imageFormatCalcLevelSize(this.format, mipWidth, mipHeight, mipDepth);
//     }

//     public fillTextureMapping(m: TextureMapping, frame: number = 0): void {
//         if (this.gfxTextures.length == 0) {
//             m.gfxTexture = null;
//         } else {
//             if (frame < 0 || frame >= this.gfxTextures.length)
//                 frame = 0;
//             m.gfxTexture = assertExists(this.gfxTextures[frame]);
//         }
//         m.gfxSampler = this.gfxSampler;
//         m.width = this.width;
//         m.height = this.height;
//         m.lateBinding = this.lateBinding;
//     }

//     public isTranslucent(): boolean {
//         return !!(this.flags & (VTFFlags.ONEBITALPHA | VTFFlags.EIGHTBITALPHA));
//     }

//     public destroy(device: GfxDevice): void {
//         for (let i = 0; i < this.gfxTextures.length; i++)
//             device.destroyTexture(this.gfxTextures[i]);
//         this.gfxTextures.length = 0;
//     }
// }
pub mod consts;
