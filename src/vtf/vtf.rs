use std::{
    fmt,
    io::{self, BufReader, Read, Seek},
    mem,
    sync::OnceLock,
};

use wgpu::{Device, Queue};

use crate::{
    binaries::BinaryData,
    vtexture::VTexture,
    vtf::header::{ResourceEntryInfo, VTFHeader, VTFHeader73},
};

use super::consts::ImageFormat;

pub type VRes<T> = Result<T, ()>;

/// Thread safe VTF file with associated texture data
pub struct VTF {
    header: VTFHeader,
    header_7_3: Option<VTFHeader73>,
    low_res_data: [Vec<u8>; 1],
    high_res_data: Vec<Vec<u8>>,
    low_res: OnceLock<VRes<VTexture>>,
    high_res: OnceLock<VRes<VTexture>>,

    low_res_imgui: OnceLock<VRes<imgui::TextureId>>,
    high_res_imgui: OnceLock<VRes<imgui::TextureId>>,
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
    pub fn new_from_header(header: VTFHeader) -> Self {
        Self {
            header,
            header_7_3: None,
            low_res_data: [vec![0; 0]],
            high_res_data: vec![vec![0; 0]; 0],
            low_res: OnceLock::new(),
            high_res: OnceLock::new(),
            low_res_imgui: OnceLock::new(),
            high_res_imgui: OnceLock::new(),
        }
    }

    pub fn set_low_res_data(&mut self, low_res_data: Vec<u8>) {
        self.low_res_data = [low_res_data];
    }

    pub fn set_high_res_data(&mut self, high_res_data: Vec<Vec<u8>>) {
        self.high_res_data = high_res_data;
    }

    pub fn width(&self) -> u32 {
        self.header.width as u32
    }
    pub fn height(&self) -> u32 {
        self.header.height as u32
    }
    pub fn header(&self) -> &VTFHeader {
        &self.header
    }
    pub fn header_7_3(&self) -> Option<&VTFHeader73> {
        self.header_7_3.as_ref()
    }
    pub fn low_res_width(&self) -> u32 {
        self.header.low_res_image_width as u32
    }
    pub fn low_res_height(&self) -> u32 {
        self.header.low_res_image_height as u32
    }

    pub fn get_high_res(&self, device: &Device, queue: &Queue) -> &VRes<VTexture> {
        self.high_res
            .get_or_init(|| self.upload_high_res(device, queue))
    }

    pub fn get_low_res(&self, device: &Device, queue: &Queue) -> &VRes<VTexture> {
        self.low_res
            .get_or_init(|| self.upload_low_res(device, queue))
    }
    pub fn get_high_res_imgui(
        &self,
        device: &Device,
        queue: &Queue,
        renderer: &mut imgui_wgpu::Renderer,
    ) -> &VRes<imgui::TextureId> {
        self.high_res_imgui
            .get_or_init(|| match self.get_high_res(device, queue) {
                Ok(high_res) => Ok(renderer
                    .textures
                    .insert(high_res.to_imgui(device, renderer))),
                Err(e) => Err(*e),
            })
    }
    pub fn get_low_res_imgui(
        &self,
        device: &Device,
        queue: &Queue,
        renderer: &mut imgui_wgpu::Renderer,
    ) -> &VRes<imgui::TextureId> {
        self.low_res_imgui
            .get_or_init(|| match self.get_low_res(device, queue) {
                Ok(low_res) => Ok(renderer.textures.insert(low_res.to_imgui(device, renderer))),
                Err(e) => Err(*e),
            })
    }

    fn upload_high_res(&self, device: &Device, queue: &Queue) -> VRes<VTexture> {
        if self.high_res_data.len() > 0 {
            Ok(self.upload(
                device,
                queue,
                self.width(),
                self.height(),
                self.header.high_res_image_format,
                &self.high_res_data[..],
            ))
        } else {
            Err(())
        }
    }

    fn upload_low_res(&self, device: &Device, queue: &Queue) -> VRes<VTexture> {
        Ok(self.upload(
            device,
            queue,
            self.low_res_width(),
            self.low_res_height(),
            self.header.low_res_image_format,
            &self.low_res_data,
        ))
    }

    fn upload(
        &self,
        device: &Device,
        queue: &Queue,
        width: u32,
        height: u32,
        format: ImageFormat,
        mipped_data: &[Vec<u8>],
    ) -> VTexture {
        let wgpu_format = format.try_into().unwrap();
        //println!("{:?} {:?}", format, wgpu_format);
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

        VTexture::new(texture, view, sampler)
    }
}
