use std::{
    io::{self, BufReader, Read, Seek},
    mem,
};

use crate::{
    binaries::BinaryData,
    vtf::{
        consts::ImageFormat,
        header::{ResourceEntryInfo, VTFHeader, VTFHeader73},
        VTF,
    },
};

impl BinaryData for VTF {
    fn read<R: Read + Seek>(
        buffer: &mut std::io::BufReader<R>,
        _max_size: Option<usize>,
    ) -> std::io::Result<Self> {
        let mut data_read = 0;

        let header = VTFHeader::read(buffer, None)?;
        let header_size = header.header_size as i64;
        data_read += mem::size_of::<VTFHeader>() as i64;

        //println!("Header size, {} used: {}", header_size, header_read);

        assert!(header.width < 4096);
        assert!(header.height < 4096);

        let major = header.version[0];
        let minor = header.version[1];

        if major == 7 && minor == 3 {
            let h_7_3 = VTFHeader73::read(buffer, None)?;
            data_read += mem::size_of::<VTFHeader73>() as i64;

            let mut entries = Vec::<ResourceEntryInfo>::new();
            entries.reserve(h_7_3.num_resources as usize);

            for _ in 0..h_7_3.num_resources as usize {
                entries.push(ResourceEntryInfo::read(buffer, None)?);
                data_read += mem::size_of::<ResourceEntryInfo>() as i64;
            }

            //println!("{:?}", entries);
            let remaining_header = header_size - data_read;
            if remaining_header > 0 {
                log::warn!(
                    "Not all header has been read, skipping {} bytes",
                    remaining_header
                );
                buffer.seek_relative(remaining_header)?;
                data_read += remaining_header;
            }

            println!("{}", remaining_header);

            let mut tex = Self::new_from_header(header);

            assert_eq!(header_size as u32, entries[0].offset);

            println!("Loading entries");
            for entry in entries {
                if entry.flags & 2 > 0 {
                    //no data
                    continue;
                }

                let dist = entry.offset as i64 - data_read;
                buffer.seek_relative(dist)?;

                match entry.tag {
                    [b'\x01', b'\0', b'\0'] => {
                        //  Low-res (thumbnail) image data
                        let low_res = read_low_res(&header, buffer)?;
                        data_read += low_res.len() as i64;
                        tex.set_low_res_data(low_res);
                    }
                    [b'\x30', b'\0', b'\0'] => {
                        //- High-res image data.
                        // For some reason there is a pretty big gap here on some textures
                        tex.set_high_res_data(read_high_res(&header, buffer)?);
                        // In theory we dont need to record data read, as this is last block
                    }
                    [b'\x10', b'\0', b'\0'] => (), //- Animated particle sheet data.
                    [b'C', b'R', b'C'] => (),      //- CRC data.
                    [b'L', b'O', b'D'] => (),      //- Texture LOD control information.
                    [b'T', b'S', b'O'] => (),      //- Game-defined "extended" VTF flags.
                    [b'K', b'V', b'D'] => (),      //- Arbitrary KeyValues data.
                    _ => {
                        panic!("Error reading entity")
                    }
                };
            }

            Ok(tex)
        } else {
            // load data
            if minor == 1 {
                let mut b = [0];
                buffer.read_exact(&mut b)?;
				data_read += 1;
            }

            //TODO: There appears to be one byte of something
            let remaining_header = header_size - data_read;
            if remaining_header > 0 {
                log::warn!(
                    "[{}.{}] Not all header has been read, skipping {} bytes",
                    major,
                    minor,
                    remaining_header
                );
                buffer.seek_relative(remaining_header)?;
            } 
			
            let low_res_data: Vec<u8> = read_low_res(&header, buffer)?;
            let high_res_data = read_high_res(&header, buffer)?;

            let mut img = Self::new_from_header(header);
            img.set_high_res_data(high_res_data);
            img.set_low_res_data(low_res_data);

            Ok(img)
        }
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

fn read_low_res<R: Read + Seek>(
    header: &VTFHeader,
    buffer: &mut BufReader<R>,
) -> io::Result<Vec<u8>> {
    let low_res_image_format = header.low_res_image_format;
    // load data
    let low_res_size = low_res_image_format.bytes_for_size(
        header.low_res_image_width as usize,
        header.low_res_image_height as usize,
        0,
    );

    let mut low_res_data = vec![0; low_res_size];

    buffer.read_exact(&mut low_res_data[..])?;
    Ok(low_res_data)
}

fn read_high_res<R: Read + Seek>(
    header: &VTFHeader,
    buffer: &mut BufReader<R>,
) -> io::Result<Vec<Vec<u8>>> {
    let smallest_size = header.width.min(header.height);

    // smallest texture is a 4x4
    let smallest_mip = ((smallest_size as f32).log2().ceil() - 1.) as usize;

    let wanted_mips = (header.mipmap_count as usize).min(smallest_mip);

    let high_res_image_format = header.high_res_image_format;
    let mut high_res_data = vec![Vec::new(); wanted_mips];

    // seek forward through mip maps we don't want
    let mut offset = 0;
    for mip_level in wanted_mips..header.mipmap_count as usize {
        offset += high_res_image_format.bytes_for_size(
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
            high_res_image_format.bytes_for_size(
                header.width as usize,
                header.height as usize,
                mip_level,
            )
        ];

        buffer.read_exact(&mut high_res_data[mip_level][..])?;

        // Do things like add empty alpha channels
        image_format_convert_data(
            high_res_image_format,
            &mut high_res_data[mip_level],
            header.width as usize >> mip_level,
            header.height as usize >> mip_level,
            header.frames as usize,
        );
    }
    Ok(high_res_data)
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
