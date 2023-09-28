use std::{sync::Arc, time::Instant};

use ::egui::FontDefinitions;
use bevy_ecs::{
    component::Component,
    system::{Commands, Query},
};
use egui::{Id, LayerId, Pos2, Rect, Ui, Vec2};
use egui_wgpu::{renderer::ScreenDescriptor, Renderer};
use egui_winit::screen_size_in_pixels;
use winit::{event::Event, window::Window};

use crate::{gui::Viewable, v::vrenderer::VRenderer};

pub struct Gui {
    last_frame: Instant,
    renderer: Renderer,
    //puffin_ui : puffin_imgui::ProfilerUi,
}
#[derive(Component)]
pub struct GuiWindow {
    opened: bool,
    view: Arc<dyn Viewable>,
}
impl GuiWindow {
    pub fn new(view: Arc<dyn Viewable>) -> Self {
        Self {
            opened: false,
            view,
        }
    }
    pub fn new_open(view: Arc<dyn Viewable>) -> Self {
        Self { opened: true, view }
    }
    pub fn draw_menu(
        &mut self,
        ui: &mut egui::Ui,
        _renderer: &VRenderer,
        _ui_renderer: &mut Renderer,
    ) {
        ui.checkbox(&mut self.opened, self.view.gui_label());
    }
    pub fn draw_window(
        &mut self,
        ui: &egui::Ui,
        renderer: &VRenderer,
        ui_renderer: &mut Renderer,
        commands: &mut Commands,
    ) {
        //if self.opened {
        //    let window = ui.window(self.view.gui_label());
        //    window
        //        .opened(&mut self.opened)
        //        .size([300.0, 600.0], Condition::FirstUseEver)
        //        .position([400.0, 0.0], Condition::FirstUseEver)
        //        .build(|| {
        //            self.view.gui_view(ui, renderer, ui_renderer, commands);
        //            //end
        //        });
        //}
    }
}

impl Gui {
    pub fn render_pass(
        &mut self,
        renderer: &VRenderer,
        state: &mut egui_winit::State,
        ctx: &egui::Context,
        mut windows: Query<&mut GuiWindow>,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        commands: &mut Commands,
    ) {
        let _delta_s = self.last_frame.elapsed();
        let now = Instant::now();
        //self.imgui.io_mut().update_delta_time(now - self.last_frame);
        self.last_frame = now;

        let id = Id::new(0);

        let full_output = ctx.run(state.take_egui_input(&renderer.window()), |ctx| {
            egui::Window::new("Test")
                .fixed_size(egui::Vec2 { x: 100.0, y: 100.0 })
                .show(&ctx, |ui| {
                    ui.label("Hello egui!");
                    ui.add_space(10.0);
                    ui.label("Hello egui!");
                });
        });
        //self.platform
        //    .prepare_frame(self.imgui.io_mut(), renderer.window())
        //    .expect("Failed to prepare frame");
        //
        //let ui = self.imgui.frame();

        //if let Some(_menu_bar) = ui.begin_main_menu_bar() {
        //    for mut window in windows.iter_mut() {
        //        window.draw_menu(ui, renderer, &mut self.renderer);
        //    }
        //}

        //for mut window in windows.iter_mut() {
        //    window.draw_window(ui, renderer, &mut self.renderer, commands);
        //}

        // if self.last_cursor != ui.mouse_cursor() {
        //     self.last_cursor = ui.mouse_cursor();
        //     self.platform.prepare_render(ui, renderer.window());
        // }

        {
            let paint_jobs = ctx.tessellate(full_output.shapes);

            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: renderer.window().inner_size().into(),
                pixels_per_point: state.pixels_per_point(),
            };

            self.renderer.update_buffers(
                renderer.device(),
                renderer.queue(),
                encoder,
                &paint_jobs,
                &screen_descriptor,
            );

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
                .render(&mut rpass, &paint_jobs, &screen_descriptor);

            // self.renderer
            //     .render(
            //         self.imgui.render(),
            //         renderer.queue(),
            //         renderer.device(),
            //         &mut rpass,
            //     )
            //     .expect("Rendering failed");
        }
    }

    pub fn init(renderer: &VRenderer) -> Self {
        // Set up dear imgui

        let hidpi_factor = renderer.window().scale_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        // imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        // imgui.fonts().add_font(&[FontSource::DefaultFontData {
        //     config: Some(imgui::FontConfig {
        //         oversample_h: 1,
        //         pixel_snap_h: true,
        //         size_pixels: font_size,
        //         ..Default::default()
        //     }),
        // }]);

        //
        // Set up dear imgui wgpu renderer
        //

        //let renderer_config = RendererConfig {
        //    texture_format: renderer
        //        .surface()
        //        .get_current_texture()
        //        .unwrap()
        //        .texture
        //        .format(),
        //    ..Default::default()
        //};

        // let imgui_renderer = Renderer::new(
        //     &mut imgui,
        //     renderer.device(),
        //     renderer.queue(),
        //     renderer_config,
        // );

        //let dx5_data = dir.load_vtf("materials/metal/metalfence001a.vtf").unwrap();
        //let dx1_data = dir.load_vtf("materials/metal/metalfloor001a.vtf").unwrap();

        //let last_cursor = None;

        let renderer =
            egui_wgpu::Renderer::new(renderer.device(), renderer.instance.format, None, 1);

        Self {
            //imgui,
            //last_cursor,
            last_frame: Instant::now(),
            renderer, //platform,
                      //renderer: imgui_renderer,
        }
    }
}

impl Gui {
    pub fn handle_event<T>(&mut self, window: &Window, event: &Event<T>) -> bool {
        //self.platform
        //    .handle_event(self.imgui.io_mut(), window, event);
        //
        //self.imgui.io().want_capture_mouse

        false
    }
}
