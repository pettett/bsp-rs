use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    time::Instant,
};

use imgui::{Condition, FontSource, Image, Ui};
use imgui_wgpu::{Renderer, RendererConfig};
use wgpu::{Device, Queue};
use winit::event::Event;

use crate::{
    state::{State, StateRenderer},
    vpk::{VPKDirectory, VPKDirectoryTree},
};

pub struct StateImgui {
    imgui: imgui::Context,
    last_cursor: Option<imgui::MouseCursor>,
    last_frame: Instant,
    platform: imgui_winit_support::WinitPlatform,
    renderer: Renderer,
    //puffin_ui : puffin_imgui::ProfilerUi,
}

fn draw_dir(
    ui: &Ui,
    renderer: &mut Renderer,
    device: &Device,
    queue: &Queue,
    tree: &VPKDirectoryTree,
    dir: &VPKDirectory,
) {
    match tree {
        VPKDirectoryTree::Leaf(file) => {
            let uncomp_data = dir.load_vtf(file).unwrap();

            ui.text(file);

            if let Some(node) = ui.tree_node("High res") {
                Image::new(
                    *uncomp_data.get_high_res_imgui(device, queue, renderer),
                    [64.0 * 4.0, 64.0 * 4.0],
                )
                .build(ui);
            }
        }
        VPKDirectoryTree::Node(dir_inner) => {
            for file in dir_inner.keys() {
                if let Some(node) = ui.tree_node(file) {
                    draw_dir(ui, renderer, device, queue, &dir_inner[file], dir);
                }
            }
        }
    }
}

impl State for StateImgui {
    fn render_pass(
        &mut self,
        state: &StateRenderer,
        encoder: &mut wgpu::CommandEncoder,
        output: &wgpu::SurfaceTexture,
        view: &wgpu::TextureView,
    ) {
        let delta_s = self.last_frame.elapsed();
        let now = Instant::now();
        self.imgui.io_mut().update_delta_time(now - self.last_frame);
        self.last_frame = now;

        self.platform
            .prepare_frame(self.imgui.io_mut(), state.window())
            .expect("Failed to prepare frame");

        let ui = self.imgui.frame();

        {
            let window = ui.window("Camera");
            window
                .size([400.0, 200.0], Condition::FirstUseEver)
                .position([400.0, 200.0], Condition::FirstUseEver)
                .build(|| {
                    ui.text(format!("Frametime: {delta_s:?}"));

                    ui.text(format!(
                        "Camera Pos: {}",
                        state.camera().transform().get_pos()
                    ));

                    ui.text(format!(
                        "Camera Rot: {:?}",
                        state
                            .camera()
                            .transform()
                            .get_rot()
                            .to_euler(glam::EulerRot::XYZ)
                    ));

                    //end
                });
            let window = ui.window("Texture Pak");
            window
                .size([400.0, 200.0], Condition::FirstUseEver)
                .position([0.0, 0.0], Condition::FirstUseEver)
                .build(|| {
                    draw_dir(
                        ui,
                        &mut self.renderer,
                        state.device(),
                        state.queue(),
                        state.texture_dir().get_root(),
                        state.texture_dir(),
                    )
                    //end
                });
            //self.puffin_ui.window(ui);
        }

        if self.last_cursor != ui.mouse_cursor() {
            self.last_cursor = ui.mouse_cursor();
            self.platform.prepare_render(ui, state.window());
        }

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.renderer
                .render(
                    self.imgui.render(),
                    state.queue(),
                    state.device(),
                    &mut rpass,
                )
                .expect("Rendering failed");
        }
    }

    fn init(renderer: &StateRenderer) -> Self {
        // Set up dear imgui
        let mut imgui = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            renderer.window(),
            imgui_winit_support::HiDpiMode::Default,
        );
        imgui.set_ini_filename(None);

        let hidpi_factor = renderer.window().scale_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        //
        // Set up dear imgui wgpu renderer
        //

        let renderer_config = RendererConfig {
            texture_format: renderer
                .surface()
                .get_current_texture()
                .unwrap()
                .texture
                .format(),
            ..Default::default()
        };

        let mut imgui_renderer = Renderer::new(
            &mut imgui,
            renderer.device(),
            renderer.queue(),
            renderer_config,
        );

        //let dx5_data = dir.load_vtf("materials/metal/metalfence001a.vtf").unwrap();
        //let dx1_data = dir.load_vtf("materials/metal/metalfloor001a.vtf").unwrap();

        let last_cursor = None;

        Self {
            imgui,
            last_cursor,
            last_frame: Instant::now(),
            platform,
            renderer: imgui_renderer,
        }
    }
}

impl StateImgui {
    pub fn handle_event<T>(&mut self, state: &StateRenderer, event: &Event<T>) {
        self.platform
            .handle_event(self.imgui.io_mut(), state.window(), event);
    }
}
