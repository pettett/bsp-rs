use std::sync::Arc;

use bevy_ecs::system::{Commands, Res, SystemState};

use bsp_explorer::{
    assets::studio::load_vmesh,
    game_data::GameData,
    geo::InstancedProp,
    v::{vpath::VGlobalPath, vrenderer::VRenderer, vshader::VShader},
    vertex::UVAlphaVertex,
    vinit, vrun,
};
use glam::{Mat4, Vec3, Vec4};

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
    let prop_shader = Arc::new(VShader::new_instanced_prop::<UVAlphaVertex, Mat4>(
        &renderer,
    ));
    let m = load_vmesh(
        &VGlobalPath::from("models/props_trainstation/train001.mdl"),
        &renderer,
        prop_shader,
        &game_data,
    )
    .unwrap();

    let mut i = InstancedProp::default();

    i.transforms.push(Mat4::IDENTITY);
    i.transforms.push(Mat4::from_translation(Vec3::X * 10.0));

    commands.spawn((m, i));

    // Create a lighting buffer for use in all shaders
    bsp_explorer::assets::bsp::loader::insert_lighting_buffer(
        &mut commands,
        &[Vec4::ONE],
        &renderer,
    );

    system_state.apply(state.world_mut());

    vrun(state, event_loop);
}
