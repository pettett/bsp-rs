use std::{cell::RefCell, sync::Arc};

use bevy_ecs::system::{Commands, Res, SystemState};
use bsp_explorer::{
    game_data::GameData, gui::state_imgui::StateImgui, state::StateApp, v::vrenderer::VRenderer,
    vinit, vrun,
};
use ini::Ini;

pub fn main() {
    println!("Starting...");

    let (state, event_loop) = pollster::block_on(vinit());

    vrun(state, event_loop);
}

fn init_world(state_arc: Arc<RefCell<StateApp>>) {
    let mut state = state_arc.borrow_mut();

    let ini = Ini::load_from_file("conf.ini").unwrap();

    let game_data = GameData::from_ini(&ini);

    state.world_mut().insert_resource(game_data);

    // Construct a `SystemState` struct, passing in a tuple of `SystemParam`
    // as if you were writing an ordinary system.

    let mut system_state: SystemState<(Commands, Res<VRenderer>, Res<GameData>)> =
        SystemState::new(state.world_mut());

    // Use system_state.get_mut(&mut world) and unpack your system parameters into variables!
    // system_state.get(&world) provides read-only versions of your system parameters instead.
    let (mut commands, renderer, game_data) = system_state.get(state.world());

    bsp_explorer::assets::bsp::loader::load_bsp(
        game_data.starter_map(),
        &mut commands,
        &game_data,
        &renderer,
    );

    system_state.apply(state.world_mut());
}
