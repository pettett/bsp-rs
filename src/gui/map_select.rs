use std::{
    fs, io,
    path::{Path, PathBuf},
};

use bevy_ecs::system::Commands;

use crate::{bsp::loader::load_bsp, state::StateRenderer};

use super::Viewable;

pub struct MapSelect {
    file_names: Vec<PathBuf>,
}
impl MapSelect {
    pub fn new(path: &str) -> io::Result<Self> {
        // Get a list of all entries in the folder
        let entries = fs::read_dir(path)?;

        let root = Path::new(path);

        // Extract the filenames from the directory entries and store them in a vector
        let file_names = entries
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.is_file() && path.extension().unwrap().to_str() == Some("bsp") {
                    Some(root.join(path))
                } else {
                    None
                }
            })
            .collect();
        Ok(Self { file_names })
    }
}

impl Viewable for MapSelect {
    fn gui_view(
        &self,
        ui: &imgui::Ui,
        renderer: &StateRenderer,
        ui_renderer: &mut imgui_wgpu::Renderer,
        commands: &mut Commands,
    ) {
        for map_path in &self.file_names {
            let map_name = map_path.file_name().unwrap().to_str().unwrap();
            if ui.button(map_name) {
                println!("Loading {}", map_name);
                load_bsp(map_path, commands, renderer);
            }
        }
    }

    fn gui_label(&self) -> &str {
        "Map Selection"
    }
}
