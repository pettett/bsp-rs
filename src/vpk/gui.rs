use imgui_wgpu::Renderer;

use crate::gui::Viewable;

use super::{VPKDirectory, VPKDirectoryTree, VPKFile};

fn draw_dir(
    ui: &imgui::Ui,
    renderer: &mut Renderer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    tree: &VPKDirectoryTree,
    dir: &VPKDirectory,
) {
    match tree {
        VPKDirectoryTree::Leaf(file) => {
            if let Some(tex) = dir.load_vtf(file).unwrap() {
                tex.gui_view(ui, renderer, device, queue);
            } else if let Some(mat) = dir.load_vmt(file).unwrap() {
                mat.gui_view(ui, renderer, device, queue);
            } else {
                ui.text("Unknown format")
            }
        }
        VPKDirectoryTree::Node(dir_inner) => {
            let mut keys: Vec<&String> = dir_inner.keys().collect();
            keys.sort();
            for file in keys {
                if let Some(_node) = ui.tree_node(file) {
                    draw_dir(ui, renderer, device, queue, &dir_inner[file], dir);
                }
            }
        }
    }
}

impl Viewable for VPKDirectory {
    fn gui_view(
        &self,
        ui: &imgui::Ui,
        renderer: &mut imgui_wgpu::Renderer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        draw_dir(ui, renderer, device, queue, &self.root, self)
    }

    fn gui_label(&self) -> &str {
        self.dir_path.file_name().unwrap().to_str().unwrap()
    }
}
