use bevy_ecs::system::Commands;

use crate::{gui::Viewable, v::vrenderer::VRenderer};

use source::vpk::VPKDirectory;

impl Viewable for VPKDirectory {
    fn gui_view(
        &self,
        ui: &mut egui::Ui,
        renderer: &VRenderer,
        ui_renderer: &mut egui_wgpu::Renderer,
        commands: &mut Commands,
    ) {
        for (ext, dirs) in &self.files {
            egui::CollapsingHeader::new(ext)
                .default_open(false)
                .show(ui, |ui| {
                    for (dir, files) in dirs {
                        egui::CollapsingHeader::new(dir)
                            .default_open(false)
                            .show(ui, |ui| {
                                for (file, data) in files {
                                    egui::CollapsingHeader::new(file).default_open(false).show(
                                        ui,
                                        |ui| {
                                            // Try to load any data associated with this file

                                            match ext.as_str() {
                                                "vmt" => match data.load_vmt(&self) {
                                                    Ok(vmt) => vmt.gui_view(
                                                        ui,
                                                        renderer,
                                                        ui_renderer,
                                                        commands,
                                                    ),
                                                    Err(e) => {
                                                        ui.label(format!(
                                                            "Error loading Material: {}",
                                                            e
                                                        ));
                                                    }
                                                },
                                                "vtf" => match data.load_vtf(&self) {
                                                    Ok(vtf) => vtf.gui_view(
                                                        ui,
                                                        renderer,
                                                        ui_renderer,
                                                        commands,
                                                    ),
                                                    Err(e) => {
                                                        ui.label(format!(
                                                            "Error loading Texture: {}",
                                                            e
                                                        ));
                                                    }
                                                },
                                                _ => {
                                                    ui.label("Unknown format");
                                                }
                                            }
                                        },
                                    );
                                }
                            });
                    }
                });
        }
    }

    fn gui_label(&self) -> &str {
        self.dir_path.file_name().unwrap().to_str().unwrap()
    }
}
