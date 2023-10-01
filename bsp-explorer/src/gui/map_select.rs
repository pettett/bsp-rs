use std::{
    fs, io,
    path::{Path, PathBuf},
};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, SystemState},
    world::World,
};

use crate::{
    state::MapChangeEvent,
    v::{vrenderer::VRenderer, VMesh},
};

use super::Viewable;

pub struct MapSelect {
    file_names: Vec<PathBuf>,
}
impl MapSelect {
    pub fn new(root: &Path) -> io::Result<Self> {
        // Get a list of all entries in the folder
        let entries = fs::read_dir(root)?;

        // Extract the filenames from the directory entries and store them in a vector
        let file_names = entries
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.is_file() && path.extension().unwrap().to_str() == Some("bsp") {
                    Some(root.join(path))
                } else {
                    None
                }
            })
            .collect();
        Ok(Self { file_names })
    }
}

impl Viewable for MapSelect {
    fn gui_view(
        &self,
        ui: &mut egui::Ui,
        renderer: &VRenderer,
        ui_renderer: &mut egui_wgpu::Renderer,
        commands: &mut Commands,
    ) {
        for map_path in &self.file_names {
            let map_name = map_path.file_name().unwrap().to_str().unwrap();
            if ui.button(map_name) {
                println!("Loading {}", map_name);
                let map_path_clone = map_path.clone();

                commands.add(|w: &mut World| {
                    let mut system_state: SystemState<(Commands, Query<(Entity, &VMesh)>)> =
                        SystemState::new(w);

                    let (mut commands, meshes) = system_state.get(w);

                    for (entity, _mesh) in meshes.iter() {
                        commands.entity(entity).despawn();
                    }

                    system_state.apply(w);

                    w.send_event(MapChangeEvent(map_path_clone))
                });
            }
        }
    }

    fn gui_label(&self) -> &str {
        "Map Selection"
    }
}
