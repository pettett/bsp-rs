use bevy_ecs::system::Commands;

use crate::{gui::Viewable, v::vrenderer::VRenderer};

use source::vpk::VPKDirectory;

impl Viewable for VPKDirectory {
    fn gui_view(
        &self,
        ui: &imgui::Ui,
        renderer: &VRenderer,
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
                                        Ok(vmt) => {
                                            vmt.gui_view(ui, renderer, ui_renderer, commands)
                                        }
                                        Err(e) => ui.text(format!("Error loading Material: {}", e)),
                                    },
                                    "vtf" => match data.load_vtf(&self) {
                                        Ok(vtf) => {
                                            vtf.gui_view(ui, renderer, ui_renderer, commands)
                                        }
                                        Err(e) => ui.text(format!("Error loading Texture: {}", e)),
                                    },
                                    _ => ui.text("Unknown format"),
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
