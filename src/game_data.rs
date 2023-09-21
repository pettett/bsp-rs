use std::{
    io,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use bevy_ecs::system::Resource;

use crate::{
    assets::vpk::VPKDirectory,
    assets::VMT,
    assets::{vpk::VPKFile, vtf::VTF},
    binaries::BinaryData,
    util::v_path::VPath,
};
#[derive(Resource)]
pub struct GameData {
    path: PathBuf,
    maps: PathBuf,
    starter_map: &'static Path,
    dirs: Vec<Arc<VPKDirectory>>,
}

pub enum Game {
    HalfLife2,
    HalfLife2Ep1,
    HalfLife2Ep2,
    Portal,
    Portal2,
    TeamFortress2,
}
impl GameData {
    pub fn load_vmt(&self, path: &dyn VPath) -> Option<&Arc<VMT>> {
        for d in &self.dirs {
            if let Ok(vmt) = d.load_vmt(path) {
                return Some(vmt);
            }
        }
        None
    }

    pub fn load_vtf(&self, path: &dyn VPath) -> Option<&Arc<VTF>> {
        for d in &self.dirs {
            if let Ok(vtf) = d.load_vtf(path) {
                return Some(vtf);
            }
        }
        None
    }

    pub fn load<'a, T: BinaryData + 'a, F: Fn(&'a VPKFile) -> &'a OnceLock<io::Result<Arc<T>>>>(
        &'a self,
        path: &dyn VPath,
        get_cell: F,
    ) -> Option<&'a Arc<T>> {
        for d in &self.dirs {
            if let Ok(vtf) = d.load_file_once(path, &get_cell) {
                return Some(vtf);
            }
        }
        None
    }

    pub fn load_game(game: Game) -> Self {
        let path =
            Path::new("D:\\Program Files (x86)\\Steam\\steamapps\\common").join(match game {
                Game::HalfLife2 | Game::HalfLife2Ep1 | Game::HalfLife2Ep2 => "Half-Life 2",
                Game::Portal => "Portal",
                Game::Portal2 => "Portal 2",
                Game::TeamFortress2 => "Team Fortress 2",
            });

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
            Game::HalfLife2Ep1 => Self {
                dirs: vec![
                    Arc::new(VPKDirectory::load(path.join("hl2\\hl2_textures_dir.vpk")).unwrap()),
                    Arc::new(VPKDirectory::load(path.join("hl2\\hl2_misc_dir.vpk")).unwrap()),
                    Arc::new(VPKDirectory::load(path.join("episodic\\ep1_pak_dir.vpk")).unwrap()),
                ],
                starter_map: Path::new("ep1_c17_01.bsp"),
                maps: path.join("episodic\\maps"),
                path,
            },
            Game::HalfLife2Ep2 => Self {
                dirs: vec![
                    Arc::new(VPKDirectory::load(path.join("hl2\\hl2_textures_dir.vpk")).unwrap()),
                    Arc::new(VPKDirectory::load(path.join("hl2\\hl2_misc_dir.vpk")).unwrap()),
                    Arc::new(VPKDirectory::load(path.join("ep2\\ep2_pak_dir.vpk")).unwrap()),
                ],
                starter_map: Path::new("ep2_outland_07.bsp"),
                maps: path.join("ep2\\maps"),
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
            Game::TeamFortress2 => Self {
                dirs: vec![
                    Arc::new(VPKDirectory::load(path.join("tf\\tf2_misc_dir.vpk")).unwrap()),
                    Arc::new(VPKDirectory::load(path.join("tf\\tf2_textures_dir.vpk")).unwrap()),
                ],
                starter_map: Path::new("ctf_2fort.bsp"),
                maps: path.join("tf\\maps"),
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
