use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, SystemState},
    world::World,
};

use crate::{
    state::{CommandTask, MapChangeEvent},
    v::{vrenderer::VRenderer, VMesh},
};

use super::Viewable;

pub struct TaskViewer {
    tasks: Arc<Mutex<Vec<&'static str>>>,
}

impl TaskViewer {
    pub fn new() -> Self {
        Self {
            tasks: Default::default(),
        }
    }
}

impl Viewable for TaskViewer {
    fn gui_view(
        &self,
        ui: &mut egui::Ui,
        renderer: &VRenderer,
        ui_renderer: &mut egui_wgpu::Renderer,
        commands: &mut Commands,
    ) {
        ui.label("Current entities");

        for t in self.tasks.lock().unwrap().iter() {
            ui.label(*t);
        }

        // update cache of running tasks
        let t = self.tasks.clone();
        commands.add(move |w: &mut World| {
            let mut system_state: SystemState<(Query<&CommandTask>,)> = SystemState::new(w);

            let (ts,) = system_state.get_manual(w);

            let mut l = t.lock().unwrap();
            l.clear();
            for task in ts.iter() {
                l.push(task.name);
            }
        })
    }

    fn gui_label(&self) -> &str {
        "Task view"
    }
}
