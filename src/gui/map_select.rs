use std::{fs, io, path::PathBuf};

use super::Viewable;

pub struct MapSelect {
    file_names: Vec<PathBuf>,
}
impl MapSelect {
    pub fn new(path: &str) -> io::Result<Self> {
        // Get a list of all entries in the folder
        let entries = fs::read_dir(path)?;

        // Extract the filenames from the directory entries and store them in a vector
        let file_names = entries
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.is_file() && path.extension().unwrap().to_str() == Some("bsp") {
                    Some(path)
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
        renderer: &mut imgui_wgpu::Renderer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        for f in &self.file_names {
            let n = f.file_name().unwrap().to_str().unwrap();
            if ui.button(n) {
                println!("Loading {}", n);
            }
        }
    }

    fn gui_label(&self) -> &str {
        "Map Selection"
    }
}
