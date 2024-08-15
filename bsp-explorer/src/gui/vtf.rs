use bevy_ecs::system::Commands;
use common::{vinstance::StateInstance, vtexture::VTexture};
use egui::{load::SizedTexture, ImageSource, TextureId, Vec2};
use egui_wgpu::Renderer;
use source::{
    vmt::VMT,
    vtf::{vtf::VRes, VTF},
};

use crate::{gui::Viewable, v::vrenderer::VRenderer};

impl Viewable for VTF {
    fn gui_view(
        &self,
        ui: &mut egui::Ui,
        renderer: &VRenderer,
        ui_renderer: &mut egui_wgpu::Renderer,
        commands: &mut Commands,
    ) {
        ui.label(format!("{}x{}", self.width(), self.height()));

        if let Ok(low_res) = get_low_res_imgui(self, &renderer.instance, ui_renderer) {
            ui.add(egui::Image::new(ImageSource::Texture(SizedTexture::new(
                low_res,
                Vec2::new(32., 32.),
            ))));
        }
        egui::CollapsingHeader::new("High res")
            .default_open(false)
            .show(ui, |ui| {
                if let Ok(high_res) = get_high_res_imgui(self, &renderer.instance, ui_renderer) {
                    ui.add(egui::Image::new(ImageSource::Texture(SizedTexture::new(
                        high_res,
                        Vec2::new(256.0, 256.0),
                    ))));
                }
            });
        // if let Some(_node) = ui.tree_node() {
        //     if let Ok(high_res) = get_high_res_imgui(self, &renderer.instance, ui_renderer) {
        //         imgui::Image::new(high_res, [64.0 * 4.0, 64.0 * 4.0]).build(ui);
        //     }
        // }
    }

    fn gui_label(&self) -> &str {
        "tex"
    }
}

pub fn get_high_res_imgui(
    tex: &VTF,
    instance: &StateInstance,
    renderer: &mut Renderer,
) -> VRes<TextureId> {
    tex.high_res_imgui
        .get_or_init(|| match tex.get_high_res(instance) {
            Ok(high_res) => match renderer.register_native_texture(
                &instance.device,
                high_res.view(),
                wgpu::FilterMode::Linear,
            ) {
                egui::TextureId::Managed(_) => todo!(),
                egui::TextureId::User(i) => Ok(i),
            },
            Err(e) => Err(*e),
        })
        .map(|x| egui::TextureId::User(x))
}

pub fn get_low_res_imgui(
    tex: &VTF,
    instance: &StateInstance,
    renderer: &mut Renderer,
) -> VRes<TextureId> {
    tex.low_res_imgui
        .get_or_init(|| match tex.get_low_res(instance) {
            Ok(low_res) => match renderer.register_native_texture(
                &instance.device,
                low_res.view(),
                wgpu::FilterMode::Linear,
            ) {
                egui::TextureId::Managed(_) => todo!(),
                egui::TextureId::User(i) => Ok(i),
            },
            Err(e) => Err(*e),
        })
        .map(|x| egui::TextureId::User(x))
}
