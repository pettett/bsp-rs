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
    game_data::{GameData, GameDataArc},
    gui::state_imgui::StateImgui,
    state::{box_cmds, command_task, spawn_command_task, MapChangeEvent, StateApp},
    v::vrenderer::VRenderer,
    vinit, vrun,
};
use ini::Ini;

pub fn main() {
    println!("Starting...");

    let (mut state, event_loop) = pollster::block_on(vinit());

    init_world(&mut state);

    vrun(state, event_loop);
}

fn init_world(state: &mut StateApp) {
    state.world_mut().spawn(command_task(|| {
        let ini = Ini::load_from_file("conf.ini").unwrap();

        let game_data = GameData::from_ini(&ini);

        box_cmds(|commands| {
            let start_map = game_data.inner.starter_map().to_owned();
            commands.add(|w: &mut World| w.send_event(MapChangeEvent(start_map)));

            commands.insert_resource(game_data);
        })
    }));

    println!("Loaded game data");
}
