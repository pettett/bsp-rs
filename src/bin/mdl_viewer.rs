use bevy_ecs::system::{Commands, Res, SystemState};

use bsp_explorer::{
    assets::studio::load_vmesh,
    game_data::GameData,
    v::{vpath::VGlobalPath, vrenderer::VRenderer},
    vinit, vrun,
};
use glam::Vec4;

pub fn main() {
    println!("Starting...");

    let (mut state, event_loop) = pollster::block_on(vinit());

    // Construct a `SystemState` struct, passing in a tuple of `SystemParam`
    // as if you were writing an ordinary system.

    let mut system_state: SystemState<(Commands, Res<VRenderer>, Res<GameData>)> =
        SystemState::new(state.world_mut());

    // Use system_state.get_mut(&mut world) and unpack your system parameters into variables!
    // system_state.get(&world) provides read-only versions of your system parameters instead.
    let (mut commands, renderer, game_data) = system_state.get(state.world());

    let m = load_vmesh(
        &VGlobalPath::from("models/props_trainstation/train001.mdl"),
        &renderer,
        &game_data,
    )
    .unwrap();

    commands.spawn(m);

    // Create a lighting buffer for use in all shaders
    bsp_explorer::assets::bsp::loader::insert_lighting_buffer(
        &mut commands,
        &[Vec4::ONE],
        &renderer,
    );

    system_state.apply(state.world_mut());

    vrun(state, event_loop);
}
