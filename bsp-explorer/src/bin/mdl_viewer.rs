use std::sync::Arc;

use bevy_ecs::system::{Commands, NonSend, Res, SystemState};

use bsp_explorer::{
    geo::{InstancedProp, PropInstance},
    v::{vmesh::load_vmesh, vrenderer::VRenderer},
    vinit, vrun,
};
use common::{vertex::UVAlphaVertex, vpath::VGlobalPath, vshader::VShader};
use glam::{Mat4, Vec3, Vec4};
use ini::Ini;
use source::{game_data::GameDataArc, prelude::GameData};

pub fn main() {
    println!("Starting...");

    let (mut state, event_loop) = pollster::block_on(vinit());

    // Construct a `SystemState` struct, passing in a tuple of `SystemParam`
    // as if you were writing an ordinary system.

    let ini = Ini::load_from_file("conf.ini").unwrap();
    let game_data = GameData::from_ini(&ini);

    state.world_mut().insert_resource(game_data);

    let mut system_state: SystemState<(Commands, NonSend<VRenderer>, Res<GameDataArc>)> =
        SystemState::new(state.world_mut());

    // Use system_state.get_mut(&mut world) and unpack your system parameters into variables!
    // system_state.get(&world) provides read-only versions of your system parameters instead.
    let (mut commands, renderer, game_data) = system_state.get(state.world());
    let prop_shader = Arc::new(VShader::new_instanced_prop::<UVAlphaVertex, PropInstance>(
        &renderer.instance(),
    ));
    let m = load_vmesh(
        &VGlobalPath::from("models/props_trainstation/train001.mdl"),
        &renderer.instance,
        prop_shader,
        &game_data.inner,
    )
    .unwrap();

    let mut i = InstancedProp::default();

    i.transforms.push(PropInstance::new(Mat4::IDENTITY));
    i.transforms
        .push(PropInstance::new(Mat4::from_translation(Vec3::X * 10.0)));

    commands.spawn((m, i));

    // Create a lighting buffer for use in all shaders
    bsp_explorer::loader::insert_lighting_buffer(&mut commands, &[Vec4::ONE], &renderer.instance);

    system_state.apply(state.world_mut());

    vrun(state, event_loop);
}
