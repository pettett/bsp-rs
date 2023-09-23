use bevy_ecs::system::Commands;
use imgui::Ui;
use imgui_wgpu::Renderer;

use crate::v::vrenderer::VRenderer;

pub trait Viewable: Sync + Send {
    fn gui_view(
        &self,
        ui: &Ui,
        renderer: &VRenderer,
        ui_renderer: &mut Renderer,
        commands: &mut Commands,
    );
    fn gui_label(&self) -> &str;
}
