pub mod vmt_asset_loader;
pub mod vpk_asset_reader;
pub mod vtf_asset_loader;

use core::f32;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader, Cursor, Read},
    mem,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use bevy::{
    asset::{
        io::{
            AssetReader, AssetReaderError, AssetSource, AssetSourceId, ErasedAssetReader, Reader,
            SliceReader, VecReader,
        },
        AssetLoader, AssetPath, AsyncReadExt, LoadContext,
    },
    color::palettes::css::WHITE,
    math::VectorSpace,
    prelude::*,
    render::{
        mesh::{MeshVertexAttribute, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::Texture,
        renderer::{RenderDevice, RenderQueue},
        texture::{
            GpuImage, ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor,
        },
    },
    tasks::futures_lite::{io::Take, AsyncRead, AsyncSeek, AsyncSeekExt},
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use common::{
    vertex::{UVAlphaVertex, UVVertex},
    vfile::VFileSystem,
    vpath::{VGlobalPath, VLocalPath, VPath, VSplitPath},
};
use egui::{load::SizedTexture, vec2, ImageSource};
use glam::{ivec2, uvec2, vec3};
use ini::Ini;
use smooth_bevy_cameras::{
    controllers::{
        fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
        unreal::{UnrealCameraController, UnrealCameraPlugin},
    },
    LookTransform, LookTransformBundle, LookTransformPlugin, Smoother,
};
use source::{
    bsp::gamelump::{load_gamelump, GameLump},
    meshes::build_meshes,
    prelude::*,
    studio::vvd::Fixup,
};
use vmt_asset_loader::VMTAssetLoader;
use vpk_asset_reader::VPKAssetReader;
use vtf_asset_loader::VTFAssetLoader;
use wgpu::{Extent3d, TextureViewDescriptor};

#[derive(Resource)]
pub struct GameDataArc {
    pub inner: Arc<GameData>,
}

#[derive(Component)]
struct SourceObject {
    egui: Option<egui::TextureId>,
    // open: bool,
}

fn main() {
    App::new()
        .register_asset_source(
            "vpk",
            AssetSource::build().with_reader(|| {
                Box::new(VPKAssetReader::new(AssetSource::get_default_reader(
                    "assets".to_string(),
                )()))
            }),
        )
        .add_plugins(DefaultPlugins)
        // .init_asset::<VMTAsset>()
        .init_asset_loader::<VMTAssetLoader>()
        // .init_asset::<VTFAsset>()
        .init_asset_loader::<VTFAssetLoader>()
        .add_plugins(EguiPlugin)
        .add_plugins(LookTransformPlugin)
        .add_plugins(UnrealCameraPlugin {
            override_input_system: false,
        })
        // Systems that create Egui widgets should be run during the `CoreSet::Update` set,
        // or after the `EguiSet::BeginFrame` system (which belongs to the `CoreSet::PreUpdate` set).
        // .add_systems(Update, ui_example_system)
        .add_systems(Startup, load)
        .run();
}

fn builder_to_mesh(
    builder: &source::meshes::MeshBuilder<UVVertex>,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> Handle<Mesh> {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        builder
            .verts()
            .iter()
            .map(|v| v.position.to_array())
            .collect::<Vec<_>>(),
    );

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        builder
            .verts()
            .iter()
            .map(|v| v.uv.to_array())
            .collect::<Vec<_>>(),
    );

    // mesh.insert_attribute(
    //     Mesh::ATTRIBUTE_COLOR,
    //     builder.verts().iter().map(|v| v.color.to_array()).collect::<Vec<_>>(),
    // );
    mesh.insert_indices(bevy::render::mesh::Indices::U16(builder.tris().to_vec()));

    meshes.add(mesh)
}

fn builder_to_mesh2(
    verts: &[UVAlphaVertex],
    indices: &[u16],
    meshes: &mut ResMut<Assets<Mesh>>,
) -> Handle<Mesh> {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        verts
            .iter()
            .map(|v| v.position.to_array())
            .collect::<Vec<_>>(),
    );

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        verts.iter().map(|v| v.uv.to_array()).collect::<Vec<_>>(),
    );

    // mesh.insert_attribute(
    //     Mesh::ATTRIBUTE_COLOR,
    //     builder.verts().iter().map(|v| v.color.to_array()).collect::<Vec<_>>(),
    // );
    mesh.insert_indices(bevy::render::mesh::Indices::U16(indices.to_vec()));

    meshes.add(mesh)
}

