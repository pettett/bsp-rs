use bevy_ecs::system::Commands;
use imgui_wgpu::Renderer;

use crate::{gui::Viewable, state::StateRenderer};

use super::{VPKDirectory, VPKDirectoryTree};

fn draw_dir(
    ui: &imgui::Ui,
    renderer: &mut Renderer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    tree: &VPKDirectoryTree,
    dir: &VPKDirectory,
) {
    match tree {
        VPKDirectoryTree::Leaf(file) => {}
        VPKDirectoryTree::Node(dir_inner) => {
            let mut keys: Vec<&String> = dir_inner.keys().collect();
            keys.sort();
            for file in keys {
                if let Some(_node) = ui.tree_node(file) {
                    draw_dir(ui, renderer, device, queue, &dir_inner[file], dir);
                }
            }
        }
    }
}

impl Viewable for VPKDirectory {
    fn gui_view(
        &self,
        ui: &imgui::Ui,
        renderer: &StateRenderer,
        ui_renderer: &mut imgui_wgpu::Renderer,
        commands: &mut Commands,
    ) {
        for (ext, dirs) in &self.files {
            if let Some(_node) = ui.tree_node(ext) {
                for (dir, files) in dirs {
                    if let Some(_node) = ui.tree_node(dir) {
                        for (file, data) in files {
                            if let Some(_node) = ui.tree_node(file) {
                                // Try to load any data associated with this file

                                match ext.as_str() {
                                    "vmt" => match data.load_vmt(&self) {
                                        Ok(Some(vmt)) => vmt.gui_view(ui, renderer, ui_renderer, commands),
                                        Ok(None) => ui.text("No Material"),
                                        Err(e) => ui.text(format!("Error loading Material: {}", e)),
                                    },
                                    "vtf" => match data.load_vtf(&self) {
                                        Ok(Some(vtf)) => vtf.gui_view(ui, renderer, ui_renderer, commands),
                                        Ok(None) => ui.text("No Texture"),
                                        Err(e) => ui.text(format!("Error loading Texture: {}", e)),
                                    },
                                    _ => ui.text("Unknown format")
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn gui_label(&self) -> &str {
        self.dir_path.file_name().unwrap().to_str().unwrap()
    }
}
