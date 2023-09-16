use crate::{
    bsp::{
        consts::LumpType,
        displacement::{BSPDispInfo, BSPDispVert},
        edges::{BSPEdge, BSPSurfEdge},
        face::BSPFace,
        header::BSPHeader,
        textures::{BSPTexData, BSPTexDataStringTable, BSPTexInfo},
    },
    shader::Shader,
    state::StateRenderer,
    state_mesh::StateMesh,
    util::v_path::{VGlobalPath, VLocalPath},
    vertex::{UVAlphaVertex, UVVertex, Vertex},
    vmt::VMT,
    vpk::VPKDirectory,
};
use bevy_ecs::system::Commands;
use glam::{vec2, vec3, Vec3, Vec4};
use rayon::prelude::*;
use std::{collections::HashMap, path::Path, sync::Arc};

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
    pub fn add_vert(&mut self, _index: u16, vertex: Vec3, s: Vec4, t: Vec4) {
        //if !self.tri_map.contains_key(&index) {
        // if not contained, add in and generate uvs
        let u = s.dot(Vec4::from((vertex, 1.0)));
        let v = t.dot(Vec4::from((vertex, 1.0)));

        //self.tri_map.insert(index, self.verts.len() as u16);

        self.verts.push(UVVertex {
            position: vertex,
            uv: vec2(u, v),
        });
        //}
    }
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

pub fn get_material<'a>(
    material_index: &i32,
    renderer: &'a StateRenderer,
    material_name_map: &HashMap<i32, String>,
    map_specific_material_map: &HashMap<&str, &str>,
) -> Option<&'a VMT> {
    let Some(material_name) = material_name_map.get(material_index).map(String::as_str) else {
        return None;
    };
    // Get material data

    let vmt_r = if let Some(&global_material_path) = map_specific_material_map.get(material_name) {
        let p: VGlobalPath = global_material_path.into();
        renderer.misc_dir().load_vmt(&p)
    } else {
        renderer
            .misc_dir()
            .load_vmt(&VLocalPath::new("materials", material_name, "vmt"))
    };

    match vmt_r {
        Ok(Some(vmt)) => Some(vmt),
        Ok(None) => {
            println!(
                "ERROR: Material {} does not have valid vmt data",
                material_name
            );
            None
        }
        Err(e) => {
            println!("ERROR: {}", e);
            None
        }
    }
}

