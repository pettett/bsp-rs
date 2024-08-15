use std::io;

use bevy::{asset::{io::{AssetReader, AssetReaderError, ErasedAssetReader, Reader, SliceReader}, AsyncReadExt}, tasks::futures_lite::{io::Take, AsyncRead, AsyncSeek, AsyncSeekExt}};
use common::vpath::VGlobalPath;
use ini::Ini;
use source::prelude::GameData;


pub struct VPKAssetReader {
    game_data: GameData,
    fallback_io: Box<dyn ErasedAssetReader>,
}

impl VPKAssetReader {
    pub fn new(fallback_io: Box<dyn ErasedAssetReader>) -> Self {
        let ini = Ini::load_from_file("conf.ini").unwrap();
        let game_data = GameData::from_ini(&ini);

        Self {
            game_data,
            fallback_io,
        }
    }
}

struct OuterTake<'a>(Take<Box<Reader<'a>>>);

impl<'a> AsyncRead for OuterTake<'a> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<io::Result<usize>> {
        let mut pinned = std::pin::pin!(&mut self.0);
        pinned.as_mut().poll_read(cx, buf)
    }
}

impl<'a> AsyncSeek for OuterTake<'a> {
    fn poll_seek(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        pos: io::SeekFrom,
    ) -> std::task::Poll<io::Result<u64>> {
        std::task::Poll::Ready(Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Cannot Seek",
        )))
    }
}

impl AssetReader for VPKAssetReader {
    async fn read<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> Result<Box<Reader<'a>>, AssetReaderError> {
        // self.fallback_io.read(path).await

        let p = path.to_str().unwrap();

        println!("Loading {p} reader");
        for dir in self.game_data.dirs() {
            let Ok(file_data) = dir.file_data(&VGlobalPath::new(p)) else {
                continue;
            };

            if let Some(preload) = file_data.preload() {
                // Load from preload data
                //TODO: delete preload data after

                println!("Loaded preload reader");

                return Ok(Box::new(SliceReader::new(preload)));
            } else {
                // Attempt to load

                // println!("Loading file reader");

                let index = file_data.archive();

                // open file
                let mut file =
                    self.fallback_io.read(&dir.pak_archive(index)).await? as Box<Reader<'a>>;
                file.seek(io::SeekFrom::Start(file_data.offset() as _))
                    .await?;
                file = Box::new(OuterTake(file.take(file_data.len() as _))) as Box<Reader<'a>>;

                // println!("Loaded file reader in {:?}", dir.pak_archive(index));

                return Ok(file);
            }
        }
        return Err(AssetReaderError::NotFound(path.to_owned()));
    }

    async fn read_meta<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> Result<Box<Reader<'a>>, AssetReaderError> {
        // self.fallback_io.read_meta(path).await
        Err(AssetReaderError::NotFound(path.to_owned()))
    }

    async fn read_directory<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> Result<Box<bevy::asset::io::PathStream>, AssetReaderError> {
        // self.fallback_io.read_directory(path)
        Err(AssetReaderError::NotFound(path.to_owned()))
    }

    async fn is_directory<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> Result<bool, AssetReaderError> {
        Err(AssetReaderError::NotFound(path.to_owned()))
    }
}