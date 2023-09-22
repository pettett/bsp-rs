use std::sync::Arc;

use bevy_ecs::system::{Commands, Res, SystemState};

use bsp_explorer::{
    assets::{studio::vvd::Fixup, vpk::VPKFile},
    game_data::GameData,
    v::{vpath::VGlobalPath, vrenderer::VRenderer, vshader::VShader, VMesh},
    vertex::UVAlphaVertex,
    vinit, vrun,
};
use glam::Vec4;
fn fixup_remapping_search(fixupTable: &Box<[Fixup]>, dstIdx: u16) -> u16 {
    for i in 0..fixupTable.len() {
        let map = fixupTable[i];
        let idx = dstIdx as i32 - map.dst;
        if idx >= 0 && idx < map.count {
            return (map.src + idx) as u16;
        }
    }

    // remap did not copy over this vertex, return as is.
    return dstIdx;
}

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

    let Some(vtx) = game_data.load(
        &VGlobalPath::from("models/props_trainstation/train001.dx90.vtx"),
        VPKFile::vtx,
    ) else {
        panic!()
    };
    let Some(mdl) = game_data.load(
        &VGlobalPath::from("models/props_trainstation/train001.mdl"),
        VPKFile::mdl,
    ) else {
        panic!()
    };
    let Some(vvd) = game_data.load(
        &VGlobalPath::from("models/props_trainstation/train001.vvd"),
        VPKFile::vvd,
    ) else {
        panic!()
    };
    let l = vtx.header.num_lods as usize;

    assert_eq!(l, vtx.body[0].0[0].0.len());

    let lod0 = &vtx.body[0].0[0].0[0];

    let verts = vvd
        .verts
        .iter()
        .map(|v| UVAlphaVertex {
            position: v.pos,
            uv: v.uv,
            alpha: 1.0,
        })
        .collect::<Vec<_>>();

    let shader_tex = Arc::new(VShader::new_triangle_strip::<UVAlphaVertex>(&renderer));

    'outer: for m in &lod0.0 {
        println!("Mesh {:?}", m.flags);

        for strip_group in &m.strip_groups {
            let mut indices = strip_group.indices.clone();
            if vvd.fixups.len() > 0 {
                let mut map_dsts = vec![0; vvd.fixups.len()];

                for i in 1..vvd.fixups.len() {
                    map_dsts[i] = map_dsts[i - 1] + vvd.fixups[i - 1].count;
                }
                println!("{:?}", map_dsts);
                println!("{:?}", vvd.fixups[0]);

                for index in indices.iter_mut() {
                    *index = fixup_remapping_search(
                        &vvd.fixups,
                        strip_group.verts[*index as usize].orig_mesh_vert_id,
                    );
                }
            } else {
                for index in indices.iter_mut() {
                    *index = strip_group.verts[*index as usize].orig_mesh_vert_id;
                }
            }

            for s in &strip_group.strips {
                let ind_start = s.header.index_offset as usize;
                let ind_count = s.header.num_indices as usize;

                let m = VMesh::new(
                    renderer.device(),
                    &verts[..],
                    &indices[ind_start..ind_start + ind_count],
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

    vrun(state, event_loop);
}
