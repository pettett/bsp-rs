use imgui::TableFlags;

use crate::gui::Viewable;

use super::VMT;

impl Viewable for VMT {
    fn gui_view(
        &self,
        ui: &imgui::Ui,
        _renderer: &mut imgui_wgpu::Renderer,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        ui.text(&self.shader);

        let flags = TableFlags::BORDERS | TableFlags::ROW_BG;

        if let Some(table) = ui.begin_table_with_flags("he;;p", 2, flags) {
            for (key, value) in &self.data {
                ui.table_next_row();

                ui.table_next_column();
                ui.text(key);
                ui.table_next_column();
                ui.text(value);
            }
        }
    }

    fn gui_label(&self) -> &str {
        "Mat"
    }
}
