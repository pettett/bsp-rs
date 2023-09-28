use bevy_ecs::system::Resource;
use common::{vfile::VFileSystem, vpath::VPath};
use ini::Ini;
use std::{
    io::{self, BufReader, Cursor},
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
    time::Instant,
};

use crate::{binaries::BinaryData, prelude::*};

#[derive(Resource)]
pub struct GameDataArc {
    pub inner: Arc<GameData>,
}

pub struct GameData {
    path: PathBuf,
    maps: PathBuf,
    starter_map: PathBuf,
    dirs: Vec<Arc<VPKDirectory>>,
}
pub enum Game {
    HalfLife2,
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
            match d.load_file_once(path, &get_cell) {
                Ok(vtf) => return Some(vtf),
                Err(_) => continue,
            }
        }
        None
    }
    pub fn from_ini(ini: &Ini) -> GameDataArc {
        println!("Loading game data... ");
        let now = Instant::now();

        let mut path = PathBuf::new();

        let launch = ini.section(Some("launch")).unwrap();

        let game = ini.section(Some(launch.get("game")).unwrap()).unwrap();
        let root = launch.get("root").unwrap();
        path.push(root);
        let name = game.get("name").unwrap();
        let map = game.get("map").unwrap();
        path.push(name);

        let maps = path.join(game.get("maps").unwrap());

        let mut dirs = Vec::new();
        for vpk in game.get_all("vpk") {
            println!("{vpk}");

            dirs.push(Arc::new(
                VPKDirectory::load(Default::default(), path.join(vpk)).unwrap(),
            ));
        }

        println!("Took {:?}", now.elapsed());
        GameDataArc {
            inner: Arc::new(GameData {
                starter_map: maps.join(map),
                maps,
                dirs,
                path,
            }),
        }
    }
    pub fn load_game(game: Game, file_load: VFileSystem) -> GameDataArc {
        let path = PathBuf::new();
        // Path::new("D:\\Program Files (x86)\\Steam\\steamapps\\common").join(match game {
        //     Game::HalfLife2 | Game::HalfLife2Ep1 | Game::HalfLife2Ep2 => "Half-Life 2",
        //     Game::Portal => "Portal",
        //     Game::Portal2 => "Portal 2",
        //     Game::TeamFortress2 => "Team Fortress 2",
        // });
        GameDataArc {
            inner: Arc::new(match game {
                Game::HalfLife2 => Self {
                    dirs: vec![
                        Arc::new(
                            VPKDirectory::load(
                                file_load.clone(),
                                path.join("hl2_textures_dir.vpk"),
                            )
                            .unwrap(),
                        ),
                        Arc::new(
                            VPKDirectory::load(file_load.clone(), path.join("hl2_misc_dir.vpk"))
                                .unwrap(),
                        ),
                    ],
                    starter_map: Path::new("d1_trainstation_02.bsp").to_owned(),
                    maps: path.clone(),
                    path,
                },
                // Game::HalfLife2Ep1 => Self {
                //     dirs: vec![
                //         Arc::new(VPKDirectory::load(path.join("hl2\\hl2_textures_dir.vpk")).unwrap()),
                //         Arc::new(VPKDirectory::load(path.join("hl2\\hl2_misc_dir.vpk")).unwrap()),
                //         Arc::new(VPKDirectory::load(path.join("episodic\\ep1_pak_dir.vpk")).unwrap()),
                //     ],
                //     starter_map: Path::new("ep1_c17_01.bsp"),
                //     maps: path.join("episodic\\maps"),
                //     path,
                // },
                // Game::HalfLife2Ep2 => Self {
                //     dirs: vec![
                //         Arc::new(VPKDirectory::load(path.join("hl2\\hl2_textures_dir.vpk")).unwrap()),
                //         Arc::new(VPKDirectory::load(path.join("hl2\\hl2_misc_dir.vpk")).unwrap()),
                //         Arc::new(VPKDirectory::load(path.join("ep2\\ep2_pak_dir.vpk")).unwrap()),
                //     ],
                //     starter_map: Path::new("ep2_outland_07.bsp"),
                //     maps: path.join("ep2\\maps"),
                //     path,
                // },
                // Game::Portal => Self {
                //     dirs: vec![
                //         Arc::new(VPKDirectory::load(path.join("hl2\\hl2_textures_dir.vpk")).unwrap()),
                //         Arc::new(VPKDirectory::load(path.join("hl2\\hl2_misc_dir.vpk")).unwrap()),
                //         Arc::new(VPKDirectory::load(path.join("portal\\portal_pak_dir.vpk")).unwrap()),
                //     ],
                //     starter_map: Path::new("testchmb_a_02.bsp"),
                //     maps: path.join("portal\\maps"),
                //     path,
                // },
                // Game::Portal2 => Self {
                //     dirs: vec![Arc::new(
                //         VPKDirectory::load(path.join("portal2\\pak01_dir.vpk")).unwrap(),
                //     )],
                //     starter_map: Path::new("sp_a4_laser_platform.bsp"),
                //     maps: path.join("portal2\\maps"),
                //     path,
                // },
                // Game::TeamFortress2 => Self {
                //     dirs: vec![
                //         Arc::new(VPKDirectory::load(path.join("tf\\tf2_misc_dir.vpk")).unwrap()),
                //         Arc::new(VPKDirectory::load(path.join("tf\\tf2_textures_dir.vpk")).unwrap()),
                //     ],
                //     starter_map: Path::new("ctf_2fort.bsp"),
                //     maps: path.join("tf\\maps"),
                //     path,
                // },
            }),
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
        &self.starter_map
    }
}
