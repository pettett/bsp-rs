use crate::geo::Static;
use crate::v::VMesh;
use common::prelude::*;
use rayon::prelude::*;
use source::bsp::gamelump::load_gamelump;
use source::{bsp::gamelump::GameLump, prelude::*};

use crate::{
    geo::{InstancedProp, PropInstance},
    state::{box_cmds, spawn_command_task, CommandTaskResult},
    transform::Transform,
    v::{vmesh::load_vmesh, vrenderer::VRenderer},
};

use bevy_ecs::system::Commands;
use glam::{ivec3, vec2, IVec3, Mat4, Quat, Vec3, Vec4};
use std::{
    collections::HashMap,
    f32::consts::PI,
    fs::File,
    io::{BufReader, Read, Seek},
    path::{Path, PathBuf},
    sync::Arc,
};
use wgpu::util::DeviceExt;


pub fn load_bsp(
    map: &Path,
    commands: &mut Commands,
    game_data: Arc<GameData>,
    renderer: &VRenderer,
    file_system_opt: Option<VFileSystem>,
) {
    let map_path = game_data.maps().join(map);
    let instance = renderer.instance();
    spawn_command_task(commands, "Loading map", move || {
        load_bsp_file_task(map_path, game_data, instance, file_system_opt)
    });
}

struct Shaders {
    shader_lines: Arc<VShader>,
    shader_tex: Arc<VShader>,
    _shader_tex_envmap: Arc<VShader>,
    shader_disp: Arc<VShader>,
    prop_shader: Arc<VShader>,
}
fn load_bsp_file_task(
    map_path: PathBuf,
    game_data: Arc<GameData>,
    instance: Arc<StateInstance>,
    file_system_opt: Option<VFileSystem>,
) -> CommandTaskResult {
    match file_system_opt {
        Some(file_system) => {
            let (header, mut buffer) = BSPHeader::load_file(&map_path, &file_system).unwrap();
            load_bsp_task(game_data, instance, header, buffer)
        }
        None => {
            #[cfg(target_arch = "x86_64")]
            {
                let file = File::open(map_path).unwrap();
                let mut buffer = BufReader::new(file);
                let header = BSPHeader::load_buf(&mut buffer).unwrap();
                return load_bsp_task(game_data, instance, header, buffer);
            }
            panic!("Failed to load bsp without desktop")
        }
    }
}

