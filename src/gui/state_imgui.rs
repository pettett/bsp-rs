use std::{sync::Arc, time::Instant};

use imgui::{Condition, FontSource};
use imgui_wgpu::{Renderer, RendererConfig};

use winit::event::Event;

use crate::{
    gui::Viewable,
    state::{State, StateRenderer},
};

pub struct StateImgui {
    imgui: imgui::Context,
    last_cursor: Option<imgui::MouseCursor>,
    last_frame: Instant,
    platform: imgui_winit_support::WinitPlatform,
    renderer: Renderer,
    //puffin_ui : puffin_imgui::ProfilerUi,
    windows: Vec<WindowState>,
}

pub struct WindowState {
    opened: bool,
    view: Arc<dyn Viewable>,
}
impl WindowState {
    pub fn new(view: Arc<dyn Viewable>) -> Self {
        Self {
            opened: false,
            view,
        }
    }
    pub fn draw_menu(
        &mut self,
        ui: &imgui::Ui,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &mut Renderer,
    ) {
        ui.checkbox(self.view.gui_label(), &mut self.opened);
    }
    pub fn draw_window(
        &mut self,
        ui: &imgui::Ui,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &mut Renderer,
    ) {
        if self.opened {
            let window = ui.window(self.view.gui_label());
            window
                .opened(&mut self.opened)
                .size([300.0, 600.0], Condition::FirstUseEver)
                .position([400.0, 0.0], Condition::FirstUseEver)
                .build(|| {
                    self.view.gui_view(ui, renderer, device, queue);
                    //end
                });
        }
    }
}

impl State for StateImgui {
    fn render_pass(
        &mut self,
        state: &StateRenderer,
        encoder: &mut wgpu::CommandEncoder,
        _output: &wgpu::SurfaceTexture,
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

        if let Some(menu_bar) = ui.begin_main_menu_bar() {
            for window in &mut self.windows {
                window.draw_menu(ui, state.device(), state.queue(), &mut self.renderer);
            }
        }

        for window in &mut self.windows {
            window.draw_window(ui, state.device(), state.queue(), &mut self.renderer);
        }

        {
            // let window = ui.window("Camera");
            // window
            //     .opened(&mut self.pak)
            //     .size([400.0, 200.0], Condition::FirstUseEver)
            //     .position([400.0, 200.0], Condition::FirstUseEver)
            //     .build(|| {
            //         ui.text(format!("Frametime: {delta_s:?}"));

            //         ui.text(format!(
            //             "Camera Pos: {}",
            //             state.camera().transform().get_pos()
            //         ));

            //         ui.text(format!(
            //             "Camera Rot: {:?}",
            //             state
            //                 .camera()
            //                 .transform()
            //                 .get_rot()
            //                 .to_euler(glam::EulerRot::XYZ)
            //         ));

            //         //end
            //     });
            // {
            //     let window = ui.window(state.texture_dir().gui_label());
            //     window
            //         .size([300.0, 600.0], Condition::FirstUseEver)
            //         .position([0.0, 0.0], Condition::FirstUseEver)
            //         .build(|| {
            //             state.texture_dir().gui_view(
            //                 ui,
            //                 &mut self.renderer,
            //                 state.device(),
            //                 state.queue(),
            //             );
            //             //end
            //         });
            // }
            // {
            //     let window = ui.window(state.misc_dir().gui_label());
            //     window
            //         .size([300.0, 600.0], Condition::FirstUseEver)
            //         .position([400.0, 0.0], Condition::FirstUseEver)
            //         .build(|| {
            //             state.misc_dir().gui_view(
            //                 ui,
            //                 &mut self.renderer,
            //                 state.device(),
            //                 state.queue(),
            //             );
            //             //end
            //         });
            // }

            // if self.pak {
            //     if let Some(pak) = &state.pak {
            //         let window = ui.window("Map Pak");

            //         window
            //             .opened(&mut self.pak)
            //             .size([300.0, 200.0], Condition::FirstUseEver)
            //             .position([400.0, 0.0], Condition::FirstUseEver)
            //             .build(|| {
            //                 for e in &pak.entries {
            //                     if let Some(_node) = ui.tree_node(&e.filename) {
            //                         if let Some(tex) = e.get_vtf() {
            //                             Image::new(
            //                                 *tex.get_high_res_imgui(
            //                                     state.device(),
            //                                     state.queue(),
            //                                     &mut self.renderer,
            //                                 ),
            //                                 [64.0 * 4.0, 64.0 * 4.0],
            //                             )
            //                             .build(ui);
            //                         }

            //                         if let Some(mat) = e.get_vmt() {
            //                             ui.text_wrapped(format!("{:#?}", mat))
            //                         }
            //                     }
            //                 }
            //                 //end
            //             });
            //     }
            // }

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

        let imgui_renderer = Renderer::new(
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
            windows: vec![
                WindowState::new(renderer.misc_dir().clone()),
                WindowState::new(renderer.texture_dir().clone()),
            ],
        }
    }
}

impl StateImgui {
    pub fn handle_event<T>(&mut self, state: &StateRenderer, event: &Event<T>) -> bool {
        self.platform
            .handle_event(self.imgui.io_mut(), state.window(), event);

        self.imgui.io().want_capture_mouse
    }
}
