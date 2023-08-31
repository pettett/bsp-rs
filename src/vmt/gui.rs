use crate::gui::Viewable;

use super::VMT;

impl Viewable for VMT {
    fn gui_view(
        &self,
        ui: &imgui::Ui,
        renderer: &mut imgui_wgpu::Renderer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
    }

    fn gui_label(&self) -> &str {
        "Mat"
    }
}