fn load_bsp_task(
    game_data: Arc<GameData>,
    instance: Arc<StateInstance>,
    header: BSPHeader,
    mut buffer: BufReader<impl Seek + Read>,
) -> CommandTaskResult {
    header.validate();

    {
        let v = header.version;
        println!("Loaded BSP File version {v}");
    }

    let shader_lines = Arc::new(VShader::new_white_lines::<Vec3>(&instance));
    let shader_tex = Arc::new(VShader::new_textured(&instance));
    let _shader_tex_envmap = Arc::new(VShader::new_textured_envmap(&instance));
    let shader_disp = Arc::new(VShader::new_displacement(&instance));
    let prop_shader = Arc::new(VShader::new_instanced_prop::<UVAlphaVertex, PropInstance>(
        &instance,
    ));

    let shaders = Arc::new(Shaders {
        shader_lines,
        shader_tex,
        _shader_tex_envmap,
        shader_disp,
        prop_shader,
    });

    //let mut mesh = StateMesh::new(renderer, wgpu::PrimitiveTopology::TriangleList);
    //mesh.load_glb_mesh(instance.clone());
    //state.add_mesh(mesh);

    //let mut mesh = StateMesh::new(renderer, wgpu::PrimitiveTopology::LineList);
    //mesh.load_debug_edges(instance.clone(), &header, &mut buffer);
    //state.add_mesh(mesh);

    let faces = header.get_lump::<BSPFace>(&mut buffer);
    let surf_edges = header.get_lump::<BSPSurfEdge>(&mut buffer);
    let edges = header.get_lump::<BSPEdge>(&mut buffer);
    let verts = header.get_lump::<Vec3>(&mut buffer);
    let tex_info = header.get_lump::<BSPTexInfo>(&mut buffer);
    let tex_data = header.get_lump::<BSPTexData>(&mut buffer);

    //let mut annotated_verts = bytemuck::zeroed_slice_box::<UVVertex>(verts.len());

    //for i in 0..verts.len() {
    //    annotated_verts[i].position = verts[i];
    //}

    //let mut tris = Vec::<u16>::new();
    // for now, filter by texture of first face
    let infos = header.get_lump::<BSPDispInfo>(&mut buffer);
    let disp_verts = header.get_lump::<BSPDispVert>(&mut buffer);
    let lighting = header.get_lump::<ColorRGBExp32>(&mut buffer);

    let mut lighting_cols: Vec<Vec4> = lighting.iter().map(|&x| x.into()).collect();

    if lighting_cols.len() == 0 {
        let entries = 500;

        println!("Failure loading lightmap, replacing with {entries} Vec4::ONE entries");

        println!("Example face:\n {:#?}", faces[0]);
        lighting_cols = vec![Vec4::ONE; entries];
    }

    let textured_tris = build_meshes(
        faces,
        &verts,
        &disp_verts,
        &tex_info,
        &tex_data,
        &infos,
        &edges,
        &surf_edges,
    );

    let pak_header = header.get_lump_header(LumpType::PakFile);
    let pak: Arc<VPKDirectory> = Arc::new(pak_header.read_binary(&mut buffer).unwrap());

    let tex_data_string_table = header.get_lump::<BSPTexDataStringTable>(&mut buffer);
    let tex_data_string_data = header.get_lump_header(LumpType::TexDataStringData);

    let material_name_map: Arc<HashMap<_, _>> = Arc::new(
        textured_tris
            .iter()
            .map(|(tex, _tris)| {
                (
                    *tex,
                    tex_data_string_table[tex_data[*tex as usize].name_string_table_id as usize]
                        .get_filename(&mut buffer, tex_data_string_data),
                )
            })
            .collect(),
    );

    let _models = header.get_lump::<BSPModel>(&mut buffer);

    // for m in models.iter() {
    //     commands.spawn((
    //         VMesh::new_box(device, m.mins(), m.maxs(), shader_lines.clone()),
    //         Static(),
    //     ));
    // }
    let gamelump = load_gamelump(header.get_lump_header(LumpType::GameLump), &mut buffer).unwrap();

    box_cmds(move |commands| {
        // Create a lighting buffer for use in all shaders
        insert_lighting_buffer(commands, &lighting_cols[..], &instance);

        for (tex, builder) in textured_tris {
            let renderer = instance.clone();
            let game_data = game_data.clone();
            let shaders = shaders.clone();
            let pak = pak.clone();
            let material_name_map = material_name_map.clone();
            spawn_command_task(commands, "Loading Static", move || {
                load_static(
                    game_data,
                    renderer,
                    material_name_map,
                    tex,
                    builder,
                    pak,
                    shaders,
                )
            });
        }

        spawn_command_task(commands, "Loading game lump", move || {
            load_props(game_data, instance, gamelump, shaders)
        });
    })
}

pub fn insert_lighting_buffer(
    commands: &mut Commands,
    lighting_cols: &[Vec4],
    instance: &StateInstance,
) {
    let lighting_buffer = instance
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Lighting Buffer"),
            contents: bytemuck::cast_slice(lighting_cols),
            usage: wgpu::BufferUsages::STORAGE,
        });

    let lighting_bind_group = instance
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &instance.lighting_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: lighting_buffer.as_entire_binding(),
            }],
            label: Some("lighting_bind_group"),
        });

    //commands.insert_resource(VPK::<0>(pak));
    commands.insert_resource(LightingData {
        buffer: VBuffer {
            buffer: lighting_buffer,
            bind_group: lighting_bind_group,
        },
    });
}

fn load_props(
    game_data: Arc<GameData>,
    instance: Arc<StateInstance>,
    gamelump: GameLump,
    shaders: Arc<Shaders>,
) -> CommandTaskResult {
    let mut instances = HashMap::new();

    for prop in &gamelump.props {
        let path = gamelump.static_prop_names[prop.prop_type as usize].as_str();

        let m = load_vmesh(
            &VGlobalPath::new(&path),
            &instance,
            shaders.prop_shader.clone(),
            &game_data,
        );

        match m {
            Ok(m) => {
                // euler angles in radians
                let a = prop.angles * PI / 180.0;
                let rot = Quat::from_axis_angle(Vec3::Z, a.y)
                    * Quat::from_axis_angle(Vec3::X, a.z)
                    * Quat::from_axis_angle(Vec3::Y, a.x);

                let t = Transform::new(prop.m_origin.into(), rot);

                let mat = PropInstance {
                    transform: t.get_local_to_world(),
                };

                let e = instances
                    .entry(prop.prop_type)
                    .or_insert_with(|| (m, InstancedProp::default()));

                e.1.transforms.push(mat);

                // commands.spawn((
                //     m,
                //    ,
                // ));
            }
            Err(e) => println!("Error loading {path}: {e}"),
        }

        // commands.spawn((
        //     VMesh::new_box(
        //         device,
        //         prop.m_Origin + Vec3::ONE,
        //         prop.m_Origin - Vec3::ONE,
        //         shader_lines.clone(),
        //     ),
        //     Static(),
        // ));
    }

    box_cmds(|commands| {
        for (i, bundle) in instances {
            commands.spawn(bundle);
        }
    })
}

