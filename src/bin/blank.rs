use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
    thread,
};

use bevy_ecs::system::{Commands, Res, SystemState};
use bsp_explorer::{
    game_data::GameData, gui::gui::Gui, state::StateApp, v::vrenderer::VRenderer, vinit, vrun,
};
use ini::Ini;

pub fn main() {
    println!("Starting...");

    let (state, event_loop) = pollster::block_on(vinit());

    vrun(state, event_loop);
}
