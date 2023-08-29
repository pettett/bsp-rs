use bsp_explorer::{
    bsp::{
        consts::LumpType,
        edges::{BSPEdge, BSPSurfEdge},
        face::BSPFace,
        header::BSPHeader,
        pak::BSPPak,
        textures::{BSPTexData, BSPTexDataStringTable, BSPTexInfo},
    },
    state::State,
    vertex::UVVertex,
};
use glam::{vec2, Vec3, Vec4};
use rayon::prelude::*;
use std::{collections::HashMap, hash::Hash, thread};
use stream_unzip::ZipReader;

use bsp_explorer::{run, state_mesh::StateMesh};

pub fn main() {
    pollster::block_on(run(|state| {
        let instance = state.renderer().instance();

        let (header,mut buffer) = BSPHeader::load(
			"D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\maps\\d1_trainstation_01.bsp").unwrap();

        header.validate();

        //let mut mesh = StateMesh::new(state.renderer(), wgpu::PrimitiveTopology::TriangleList);
        //mesh.load_glb_mesh(instance.clone());
        //state.add_mesh(mesh);

        //let mut mesh = StateMesh::new(state.renderer(), wgpu::PrimitiveTopology::LineList);
        //mesh.load_debug_edges(instance.clone(), &header, &mut buffer);
        //state.add_mesh(mesh);

        let faces = header.get_lump::<BSPFace>(&mut buffer);
        let surfedges = header.get_lump::<BSPSurfEdge>(&mut buffer);
        let edges = header.get_lump::<BSPEdge>(&mut buffer);
        let verts = header.get_lump::<Vec3>(&mut buffer);
        let tex_info = header.get_lump::<BSPTexInfo>(&mut buffer);
        let tex_data = header.get_lump::<BSPTexData>(&mut buffer);

        let mut annotated_verts = bytemuck::zeroed_slice_box::<UVVertex>(verts.len());

        for i in 0..verts.len() {
            annotated_verts[i].position = verts[i];
        }

        //let mut tris = Vec::<u16>::new();
        // for now, filter by texture of first face

        let mut textured_tris = HashMap::<i32, Vec<u16>>::new();

        for face in faces.iter() {
            let root_edge_index = face.first_edge as usize;
            let root_edge = surfedges[root_edge_index].get_edge(&edges);

            let tex = tex_info[face.tex_info as usize];
            let texdata = tex.tex_data;
            let data = tex_data[texdata as usize];

            let s = tex.tex_s / data.width as f32;
            let t = tex.tex_t / data.height as f32;

            for i in 0..(face.num_edges as usize) {
                let edge = surfedges[root_edge_index + i].get_edge(&edges);

                // add uv info to these vertexes
                // The 2D coordinates (u, v) of a texture pixel (or texel) are mapped to the world coordinates (x, y, z) of a point on a face by:
                //
                // u = tv0,0 * x + tv0,1 * y + tv0,2 * z + tv0,3
                //
                // v = tv1,0 * x + tv1,1 * y + tv1,2 * z + tv1,3
                //
                for ee in [edge.0, edge.1] {
                    let e = ee as usize;
                    let u = s.dot(Vec4::from((annotated_verts[e].position, 1.0)));
                    let v = t.dot(Vec4::from((annotated_verts[e].position, 1.0)));

                    annotated_verts[e].uv = vec2(u, v);
                }
            }

            for i in 1..(face.num_edges as usize) {
                let edge = surfedges[root_edge_index + i].get_edge(&edges);

                let tris = match textured_tris.get_mut(&texdata) {
                    Some(x) => x,
                    None => {
                        textured_tris.insert(texdata, Default::default());
                        textured_tris.get_mut(&texdata).unwrap()
                    }
                };

                tris.extend([edge.0, root_edge.0, edge.1]);
            }
        }

        let pak_header = header.get_lump_header(LumpType::PAKFILE);

        let pak: BSPPak = pak_header.read_binary(&mut buffer).unwrap();

        let map: HashMap<&str, &str> = pak
            .entries
            .par_iter()
            .filter_map(|entry| {
                if entry.filename.contains(".vmt") {
                    let data = std::str::from_utf8(&entry.bytes[..]).unwrap();
                    if let Some(include) = data.find("\"include\"") {
                        // Get this value
                        let data = &data[include + 9..];

                        if let Some(open) = data.find('"') {
                            let data = &data[open + 1..];

                            if let Some(close) = data.find('"') {
                                //map.insert();

                                //println!("{}", data);
                                //println!("{} {}", entry.header().filename, &data[..close])
                                Some((entry.filename.as_str(), &data[..close]))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        let tex_data_string_table = header.get_lump::<BSPTexDataStringTable>(&mut buffer);
        let tex_data_string_data = header.get_lump_header(LumpType::TEXDATA_STRING_DATA);

        let tex_name_map: HashMap<i32, String> = textured_tris
            .iter()
            .map(|(tex, tris)| {
                (
                    *tex,
                    tex_data_string_table[tex_data[*tex as usize].name_string_table_id as usize]
                        .get_filename(&mut buffer, tex_data_string_data),
                )
            })
            .collect();

        let textures: HashMap<i32, String> = textured_tris
            .par_iter()
            .map(|(tex, tris)| {
                // turn surfaces into meshes
                let filename_mat = format!("materials/{}.vmt", tex_name_map[tex]);

                if let Some(mapped_file) = map.get(filename_mat.as_str()) {
                    //println!("Mapped {} to {}", filename_mat, mapped_file);
                    let mut s = (*mapped_file).to_owned();
                    s.make_ascii_lowercase();
                    (*tex, s.replace(".vmt", ".vtf"))
                } else {
                    (*tex, format!("materials/{}.vtf", tex_name_map[tex]))
                }
            })
            .collect();

        let r = state.renderer();

        //preload all textures in parallel
        textures.par_iter().for_each(|(tex, name)| {
            if let Ok(Some(tex)) = r.texture_dir().load_vtf(&textures[tex]) {
                tex.get_high_res(r.device(), r.queue());
            }
        });

        for (tex, tris) in &textured_tris {
            if let Ok(Some(tex)) = r.texture_dir().load_vtf(&textures[tex]) {
                if !tex.ready_for_use() {
                    return;
                }

                let mut mesh = StateMesh::new(r, wgpu::PrimitiveTopology::TriangleList);

                mesh.from_verts_and_tris(
                    instance.clone(),
                    bytemuck::cast_slice(&annotated_verts),
                    bytemuck::cast_slice(&tris),
                    tris.len() as u32,
                );

                mesh.load_tex(instance.clone(), &tex.get_high_res(r.device(), r.queue()));
                state.add_mesh(mesh);
            } else {
                println!("Could not find texture for {}", textures[tex])
            }
        }
    }));
}
