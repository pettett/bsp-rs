use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
    thread,
};

use bevy_ecs::{
    system::{Commands, Res, SystemState},
    world::World,
};
use bsp_explorer::{
    state::{box_cmds, command_task, spawn_command_task, MapChangeEvent, StateApp},
    v::vrenderer::VRenderer,
    vinit, vrun,
};

#[cfg(target_arch = "x86_64")]
use bsp_explorer::gui::{
    gui::{Gui, GuiWindow},
    map_select::MapSelect,
};

use ini::Ini;
use source::prelude::GameData;

pub fn main() {
    println!("Starting...");

    let (mut state, event_loop) = pollster::block_on(vinit());

    init_world(&mut state);

    vrun(state, event_loop);
}

fn init_world(state: &mut StateApp) {
    state
        .world_mut()
        .spawn(command_task("Loading game data", || {
            let ini = Ini::load_from_file("conf.ini").unwrap();

            let game_data = GameData::from_ini(&ini);

            box_cmds(|commands| {
                let start_map = game_data.inner.starter_map().to_owned();
                commands.add(|w: &mut World| w.send_event(MapChangeEvent(start_map)));

                #[cfg(target_arch = "x86_64")]
                for d in game_data.inner.dirs() {
                    commands.spawn(GuiWindow::new(d.clone()));
                }
                #[cfg(target_arch = "x86_64")]
                commands.spawn(GuiWindow::new(Arc::new(
                    MapSelect::new(game_data.inner.maps()).unwrap(),
                )));

                commands.insert_resource(game_data);
            })
        }));

    println!("Loaded game data");
}
