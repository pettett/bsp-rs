use std::sync::Arc;

use bevy_ecs::system::{Commands, Res, SystemState};

use bsp_explorer::{
    assets::vpk::VPKFile,
    game_data::GameData,
    run,
    util::v_path::VGlobalPath,
    v::{vrenderer::VRenderer, vshader::VShader, VMesh},
    vertex::UVAlphaVertex,
};
use glam::Vec4;

pub fn main() {
    println!("Starting...");
    pollster::block_on(run(|state| {
        // Construct a `SystemState` struct, passing in a tuple of `SystemParam`
        // as if you were writing an ordinary system.

        let mut system_state: SystemState<(Commands, Res<VRenderer>, Res<GameData>)> =
            SystemState::new(state.world_mut());

        // Use system_state.get_mut(&mut world) and unpack your system parameters into variables!
        // system_state.get(&world) provides read-only versions of your system parameters instead.
        let (mut commands, renderer, game_data) = system_state.get(state.world());

        let Some(vtx) = game_data.load(
            &VGlobalPath::from("models/props_c17/bench01a.dx90.vtx"),
            VPKFile::vtx,
        ) else {
            panic!()
        };
        let Some(mdl) = game_data.load(
            &VGlobalPath::from("models/props_c17/bench01a.mdl"),
            VPKFile::mdl,
        ) else {
            panic!()
        };
        let Some(vvd) = game_data.load(
            &VGlobalPath::from("models/props_c17/bench01a.vvd"),
            VPKFile::vvd,
        ) else {
            panic!()
        };
        let l = vtx.header.num_lods as usize;

        assert_eq!(l, vtx.body[0].0[0].0.len());

        let lod0 = &vtx.body[0].0[0].0[0];

        let verts: Vec<UVAlphaVertex> = vvd
            .verts
            .iter()
            .map(|v| UVAlphaVertex {
                position: v.pos,
                uv: v.uv,
                alpha: 1.0,
            })
            .collect();

        let shader_tex = Arc::new(VShader::new_triangle_strip::<UVAlphaVertex>(&renderer));
        'outer: for m in &lod0.0 {
            println!("Mesh");
            for strip_group in &m.0 {
                println!("{:?}", strip_group.head.flags);

                for s in &strip_group.strips {
                    let ind_start = s.header.index_offset as usize;
                    let ind_count = s.header.num_indices as usize;

                    println!("{ind_start} {ind_count}");

                    let m = VMesh::new(
                        renderer.device(),
                        &verts[..],
                        &strip_group.indices[ind_start..ind_start + ind_count],
                        shader_tex.clone(),
                    );

                    commands.spawn(m);
                    break 'outer;
                }
            }
        }

        // Create a lighting buffer for use in all shaders
        bsp_explorer::assets::bsp::loader::insert_lighting_buffer(
            &mut commands,
            &[Vec4::ONE],
            &renderer,
        );

        system_state.apply(state.world_mut());
    }));
}
