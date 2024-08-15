use std::io::{self, BufReader, Cursor};

use bevy::{
    asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypePath,
    render::{
        render_asset::RenderAssetUsages,
        texture::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    },
};
use source::binaries::BinaryData;
use source::vtf::VTF;

#[derive(Default)]
pub struct VTFAssetLoader;

impl AssetLoader for VTFAssetLoader {
    type Asset = Image;
    type Settings = ();
    type Error = io::Error;
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
		let len = bytes.len();
        reader.read_to_end(&mut bytes).await?;

        let mut c = Cursor::new(bytes);
        let vtf = VTF::read(&mut BufReader::new(&mut c), Some(len))?;

        if vtf.high_res_data().len() > 0 {
            let image = vtf_to_image(&vtf);

            Ok(image)
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidData, "failed to load image"))
        }
    }

    fn extensions(&self) -> &[&str] {
        &["vtf"]
    }
}

fn vtf_to_image(vtf: &VTF) -> Image {
    Image {
        data: vtf.high_res_data()[0].clone(),
        texture_descriptor: vtf.descriptor_high_res(),
        asset_usage: RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
            label: None,
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            mag_filter: ImageFilterMode::Linear,
            min_filter: ImageFilterMode::Linear,
            ..Default::default()
        }),
        ..Default::default()
    }
}