fn load_static(
    game_data: Arc<GameData>,
    instance: Arc<StateInstance>,
    material_name_map: Arc<HashMap<i32, String>>,
    material: i32,
    builder: MeshBuilder<UVVertex>,
    pak: Arc<VPKDirectory>,
    shaders: Arc<Shaders>,
) -> CommandTaskResult {
    // Load vmt
    let Some(mat_name) = material_name_map.get(&material) else {
        return box_cmds(move |commands| {});
    };

    let mat_path = VLocalPath::new("materials", mat_name, "vmt");
    let pak_vmt = pak.load_vmt(&mat_path);

    let vmt = if let Ok(pak_vmt) = pak_vmt {
        if pak_vmt.shader() == "patch" {
            // If this is a patch, link it to the other patch
            pak_vmt.patch.get_or_init(|| {
                if let Some(include) = pak_vmt.data.get("include") {
                    game_data
                        .load_vmt(&VGlobalPath::from(include.as_str()))
                        .map(Clone::clone)
                } else {
                    None
                }
            });
        }

        Some(pak_vmt)
    } else {
        game_data.load_vmt(&mat_path)
    };

    let Some(vmt) = vmt else {
        return box_cmds(move |commands| {});
    };

    //Load shader

    let (shader, shader_textures) = match vmt.shader() {
        "patch" => match vmt.patch.get() {
            Some(Some(vmt_patch)) => match vmt_patch.shader() {
                "lightmappedgeneric" => (shaders.shader_tex.clone(), vec!["$basetexture"]),
                "unlittwotexture" => (shaders.shader_tex.clone(), vec!["$basetexture"]),
                "worldvertextransition" => (
                    shaders.shader_disp.clone(),
                    vec!["$basetexture2", "$basetexture"],
                ), // displacement - TODO: Include envmap

                x => {
                    println!(
                        "Unknown patched shader {x} - Patch:\n {:#?}\n Original:\n {:#?}",
                        vmt.data,
                        vmt.patch.get().as_ref().unwrap().as_ref().unwrap().data
                    );
                    (shaders.shader_lines.clone(), vec![])
                }
            },
            _ => {
                println!("ERROR: No patch vmt");
                (shaders.shader_lines.clone(), vec![])
            }
        }, //normal brushes with lightmap
        "lightmappedgeneric" => (shaders.shader_tex.clone(), vec!["$basetexture"]), // normal brushes
        "unlittwotexture" => (shaders.shader_tex.clone(), vec!["$basetexture"]),    // screens
        "unlitgeneric" => (shaders.shader_tex.clone(), vec!["$basetexture"]),       // glass?
        "worldvertextransition" => (
            shaders.shader_disp.clone(),
            vec!["$basetexture2", "$basetexture"],
        ), // displacement
        x => {
            println!("Unknown shader {x}");
            (shaders.shader_lines.clone(), vec![])
        }
    };

    let mut mesh = VMesh::new_empty(&instance.device, shader);

    mesh.from_verts_and_tris(
        &instance.device,
        bytemuck::cast_slice(&builder.verts),
        bytemuck::cast_slice(&builder.tris),
        builder.tris.len() as u32,
    );
    let mut all_success = true;
    for (i, tex) in shader_textures.iter().enumerate() {
        let tex_path = {
            let Some(tex_path) = vmt.get(tex) else {
                println!("ERROR: Could not find {} texture for {:?}", tex, vmt);
                continue;
            };

            tex_path.replace('\\', "/")
        };

        let vtf_path = VLocalPath::new("materials", &tex_path, "vtf");

        let vtf = if let Some(vtf) = game_data.load_vtf(&vtf_path) {
            vtf
        } else {
            match pak.load_vtf(&vtf_path) {
                Ok(vtf) => vtf,
                Err(x) => {
                    println!("ERROR: {x} Could not find vtf for {tex}: <{tex_path}>");
                    continue;
                }
            }
        };

        if let Ok(high_res) = vtf.get_high_res(&instance) {
            mesh.load_tex(&instance.device, i as u32, high_res);
        } else {
            all_success = false;
            break;
        }
    }

    box_cmds(move |commands| {
        if all_success {
            commands.spawn((mesh, Static()));
        }
    })
}