pub fn load_bsp(map: &Path, commands: &mut Commands, renderer: &StateRenderer) {
    println!("Loading BSP File...");

    let device = renderer.device();

    let (header, mut buffer) = BSPHeader::load(map).unwrap();

    header.validate();

    //let mut mesh = StateMesh::new(renderer, wgpu::PrimitiveTopology::TriangleList);
    //mesh.load_glb_mesh(instance.clone());
    //state.add_mesh(mesh);

    //let mut mesh = StateMesh::new(renderer, wgpu::PrimitiveTopology::LineList);
    //mesh.load_debug_edges(instance.clone(), &header, &mut buffer);
    //state.add_mesh(mesh);

    println!("Loading BSP Headers...");

    let faces = header.get_lump::<BSPFace>(&mut buffer);
    let surfedges = header.get_lump::<BSPSurfEdge>(&mut buffer);
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

    println!("Loading BSP Faces...");
    let mut textured_tris = HashMap::<i32, MeshBuilder<UVVertex>>::new();

    for face in faces.iter() {
        if face.disp_info != -1 {
            // skip displacements
            continue;
        }

        let root_edge_index = face.first_edge as usize;
        let root_edge = surfedges[root_edge_index].get_edge(&edges);

        let tex = tex_info[face.tex_info as usize];
        let texdata = tex.tex_data;
        let data = tex_data[texdata as usize];

        let s = tex.tex_s / data.width as f32;
        let t = tex.tex_t / data.height as f32;

        for i in 1..(face.num_edges as usize) {
            let edge = surfedges[root_edge_index + i].get_edge(&edges);

            let builder = match textured_tris.get_mut(&texdata) {
                Some(x) => x,
                None => {
                    textured_tris.insert(texdata, Default::default());
                    textured_tris.get_mut(&texdata).unwrap()
                }
            };

            let tri = [edge.0, root_edge.0, edge.1];
            for i in tri {
                builder.add_vert(i, verts[i as usize], s, t);
            }
            builder.push_tri();
        }
    }

    println!("Loading BSP Pak...");
    let pak_header = header.get_lump_header(LumpType::PAKFILE);

    let pak: VPKDirectory = pak_header.read_binary(&mut buffer).unwrap();
    let map_specific_material_map: HashMap<&str, &str> = Default::default();
    // map map specific materials to global materials
    // let map_specific_material_map: HashMap<&str, &str> = pak
    //     .entries
    //     .par_iter()
    //     .filter_map(|entry| {
    //         if let Some(vmt) = entry.get_vmt() {
    //             if let Some(mat) = vmt.data.get("include") {
    //                 Some((entry.filename.as_str(), mat.as_str()))
    //             } else {
    //                 None
    //             }
    //         } else {
    //             None
    //         }
    //     })
    //     .collect();

    println!("Loading BSP Texture strings...");
    let tex_data_string_table = header.get_lump::<BSPTexDataStringTable>(&mut buffer);
    let tex_data_string_data = header.get_lump_header(LumpType::TexdataStringData);

    let material_name_map: HashMap<i32, String> = textured_tris
        .iter()
        .map(|(tex, _tris)| {
            (
                *tex,
                tex_data_string_table[tex_data[*tex as usize].name_string_table_id as usize]
                    .get_filename(&mut buffer, tex_data_string_data),
            )
        })
        .collect();

    println!("Loading BSP Materials...");
    let materials: HashMap<i32, Option<&VMT>> = textured_tris
        .par_iter()
        .filter_map(|(tex, _tris)| {
            material_name_map.get(tex).map(|tex_name| {
                println!("{tex_name}");

                let mat_path = VLocalPath::new("materials", tex_name, "vmt");

                let pak_vmt = pak.load_vmt(&mat_path);

                let vmt = if let Ok(Some(pak_vmt)) = pak_vmt {
                    renderer
                        .misc_dir()
                        .load_vmt(&VGlobalPath::from(pak_vmt.data["include"].as_str()))
                        .unwrap()
                } else {
                    renderer.misc_dir().load_vmt(&mat_path).unwrap()
                };

                (*tex, vmt)
            })
        })
        .collect();

    println!("Loading BSP meshes...");
    let shader_lines = Arc::new(Shader::new_white_lines::<Vec3>(renderer));
    let shader_tex = Arc::new(Shader::new_textured(renderer));
    let shader_disp = Arc::new(Shader::new_displacement(renderer));

    for (tex, builder) in &textured_tris {
        let Some(Some(vmt)) = materials.get(tex) else {
            println!("Could not find material for {:?}", tex);
            continue;
        };

        let Some(basetex) = vmt.get_tex_name() else {
            println!("Could not find base texture for {:?}", vmt);
            continue;
        };

        let Ok(Some(tex)) =
            renderer
                .texture_dir()
                .load_vtf(&VLocalPath::new("materials", &basetex, "vtf"))
        else {
            println!("Could not find texture for {:?}", materials.get(tex));
            continue;
        };

        let Ok(high_res) = tex.get_high_res(device, renderer.queue()) else {
            continue;
        };
        let mut mesh = StateMesh::new_empty(device, shader_tex.clone());

        mesh.from_verts_and_tris(
            device,
            bytemuck::cast_slice(&builder.verts),
            bytemuck::cast_slice(&builder.tris),
            builder.tris.len() as u32,
        );

        mesh.load_tex(device, 1, high_res);
        commands.spawn(mesh);
    }

    // Load displacement data

    println!("Loading BSP displacements...");
    let infos = header.get_lump::<BSPDispInfo>(&mut buffer);
    let disp_verts = header.get_lump::<BSPDispVert>(&mut buffer);

    for info in infos.iter() {
        // Build a mesh for the vertex

        let face = faces[info.map_face as usize];

        let face_verts = face.get_verts(&edges, &surfedges);

        let mut corners = [Vec3::ZERO; 4];
        for i in 0..4 {
            corners[i] = verts[face_verts[i]];
        }

        // TODO: better way to get tex/uv info from faces
        let tex = tex_info[face.tex_info as usize];
        let texdata = tex.tex_data;
        let data = tex_data[texdata as usize];

        let s = tex.tex_s / data.width as f32;
        let t = tex.tex_t / data.height as f32;

        if let Some(vmt) = get_material(
            &texdata,
            renderer,
            &material_name_map,
            &map_specific_material_map,
        ) {
            if let Some(Ok(Some(tex0))) = vmt.get_tex_name().map(|t| {
                renderer
                    .texture_dir()
                    .load_vtf(&VLocalPath::new("materials", &t, "vtf"))
            }) {
                if let Some(Ok(Some(tex1))) = vmt.get_tex2_name().map(|t| {
                    renderer
                        .texture_dir()
                        .load_vtf(&VLocalPath::new("materials", &t, "vtf"))
                }) {
                    let mut builder = MeshBuilder::default();

                    let _c = info.start_position;

                    let disp_side_len = (1 << (info.power)) + 1;

                    let get_i = |x: usize, y: usize| -> usize { x + disp_side_len * y };

                    for y in 0..disp_side_len {
                        let dy = y as f32 / (disp_side_len as f32 - 1.0);

                        let v0 = Vec3::lerp(corners[0], corners[3], dy);
                        let v1 = Vec3::lerp(corners[1], corners[2], dy);

                        for x in 0..disp_side_len {
                            let dx = x as f32 / (disp_side_len as f32 - 1.0);

                            let i = get_i(x, y);

                            let vert = disp_verts[i + info.disp_vert_start as usize];

                            let pos = vert.vec + Vec3::lerp(v0, v1, dx);

                            builder.add_vert_a(i as u16, pos, s, t, vert.alpha);
                        }
                    }
                    let disp_side_len = disp_side_len as u16;

                    // Build grid index buffer.
                    for y in 0..(disp_side_len - 1) {
                        for x in 0..(disp_side_len - 1) {
                            let base = y * disp_side_len + x;
                            builder.add_tri([base, base + disp_side_len, base + disp_side_len + 1]);
                            builder.add_tri([base, base + disp_side_len + 1, base + 1]);
                        }
                    }

                    assert_eq!(builder.tris.len() as u16, ((disp_side_len - 1).pow(2)) * 6);

                    let mut mesh = StateMesh::new_empty(device, shader_disp.clone());

                    mesh.from_verts_and_tris(
                        device,
                        bytemuck::cast_slice(&builder.verts),
                        bytemuck::cast_slice(&builder.tris),
                        builder.tris.len() as u32,
                    );

                    let Ok(high_res1) = tex0.get_high_res(device, renderer.queue()) else {
                        continue;
                    };
                    let Ok(high_res2) = tex1.get_high_res(device, renderer.queue()) else {
                        continue;
                    };
                    mesh.load_tex(device, 1, high_res1);
                    mesh.load_tex(device, 2, high_res2);

                    commands.spawn((mesh,));
                }
            }
        } else {
            println!("Missing material for a displacement");
        }
    }
    //state.add_mesh(StateMesh::new_box(
    //    r,
    //    vec3(0., 0., 0.),
    //    vec3(1000., 1000., 1000.),
    //    shader_lines,
    //));

    //commands.insert_resource(pak);
    //renderer.pak = Some(pak);
}
