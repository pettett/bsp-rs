use std::{
    io::{self, BufReader, Cursor},
    path::PathBuf,
};

use bevy::{
    asset::{
        io::{AssetSourceId, Reader},
        Asset, AssetLoader, AssetLoaderError, AssetPath, AsyncReadExt, LoadContext,
    },
    prelude::*,
    reflect::TypePath,
};
use source::{
    binaries::BinaryData,
    vmt::{VMTError, VMT},
};
use thiserror::Error;

#[derive(Default)]
pub struct VMTAssetLoader;

// #[derive(Asset, TypePath)]
// pub struct VMTAsset {
//     pub vmt: VMT,
//     pub base_texture: Option<Handle<Image>>,
// }

impl AssetLoader for VMTAssetLoader {
    type Asset = StandardMaterial;
    type Settings = ();
    type Error = VMTError;
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();

		reader.read_to_end(&mut bytes).await.unwrap();

        let data = String::from_utf8(bytes).unwrap();
 
        let vmt = VMT::from_string(data)?;

        // vmt.load_dependants(load_context.loader().load(path));

        let tex = "$basetexture";

        match vmt.get(tex) {
            Some(tex_path) => {
                let tex_path = tex_path.replace('\\', "/");

                let vtf_path = PathBuf::from(format!("materials/{}.vtf", tex_path));
                let source = AssetSourceId::from("vpk");
                let asset_path = AssetPath::from_path(&vtf_path).with_source(source);

                println!("Loading dependency image from {}", asset_path);

                Ok(StandardMaterial {
                    base_color_texture: Some(load_context.load(asset_path)),
                    ..default()
                })
                // None
            }
            None => Ok(StandardMaterial { ..default() }),
        }
    }

    fn extensions(&self) -> &[&str] {
        &["vmt"]
    }
}
