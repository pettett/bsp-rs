use imgui::Ui;
use imgui_wgpu::Renderer;

pub trait Viewable {
    fn gui_view(
        &self,
        ui: &Ui,
        renderer: &mut Renderer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    );
    fn gui_label(&self) -> &str;
}
