use bevy_ecs::system::Commands;
use source::vmt::VMT;

use crate::{gui::Viewable, v::vrenderer::VRenderer};

impl Viewable for VMT {
    fn gui_view(
        &self,
        ui: &mut egui::Ui,
        renderer: &VRenderer,
        ui_renderer: &mut egui_wgpu::Renderer,
        commands: &mut Commands,
    ) {
        ui.label(&self.shader);

        egui::Grid::new(&self.source).show(ui, |ui| {
            for (key, value) in &self.data {
                ui.label(key);
                ui.label(value);
                ui.end_row();
            }
        });

        ui.label(&self.source);
    }

    fn gui_label(&self) -> &str {
        "Mat"
    }
}
