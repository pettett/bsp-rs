use bevy_ecs::system::Commands;
use imgui::Ui;
use imgui_wgpu::Renderer;

use crate::state::StateRenderer;

pub trait Viewable {
    fn gui_view(
        &self,
        ui: &Ui,
        renderer: &StateRenderer,
        ui_renderer: &mut Renderer,
        commands: &mut Commands,
    );
    fn gui_label(&self) -> &str;
}
