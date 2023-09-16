use bevy_ecs::system::Commands;
use imgui::TableFlags;

use crate::{gui::Viewable, state::StateRenderer};

use super::VMT;

impl Viewable for VMT {
    fn gui_view(
        &self,
        ui: &imgui::Ui,
        _renderer: &StateRenderer,
        _ui_renderer: &mut imgui_wgpu::Renderer,
        _commands: &mut Commands,
    ) {
        ui.text(&self.shader);

        let flags = TableFlags::BORDERS | TableFlags::ROW_BG;

        if let Some(_table) = ui.begin_table_with_flags("he;;p", 2, flags) {
            for (key, value) in &self.data {
                ui.table_next_row();

                ui.table_next_column();
                ui.text(key);
                ui.table_next_column();
                ui.text(value);
            }
        }

        ui.text(&self.source);
    }

    fn gui_label(&self) -> &str {
        "Mat"
    }
}
