use crate::{
    bsp::{
        consts::LumpType,
        displacement::{BSPDispInfo, BSPDispVert},
        edges::{BSPEdge, BSPSurfEdge},
        face::BSPFace,
        header::BSPHeader,
        lightmap::{ColorRGBExp32, LightingData},
        model::BSPModel,
        textures::{BSPTexData, BSPTexDataStringTable, BSPTexInfo},
    },
    util::v_path::{VGlobalPath, VLocalPath},
    v::{vrenderer::VRenderer, vshader::VShader, VMesh},
    vertex::{UVAlphaVertex, UVVertex, Vertex},
    vmt::VMT,
    vpk::VPKDirectory,
};
use bevy_ecs::system::Commands;
use glam::{vec2, vec3, Vec3, Vec4};
use rayon::prelude::*;
use std::{collections::HashMap, num::NonZeroU32, path::Path, sync::Arc};
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
        color: Vec3,
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

pub fn load_bsp(map: &Path, commands: &mut Commands, renderer: &VRenderer) {
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

    let infos = header.get_lump::<BSPDispInfo>(&mut buffer);
    let disp_verts = header.get_lump::<BSPDispVert>(&mut buffer);
    let lighting = header.get_lump::<ColorRGBExp32>(&mut buffer);

    let lighting_cols: Vec<Vec4> = lighting.iter().map(|&x| x.into()).collect();

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

        let tex_s = tex.tex_s / data.width as f32;
        let tex_t = tex.tex_t / data.height as f32;

        let lightmap_s = tex.lightmap_s;
        let lightmap_t = tex.lightmap_t;

        if face.light_ofs == -1 {
            continue;
        }
        // light_ofs is a byte offset, and these are 4 byte structures
        assert_eq!(face.light_ofs % 4, 0);

        let light_base_index = face.light_ofs as usize / 4;

        let first_col = lighting[light_base_index];

        let lightmap_texture_mins_in_luxels = face.lightmap_texture_mins_in_luxels;
        let lightmap_texture_size_in_luxels = face.lightmap_texture_size_in_luxels + 1;

        let light_data = vec3(
            light_base_index as f32,
            lightmap_texture_size_in_luxels.x as f32,
            0.0,
        );

        if face.disp_info != -1 {
            // This is a displacement

            let info = infos[face.disp_info as usize];

            assert_eq!(info.map_face as usize, i_face);

            let face_verts = face.get_verts(&edges, &surfedges);

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
                        i as u16, pos, tex_s, tex_t, lightmap_s, lightmap_t, vert.alpha, light_data,
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
            let root_edge = surfedges[root_edge_index].get_edge(&edges);

            for i in 1..(face.num_edges as usize) {
                let edge = surfedges[root_edge_index + i].get_edge(&edges);

                let tri = [edge.0, root_edge.0, edge.1];
                for i in tri {
                    let l = builder.verts.len();
                    builder.add_vert(
                        i,
                        verts[i as usize],
                        tex_s,
                        tex_t,
                        lightmap_s,
                        lightmap_t,
                        1.0,
                        light_data,
                    );
                    let v = &mut builder.verts[l];

                    // The lightmapVecs float array performs a similar mapping of the lightmap samples of the
                    // texture onto the world. It is the same formula but with lightmapVecs instead of textureVecs,
                    // and then subtracting the [0] and [1] values of LightmapTextureMinsInLuxels for u and v respectively.
                    // LightmapTextureMinsInLuxels is referenced in dface_t;

                    v.lightmap_uv += 0.5 - lightmap_texture_mins_in_luxels.as_vec2();
                    //v.lightmap_uv /= lightmap_texture_size_in_luxels.as_vec2();
                }
                builder.push_tri();
            }
        }
    }

    println!("Loading BSP Pak...");
    let pak_header = header.get_lump_header(LumpType::PakFile);

    let pak: VPKDirectory = pak_header.read_binary(&mut buffer).unwrap();

    println!("Loading BSP Texture strings...");
    let tex_data_string_table = header.get_lump::<BSPTexDataStringTable>(&mut buffer);
    let tex_data_string_data = header.get_lump_header(LumpType::TexDataStringData);

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
    let materials: HashMap<i32, &Arc<VMT>> = textured_tris
        .par_iter()
        .filter_map(|(tex, _tris)| {
            let Some(mat_name) = material_name_map.get(tex) else {
                return None;
            };

            let mat_path = VLocalPath::new("materials", mat_name, "vmt");
            let pak_vmt = pak.load_vmt(&mat_path);

            let vmt = if let Ok(Some(pak_vmt)) = pak_vmt {
                if pak_vmt.shader() == "patch" {
                    // If this is a patch, link it to the other patch
                    pak_vmt.patch.get_or_init(|| {
                        renderer
                            .misc_dir()
                            .load_vmt(&VGlobalPath::from(pak_vmt.data["include"].as_str()))
                            .unwrap()
                            .map(Clone::clone)
                    });
                }

                Some(pak_vmt)
            } else {
                renderer.misc_dir().load_vmt(&mat_path).unwrap()
            };

            if let Some(vmt) = vmt {
                Some((*tex, vmt))
            } else {
                None
            }
        })
        .collect();

    println!("Loading BSP meshes...");
    let shader_lines = Arc::new(VShader::new_white_lines::<Vec3>(renderer));
    let shader_tex = Arc::new(VShader::new_textured(renderer));
    let shader_tex_envmap = Arc::new(VShader::new_textured_envmap(renderer));
    let shader_disp = Arc::new(VShader::new_displacement(renderer));

    for (tex, builder) in &textured_tris {
        let Some(vmt) = materials.get(tex) else {
            println!("Could not find material for {:?}", tex);
            continue;
        };

        let (shader, shader_textures) = match vmt.shader() {
            "patch" => match vmt.patch.get().as_ref().unwrap().as_ref().unwrap().shader() {
                "lightmappedgeneric" => {
                    (shader_tex_envmap.clone(), vec!["$basetexture", "$envmap"])
                }
                "unlittwotexture" => (shader_tex_envmap.clone(), vec!["$basetexture", "$envmap"]),
                "worldvertextransition" => {
                    (shader_disp.clone(), vec!["$basetexture2", "$basetexture"])
                } // displacement - TODO: Include envmap

                x => {
                    println!(
                        "Unknown patched shader {x} - Patch:\n {:#?}\n Original:\n {:#?}",
                        vmt.data,
                        vmt.patch.get().as_ref().unwrap().as_ref().unwrap().data
                    );
                    (shader_lines.clone(), vec![])
                }
            }, //normal brushes with lightmap
            "lightmappedgeneric" => (shader_tex.clone(), vec!["$basetexture"]), // normal brushes
            "unlittwotexture" => (shader_tex.clone(), vec!["$basetexture"]),    // screens
            "unlitgeneric" => (shader_tex.clone(), vec!["$basetexture"]),       // glass?
            "worldvertextransition" => (shader_disp.clone(), vec!["$basetexture2", "$basetexture"]), // displacement
            x => {
                println!("Unknown shader {x}");
                (shader_lines.clone(), vec![])
            }
        };

        let mut mesh = VMesh::new_empty(device, shader);

        mesh.from_verts_and_tris(
            device,
            bytemuck::cast_slice(&builder.verts),
            bytemuck::cast_slice(&builder.tris),
            builder.tris.len() as u32,
        );
        let mut all_success = true;
        for (i, tex) in shader_textures.iter().enumerate() {
            let Some(tex_path) = vmt.get(tex) else {
                println!("Could not find {} texture for {:?}", tex, vmt);
                continue;
            };

            let fixed_path = tex_path.replace('\\', "/");

            let vtf_path = VLocalPath::new("materials", &fixed_path, "vtf");

            let vtf = if let Ok(Some(vtf)) = renderer.texture_dir().load_vtf(&vtf_path) {
                vtf
            } else {
                if let Ok(Some(vtf)) = pak.load_vtf(&vtf_path) {
                    vtf
                } else {
                    println!("Could not find vtf for {}:{}", tex, tex_path);
                    continue;
                }
            };

            if let Ok(high_res) = vtf.get_high_res(device, renderer.queue()) {
                mesh.load_tex(device, i as u32, high_res);
            } else {
                all_success = false;
                break;
            }
        }
        if all_success {
            commands.spawn(mesh);
        }
    }

    let models = header.get_lump::<BSPModel>(&mut buffer);

    for m in models.iter() {
        commands.spawn(VMesh::new_box(
            device,
            m.mins(),
            m.maxs(),
            shader_lines.clone(),
        ));
    }

    // Create a lighting buffer for use in all shaders

    let lighting_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Lighting Buffer"),
        contents: bytemuck::cast_slice(&lighting_cols[..]),
        usage: wgpu::BufferUsages::STORAGE,
    });

    let lighting_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &renderer.lighting_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: lighting_buffer.as_entire_binding(),
        }],
        label: Some("lighting_bind_group"),
    });

    //commands.insert_resource(VPK::<0>(pak));
    commands.insert_resource(LightingData {
        lighting_buffer,
        lighting_bind_group,
    });
}
