use bevy_ecs::system::Commands;
use common::{vinstance::StateInstance, vtexture::VTexture};
use source::{
    vmt::VMT,
    vtf::{vtf::VRes, VTF},
};

use crate::{gui::Viewable, v::vrenderer::VRenderer};

impl Viewable for VTF {
    fn gui_view(
        &self,
        ui: &imgui::Ui,
        renderer: &VRenderer,
        ui_renderer: &mut imgui_wgpu::Renderer,
        _commands: &mut Commands,
    ) {
        ui.text(format!("{}x{}", self.width(), self.height()));

        if let Ok(low_res) = get_low_res_imgui(self, &renderer.instance, ui_renderer) {
            imgui::Image::new(low_res, [32.0, 32.0]).build(ui);
        }

        if let Some(_node) = ui.tree_node("High res") {
            if let Ok(high_res) = get_high_res_imgui(self, &renderer.instance, ui_renderer) {
                imgui::Image::new(high_res, [64.0 * 4.0, 64.0 * 4.0]).build(ui);
            }
        }
    }

    fn gui_label(&self) -> &str {
        "tex"
    }
}

pub fn get_high_res_imgui(
    tex: &VTF,
    instance: &StateInstance,
    renderer: &mut imgui_wgpu::Renderer,
) -> VRes<imgui::TextureId> {
    tex.high_res_imgui
        .get_or_init(|| match tex.get_high_res(instance) {
            Ok(high_res) => Ok(renderer
                .textures
                .insert(to_imgui(&high_res, &instance.device, renderer))
                .id()),
            Err(e) => Err(*e),
        })
        .map(|x| imgui::TextureId::new(x))
}

pub fn get_low_res_imgui(
    tex: &VTF,
    instance: &StateInstance,
    renderer: &mut imgui_wgpu::Renderer,
) -> VRes<imgui::TextureId> {
    tex.low_res_imgui
        .get_or_init(|| match tex.get_low_res(instance) {
            Ok(low_res) => Ok(renderer
                .textures
                .insert(to_imgui(&low_res, &instance.device, renderer))
                .id()),
            Err(e) => Err(*e),
        })
        .map(|x| imgui::TextureId::new(x))
}

pub fn to_imgui(
    tex: &VTexture,
    device: &wgpu::Device,
    renderer: &imgui_wgpu::Renderer,
) -> imgui_wgpu::Texture {
    let size = tex.texture.size();
    imgui_wgpu::Texture::from_raw_parts(
        device,
        renderer,
        tex.texture.clone(),
        tex.view.clone(),
        None,
        Some(&imgui_wgpu::RawTextureConfig {
            label: Some("Test"),
            sampler_desc: Default::default(),
        }),
        size,
    )
}
