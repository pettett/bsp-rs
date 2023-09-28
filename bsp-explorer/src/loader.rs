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

#[derive(Default)]
struct MeshBuilder<V: Vertex + Default> {
    tris: Vec<u16>,
    //tri_map: HashMap<u16, u16>,
    verts: Vec<V>,
}
impl MeshBuilder<UVAlphaVertex> {
    pub fn add_vert_a(&mut self, _index: u16, vertex: Vec3, s: Vec4, t: Vec4, alpha: f32) {
        //if !self.tri_map.contains_key(&index) {
        // if not contained, add in and generate uvs
        let u = s.dot(Vec4::from((vertex, 1.0)));
        let v = t.dot(Vec4::from((vertex, 1.0)));

        //self.tri_map.insert(index, self.verts.len() as u16);

        self.verts.push(UVAlphaVertex {
            position: vertex,
            uv: vec2(u, v),
            alpha,
        });
        //}
    }
}

impl MeshBuilder<UVVertex> {
    pub fn add_vert(
        &mut self,
        _index: u16,
        vertex: Vec3,
        tex_s: Vec4,
        tex_t: Vec4,
        lightmap_s: Vec4,
        lightmap_t: Vec4,
        alpha: f32,
        color: IVec3,
    ) {
        //if !self.tri_map.contains_key(&index) {
        // if not contained, add in and generate uvs
        let tex_u = tex_s.dot(Vec4::from((vertex, 1.0)));
        let tex_v = tex_t.dot(Vec4::from((vertex, 1.0)));
        let env_u = lightmap_s.dot(Vec4::from((vertex, 1.0)));
        let env_v = lightmap_t.dot(Vec4::from((vertex, 1.0)));

        //self.tri_map.insert(index, self.verts.len() as u16);

        self.verts.push(UVVertex {
            position: vertex,
            uv: vec2(tex_u, tex_v),
            lightmap_uv: vec2(env_u, env_v),
            alpha,
            color,
        });
        //}
    }
}

