use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use bevy_ecs::system::Resource;

use crate::{util::v_path::VPath, vmt::VMT, vpk::VPKDirectory, vtf::VTF};
#[derive(Resource)]
pub struct GameData {
    path: PathBuf,
    maps: PathBuf,
    starter_map: &'static Path,
    dirs: Vec<Arc<VPKDirectory>>,
}

pub enum Game {
    HalfLife2,
    Portal,
    Portal2,
}
impl GameData {
    pub fn load_vmt(&self, path: &dyn VPath) -> Option<&Arc<VMT>> {
        for d in &self.dirs {
            if let Ok(Some(vmt)) = d.load_vmt(path) {
                return Some(vmt);
            }
        }
        None
    }

    pub fn load_vtf(&self, path: &dyn VPath) -> Option<&Arc<VTF>> {
        for d in &self.dirs {
            if let Ok(Some(vtf)) = d.load_vtf(path) {
                return Some(vtf);
            }
        }
        None
    }
    pub fn load_game(game: Game, path: PathBuf) -> Self {
        match game {
            Game::HalfLife2 => Self {
                dirs: vec![
                    Arc::new(VPKDirectory::load(path.join("hl2\\hl2_textures_dir.vpk")).unwrap()),
                    Arc::new(VPKDirectory::load(path.join("hl2\\hl2_misc_dir.vpk")).unwrap()),
                ],
                starter_map: Path::new("d1_trainstation_02.bsp"),
                maps: path.join("hl2\\maps"),
                path,
            },
            Game::Portal => Self {
                dirs: vec![
                    Arc::new(VPKDirectory::load(path.join("hl2\\hl2_textures_dir.vpk")).unwrap()),
                    Arc::new(VPKDirectory::load(path.join("hl2\\hl2_misc_dir.vpk")).unwrap()),
                    Arc::new(VPKDirectory::load(path.join("portal\\portal_pak_dir.vpk")).unwrap()),
                ],
                starter_map: Path::new("testchmb_a_02.bsp"),
                maps: path.join("portal\\maps"),
                path,
            },
            Game::Portal2 => Self {
                dirs: vec![Arc::new(
                    VPKDirectory::load(path.join("portal2\\pak01_dir.vpk")).unwrap(),
                )],
                starter_map: Path::new("sp_a4_laser_platform.bsp"),
                maps: path.join("portal2\\maps"),
                path,
            },
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn maps(&self) -> &Path {
        &self.maps
    }

    pub fn dirs(&self) -> &[Arc<VPKDirectory>] {
        self.dirs.as_ref()
    }

    pub fn starter_map(&self) -> &Path {
        self.starter_map
    }
}