pub fn load_vmesh(
    mdl_path: &dyn VPath,
    game_data: &GameData,
    asset_server: &AssetServer,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> Result<(Handle<Mesh>, Handle<StandardMaterial>), &'static str> {
    let mdl = game_data
        .load(mdl_path, VPKFile::mdl)
        .ok_or("No mdl file")?;

    let dir = mdl_path.dir();

    let mut mat_dir = "materials/".to_owned();
    mat_dir.push_str(&dir);

    let mut vtx_filename = mdl_path.filename().to_owned();
    vtx_filename.push_str(".dx90");
    //println!("{vtx_filename}");
    let vtx_path = VSplitPath::new(&dir, &vtx_filename, "vtx");
    let vvd_path = VSplitPath::new(&dir, mdl_path.filename(), "vvd");

    let vtx = game_data
        .load(&vtx_path, VPKFile::vtx)
        .ok_or("No VTX File")?;

    let vvd = game_data
        .load(&vvd_path, VPKFile::vvd)
        .ok_or("No VVD File")?;

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

    for m in &lod0.0 {
        //println!("Mesh {:?}", m.flags);

        for strip_group in &m.strip_groups {
            let mut indices = strip_group.indices.clone();
            if vvd.fixups.len() > 0 {
                let mut map_dsts = vec![0; vvd.fixups.len()];

                for i in 1..vvd.fixups.len() {
                    map_dsts[i] = map_dsts[i - 1] + vvd.fixups[i - 1].count;
                }
                //println!("{:?}", map_dsts);
                //println!("{:?}", vvd.fixups[0]);

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

            // reverse face order (cause its easy)
            for tri in indices.chunks_exact_mut(3) {
                tri.swap(1, 2);
            }

            for s in &strip_group.strips {
                let ind_start = s.header.index_offset as usize;
                let ind_count = s.header.num_indices as usize;

                // let mut m = VMesh::new(
                //     &instance.device,
                //     &verts[..],
                //     &indices[ind_start..ind_start + ind_count],
                //     shader_tex,
                // );

                let mat =
                    asset_server.load(format!("vpk://{mat_dir}/{}.vmt", mdl.textures[0].name));

                let mesh =
                    builder_to_mesh2(&verts, &indices[ind_start..ind_start + ind_count], meshes);
                // let image = vtf_to_image(vtf, images);

                return Ok((mesh, mat));
            }
        }
    }
    Err("No mesh in LODs")
}

pub fn fixup_remapping_search(fixup_table: &Box<[Fixup]>, dst_idx: u16) -> u16 {
    for i in 0..fixup_table.len() {
        let map = fixup_table[i];
        let idx = dst_idx as i32 - map.dst;
        if idx >= 0 && idx < map.count {
            return (map.src + idx) as u16;
        }
    }

    // remap did not copy over this vertex, return as is.
    return dst_idx;
}

fn load(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut vmt_materials: ResMut<Assets<VMTAsset>>,
) {
    let ini = Ini::load_from_file("conf.ini").unwrap();
    let game_data = GameData::from_ini(&ini);

    // println!("{:?}",game_data.dirs()[0].files);

    let (header, mut buffer) = BSPHeader::load(&game_data.starter_map()).unwrap();

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

    let gamelump = load_gamelump(header.get_lump_header(LumpType::GameLump), &mut buffer).unwrap();

    let mut lighting_cols: Vec<Vec4> = lighting.iter().map(|&x| x.into()).collect();

    let pak_header = header.get_lump_header(LumpType::PakFile);
    let pak_vpk: VPKDirectory = pak_header.read_binary(&mut buffer).unwrap();

    //let pak: VPKDirectory = VPKDirectory::read(&mut buffer, files, "".into()).unwrap();

    if lighting_cols.len() == 0 {
        let entries = 500;

        // println!("Failure loading lightmap, replacing with {entries} Vec4::ONE entries");

        // println!("Example face:\n {:#?}", faces[0]);
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

    let scene = commands
        .spawn((SpatialBundle {
            transform: Transform::from_rotation(Quat::from_rotation_x(-90_f32.to_radians()))
                .with_scale(Vec3::ONE * 0.01),
            ..default()
        },))
        .id();

    for prop in &gamelump.props {
        let path = gamelump.static_prop_names[prop.prop_type as usize].as_str();

        // println!("{}", path);

        let Ok((mesh, material)) = load_vmesh(
            &VGlobalPath::new(path),
            &game_data,
            &asset_server,
            &mut meshes,
        ) else {
            log::warn!("Failed to load prop mesh");
            continue;
        };

        let a = prop.angles * f32::consts::PI / 180.0;
        let rot = Quat::from_axis_angle(Vec3::Z, a.y)
            * Quat::from_axis_angle(Vec3::X, a.z)
            * Quat::from_axis_angle(Vec3::Y, a.x);

        let transform = Transform::from_translation(prop.m_origin.into()).with_rotation(rot);

        let obj = commands
            .spawn((
                SourceObject { egui: None },
                MaterialMeshBundle::<StandardMaterial> {
                    mesh,
                    material,
                    transform,
                    ..default()
                },
            ))
            .id();
        commands.entity(scene).push_children(&[obj]);
    }

    for (material, builder) in textured_tris {
        let mesh = builder_to_mesh(&builder, &mut meshes);

        let Some(mat_name) = material_name_map.get(&material) else {
            log::warn!("Failed to load texture {material}");
            continue;
        };

        // let pak_header = header.get_lump_header(LumpType::PakFile);
        // let pak: Arc<VPKDirectory> = Arc::new(pak_header.read_binary(&mut buffer).unwrap());
        // let pak = &game_data.dirs()[0];

        let mat_path = VLocalPath::new("materials", mat_name, "vmt");

        let vmt_path = PathBuf::from(format!("materials/{}.vmt", mat_name));
        let source = AssetSourceId::from("vpk");

        let material: Option<Handle<StandardMaterial>> = match pak_vpk.load_vmt(&mat_path) {
            Ok(vmt) => {
                if vmt.shader() == "patch" {
                    if let Some(inc) = vmt.get("include") {
                        Some(
                            asset_server
                                .load(AssetPath::from_path(Path::new(inc)).with_source(source)),
                        )
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Err(err) => {
                Some(asset_server.load(AssetPath::from_path(&vmt_path).with_source(source)))
            }
        };
        if let Some(material) = material {
            let obj = commands
                .spawn((
                    SourceObject { egui: None },
                    MaterialMeshBundle::<StandardMaterial> {
                        mesh,
                        material,
                        ..default()
                    },
                ))
                .id();

            commands.entity(scene).push_children(&[obj]);
        }

        // vmt_materials.get(vmt.id()).unwrap();

        // let pak_vmt = game_data.load_vmt(&mat_path);

        // let vmt = if let Some(pak_vmt) = pak_vmt {
        //     if pak_vmt.shader() == "patch" {
        //         // If this is a patch, link it to the other patch
        //         pak_vmt.patch.get_or_init(|| {
        //             if let Some(include) = pak_vmt.data.get("include") {
        //                 game_data
        //                     .load_vmt(&VGlobalPath::from(include.as_str()))
        //                     .map(Clone::clone)
        //             } else {
        //                 None
        //             }
        //         });
        //     }

        //     Some(pak_vmt)
        // } else {
        //     game_data.load_vmt(&mat_path)
        // };

        // let Some(vmt) = vmt else {
        //     log::warn!("Failed to load VMT for texture {material}");
        //     continue;
        // };

        // //Load shader

        // let shader_textures = ["$basetexture"];

        // // let (shader, shader_textures) = match vmt.shader() {
        // // 	"patch" => match vmt.patch.get() {
        // // 		Some(Some(vmt_patch)) => match vmt_patch.shader() {
        // // 			"lightmappedgeneric" => (shaders.shader_tex.clone(), vec!["$basetexture"]),
        // // 			"unlittwotexture" => (shaders.shader_tex.clone(), vec!["$basetexture"]),
        // // 			"worldvertextransition" => (
        // // 				shaders.shader_disp.clone(),
        // // 				vec!["$basetexture2", "$basetexture"],
        // // 			), // displacement - TODO: Include envmap

        // // 			x => {
        // // 				println!(
        // // 					"Unknown patched shader {x} - Patch:\n {:#?}\n Original:\n {:#?}",
        // // 					vmt.data,
        // // 					vmt.patch.get().as_ref().unwrap().as_ref().unwrap().data
        // // 				);
        // // 				(shaders.shader_lines.clone(), vec![])
        // // 			}
        // // 		},
        // // 		_ => {
        // // 			println!("ERROR: No patch vmt");
        // // 			(shaders.shader_lines.clone(), vec![])
        // // 		}
        // // 	}, //normal brushes with lightmap
        // // 	"lightmappedgeneric" => (shaders.shader_tex.clone(), vec!["$basetexture"]), // normal brushes
        // // 	"unlittwotexture" => (shaders.shader_tex.clone(), vec!["$basetexture"]),    // screens
        // // 	"unlitgeneric" => (shaders.shader_tex.clone(), vec!["$basetexture"]),       // glass?
        // // 	"worldvertextransition" => (
        // // 		shaders.shader_disp.clone(),
        // // 		vec!["$basetexture2", "$basetexture"],
        // // 	), // displacement
        // // 	x => {
        // // 		println!("Unknown shader {x}");
        // // 		(shaders.shader_lines.clone(), vec![])
        // // 	}
        // // };

        // // for (i, tex) in shader_textures.iter().enumerate() {

        // let tex = shader_textures[0];

        // let tex_path = {
        //     let Some(tex_path) = vmt.get(tex) else {
        //         println!("ERROR: Could not find {} texture for {:?}", tex, vmt);
        //         continue;
        //     };

        //     tex_path.replace('\\', "/")
        // };

        // // let vtf_path = VLocalPath::new("materials", &tex_path, "vtf");
        // // let vtf = if let Some(vtf) = game_data.load_vtf(&vtf_path) {
        // //     vtf
        // // } else {
        // //     match game_data.load_vtf(&vtf_path) {
        // //         Some(vtf) => vtf,
        // //         None => {
        // //             println!("ERROR:  Could not find vtf for {tex}: <{tex_path}>");
        // //             continue;
        // //         }
        // //     }
        // // };
        // // if vtf.high_res_data().len() > 0 {
        // //     let image = vtf_to_image(vtf, &mut images);

        // let vtf_path = PathBuf::from(format!("materials/{}.vtf", mat_name));
        // let source = AssetSourceId::from("vpk");
        // let asset_path = AssetPath::from_path(&vtf_path).with_source(source);

        // let image: Handle<Image> = asset_server.load(asset_path);

        // //             let image = asset_server.load("null.png");
        // // images.get(image)

        // // println!("{}", image.is_compressed());
    }

    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(4.0)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::srgb_u8(124, 144, 255)),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // light
    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         shadows_enabled: false,
    //         ..default()
    //     },
    //     transform: Transform::from_xyz(4.0, 8.0, 4.0),
    //     ..default()
    // });
    // ambient light
    commands.insert_resource(AmbientLight {
        color: WHITE.into(),
        brightness: 100.0,
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        UnrealCameraController::default(),
        LookTransformBundle {
            transform: LookTransform::new(vec3(-2.5, 4.5, 9.0), Vec3::ZERO, Vec3::Y),
            smoother: Smoother::new(00.1), // Value between 0.0 and 1.0, higher is smoother.
        },
    ));

    // println!("{:?c}", textured_tris.len());

    // commands.insert_resource(GameDataArc {
    //     inner: Arc::new(game_data),
    // });
}

// fn ui_example_system(
//     mut contexts: EguiContexts,
//     mut objects: Query<(&mut SourceObject, &Handle<StandardMaterial>)>,
//     materials: Res<Assets<StandardMaterial>>,
// ) {
//     for (mut src, mat) in objects.iter_mut() {
//         if src.egui.is_none() {
//             src.egui = Some(
//                 contexts.add_image(
//                     materials
//                         .get(mat)
//                         .as_ref()
//                         .unwrap()
//                         .base_color_texture
//                         .as_ref()
//                         .unwrap()
//                         .clone_weak(),
//                 ),
//             );
//         }
//     }

//     if let Some(ctx) = contexts.try_ctx_mut() {
//         egui::Window::new("Textures").show(ctx, |ui| {
//             egui::ScrollArea::new([false, true]).show(ui, |ui| {
//                 for (src, mat) in objects.iter() {
//                     ui.add(egui::widgets::Image::new(ImageSource::Texture(
//                         SizedTexture {
//                             id: src.egui.unwrap(),
//                             size: vec2(100., 100.),
//                         },
//                     )));
//                 }
//             });
//         });
//     }
// }