fn build_meshes(
    faces: Box<[BSPFace]>,
    verts: &Box<[Vec3]>,
    disp_verts: &Box<[BSPDispVert]>,
    tex_info: &Box<[BSPTexInfo]>,
    tex_data: &Box<[BSPTexData]>,
    infos: &Box<[BSPDispInfo]>,
    edges: &Box<[BSPEdge]>,
    surf_edges: &Box<[BSPSurfEdge]>,
) -> HashMap<i32, MeshBuilder<UVVertex>> {
    let mut textured_tris = HashMap::<i32, MeshBuilder<UVVertex>>::new();

    for (i_face, face) in faces.iter().enumerate() {
        let tex = tex_info[face.tex_info as usize];
        let i_texdata = tex.tex_data;
        let data = tex_data[i_texdata as usize];

        let builder = match textured_tris.get_mut(&i_texdata) {
            Some(x) => x,
            None => {
                textured_tris.insert(i_texdata, Default::default());
                textured_tris.get_mut(&i_texdata).unwrap()
            }
        };

        // TODO: better way to get tex/uv info from faces

        let tex_s: Vec4 = tex.tex_s.into();
        let tex_t: Vec4 = tex.tex_t.into();

        let tex_s = tex_s / data.width as f32;
        let tex_t = tex_t / data.height as f32;

        let lightmap_s = tex.lightmap_s;
        let lightmap_t = tex.lightmap_t;

        if face.light_ofs == -1 {
            continue;
        }
        // light_ofs is a byte offset, and these are 4 byte structures
        assert_eq!(face.light_ofs % 4, 0);

        let light_base_index = face.light_ofs as usize / 4;

        // Ensure we have the data
        //let Some(lighting) = lighting.get(light_base_index) else {
        //    panic!("Face has incorrect lighting data:\n {:#?}", face);
        //};

        let lightmap_texture_mins_in_luxels = face.lightmap_texture_mins_in_luxels;
        let lightmap_texture_size_in_luxels = face.lightmap_texture_size_in_luxels + 1;

        let light_data = ivec3(
            light_base_index as i32,
            lightmap_texture_size_in_luxels.x,
            0,
        );

        if face.disp_info != -1 {
            // This is a displacement

            let info = infos[face.disp_info as usize];

            assert_eq!(info.map_face as usize, i_face);

            let face_verts = face.get_verts(edges, surf_edges);

            let mut corners = [Vec3::ZERO; 4];
            for i in 0..4 {
                corners[i] = verts[face_verts[i]];
            }

            // TODO: better way to get tex/uv info from faces

            let _c = info.start_position;

            let disp_side_len = (1 << (info.power)) + 1;

            let get_i = |x: usize, y: usize| -> usize { x + disp_side_len * y };

            let old_vert_count = builder.verts.len() as u16;

            for y in 0..disp_side_len {
                let dy = y as f32 / (disp_side_len as f32 - 1.0);

                let v0 = Vec3::lerp(corners[0], corners[3], dy);
                let v1 = Vec3::lerp(corners[1], corners[2], dy);

                for x in 0..disp_side_len {
                    let dx = x as f32 / (disp_side_len as f32 - 1.0);

                    let i = get_i(x, y);

                    let vert = disp_verts[i + info.disp_vert_start as usize];

                    let pos = vert.vec + Vec3::lerp(v0, v1, dx);

                    builder.add_vert(
                        i as u16,
                        pos,
                        tex_s,
                        tex_t,
                        lightmap_s.into(),
                        lightmap_t.into(),
                        vert.alpha,
                        light_data,
                    );
                }
            }
            let disp_side_len = disp_side_len as u16;

            // Build grid index buffer.
            for y in 0..(disp_side_len - 1) {
                for x in 0..(disp_side_len - 1) {
                    let base = y * disp_side_len + x + old_vert_count;
                    builder.add_tri([base, base + disp_side_len, base + disp_side_len + 1]);
                    builder.add_tri([base, base + disp_side_len + 1, base + 1]);
                }
            }

            // assert_eq!(builder.tris.len() as u16, ((disp_side_len - 1).pow(2)) * 6);
        } else {
            let root_edge_index = face.first_edge as usize;
            let root_edge = surf_edges[root_edge_index].get_edge(edges);

            for i in 1..(face.num_edges as usize) {
                let edge = surf_edges[root_edge_index + i].get_edge(edges);

                let tri = [edge.0, root_edge.0, edge.1];
                for i in tri {
                    let l = builder.verts.len();
                    builder.add_vert(
                        i,
                        verts[i as usize],
                        tex_s,
                        tex_t,
                        lightmap_s.into(),
                        lightmap_t.into(),
                        1.0,
                        light_data,
                    );
                    let v = &mut builder.verts[l];

                    // The lightmapVecs float array performs a similar mapping of the lightmap samples of the
                    // texture onto the world. It is the same formula but with lightmapVecs instead of textureVecs,
                    // and then subtracting the [0] and [1] values of LightmapTextureMinsInLuxels for u and v respectively.
                    // LightmapTextureMinsInLuxels is referenced in dface_t;

                    v.lightmap_uv -= lightmap_texture_mins_in_luxels.as_vec2();
                    //v.lightmap_uv /= lightmap_texture_size_in_luxels.as_vec2();
                }
                builder.push_tri();
            }
        }
    }
    textured_tris
}

impl<V: Vertex + Default> MeshBuilder<V> {
    pub fn push_tri(&mut self) {
        for i in 0..3u16 {
            self.tris.push(self.verts.len() as u16 + i - 3);
        }
    }
    pub fn add_tri(&mut self, tri: [u16; 3]) {
        for i in 0..3 {
            self.tris.push(tri[i]);
        }
    }

    pub fn tris_to_lines(&self) -> Vec<u16> {
        let mut lines: Vec<u16> = Default::default();

        for i in (0..self.tris.len()).step_by(3) {
            lines.push(self.tris[i]);
            lines.push(self.tris[i + 1]);

            lines.push(self.tris[i + 1]);
            lines.push(self.tris[i + 2]);

            lines.push(self.tris[i + 2]);
            lines.push(self.tris[i + 3]);
        }

        lines
    }
}

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
