use bevy_ecs::system::{Commands, Res, ResMut, SystemState};
use bsp_explorer::{
    bsp::{
        consts::LumpType,
        displacement::{BSPDispInfo, BSPDispVert},
        edges::{BSPEdge, BSPSurfEdge},
        face::BSPFace,
        header::BSPHeader,
        pak::BSPPak,
        textures::{BSPTexData, BSPTexDataStringTable, BSPTexInfo},
    },
    shader::Shader,
    state::StateRenderer,
    util::v_path::{VGlobalPath, VLocalPath},
    vertex::{UVAlphaVertex, UVVertex, Vertex},
    vmt::VMT,
};
use glam::{vec2, vec3, Vec3, Vec4};
use rayon::prelude::*;
use std::{collections::HashMap, path::Path, sync::Arc};

use bsp_explorer::{run, state_mesh::StateMesh};

pub fn main() {
    println!("Starting...");
    pollster::block_on(run(|state| {
        // Construct a `SystemState` struct, passing in a tuple of `SystemParam`
        // as if you were writing an ordinary system.
        let mut system_state: SystemState<(Commands, ResMut<StateRenderer>)> =
            SystemState::new(state.world_mut());

        // Use system_state.get_mut(&mut world) and unpack your system parameters into variables!
        // system_state.get(&world) provides read-only versions of your system parameters instead.
        let (mut commands, mut renderer) = system_state.get_mut(state.world_mut());

        bsp_explorer::bsp::loader::load_bsp(Path::new("D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\maps\\d1_trainstation_02.bsp" ) ,&mut commands, &mut renderer);

        system_state.apply(state.world_mut());
    }));
}
