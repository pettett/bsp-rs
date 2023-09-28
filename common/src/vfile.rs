use std::{
    io::{BufReader, Cursor, Seek},
    path::{Path, PathBuf},
    sync::Arc,
};

use bevy_ecs::system::Resource;
use std::collections::HashMap;

#[derive(Default)]
pub struct VFile {
    pub data: Vec<u8>,
}

#[derive(Resource, Default, Clone)]
pub struct VFileSystem {
    pub files: Arc<HashMap<String, VFile>>,
}

impl VFileSystem {
    pub fn get(&self, path: &Path) -> Option<BufReader<Cursor<&[u8]>>> {
        self.get_str(path.to_str().unwrap())
    }

    pub fn get_str(&self, path: &str) -> Option<BufReader<Cursor<&[u8]>>> {
        match self.files.get(path) {
            Some(file) => {
                let c = Cursor::new(&file.data[..]);

                Some(BufReader::new(c))
            }
            None => {
                log::error!("{:?} file not found", path);
                None
            }
        }
    }
}
