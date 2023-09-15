use bevy_ecs::system::Commands;

use crate::{gui::Viewable, state::StateRenderer};

use super::VTF;

impl Viewable for VTF {
    fn gui_view(
        &self,
        ui: &imgui::Ui,
        renderer: &StateRenderer,
        ui_renderer: &mut imgui_wgpu::Renderer,
        commands: &mut Commands,
    ) {
        ui.text(format!("{}x{}", self.width(), self.height()));

        if let Ok(low_res) = self.get_low_res_imgui(renderer.device(), renderer.queue(), ui_renderer) {
            imgui::Image::new(
                *low_res,
                [32.0, 32.0],
            )
                .build(ui);
        }

        if let Some(_node) = ui.tree_node("High res") {
            if let Ok(high_res) = self.get_high_res_imgui(renderer.device(), renderer.queue(), ui_renderer) {
                imgui::Image::new(
                    *high_res,
                    [64.0 * 4.0, 64.0 * 4.0],
                )
                    .build(ui);
            }
        }
    }

    fn gui_label(&self) -> &str {
        "tex"
    }
}
