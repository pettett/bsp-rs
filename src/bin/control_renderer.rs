use bsp_explorer::{
    bsp::{
        consts::LumpType,
        edges::{dedge_t, dsurfedge_t},
        face::dface_t,
        header::dheader_t,
        textures::{texdata_t, texdatastringtable_t, texinfo_t},
    },
    state::State,
    vertex::UVVertex,
};
use glam::{vec2, Vec3, Vec4};
use std::{collections::HashMap, thread};
use stream_unzip::ZipReader;

use bsp_explorer::{run, state_mesh::StateMesh};

pub fn main() {
    pollster::block_on(run(|state| {
        let instance = state.renderer().instance();

        let (header,mut buffer) = dheader_t::load(
			"D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\maps\\d1_trainstation_02.bsp").unwrap();

        header.validate();

        //let mut mesh = StateMesh::new(state.renderer(), wgpu::PrimitiveTopology::TriangleList);
        //mesh.load_glb_mesh(instance.clone());
        //state.add_mesh(mesh);

        //let mut mesh = StateMesh::new(state.renderer(), wgpu::PrimitiveTopology::LineList);
        //mesh.load_debug_edges(instance.clone(), &header, &mut buffer);
        //state.add_mesh(mesh);

        let faces = header.get_lump::<dface_t>(&mut buffer);
        let surfedges = header.get_lump::<dsurfedge_t>(&mut buffer);
        let edges = header.get_lump::<dedge_t>(&mut buffer);
        let verts = header.get_lump::<Vec3>(&mut buffer);
        let tex_info = header.get_lump::<texinfo_t>(&mut buffer);
        let tex_data = header.get_lump::<texdata_t>(&mut buffer);

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

        let pakfile = header.get_lump_header(LumpType::PAKFILE);

        let pakfile_data = pakfile.get_bytes(&mut buffer).unwrap();

        let mut zip_reader = ZipReader::default();

        zip_reader.update(pakfile_data.into());

        // Or read the whole file and deal with the entries
        // at the end.
        zip_reader.finish();
        let mut map = HashMap::<&str, &str>::new();
        let entries = zip_reader.drain_entries();
        for entry in &entries {
            // write to disk or whatever you need.
            if entry.header().filename.contains(".vmt") {
                let data = std::str::from_utf8(entry.compressed_data()).unwrap();
                if let Some(include) = data.find("\"include\"") {
                    // Get this value
                    let data = &data[include + 9..];

                    if let Some(open) = data.find('"') {
                        let data = &data[open + 1..];

                        if let Some(close) = data.find('"') {
                            map.insert(&entry.header().filename, &data[..close]);

                            //println!("{}", data);
                            println!("{} {}", entry.header().filename, &data[..close]);
                        }
                    }
                }
            }
        }

        let tex_data_string_table = header.get_lump::<texdatastringtable_t>(&mut buffer);
        let tex_data_string_data = header.get_lump_header(LumpType::TEXDATA_STRING_DATA);

        for (tex, tris) in &textured_tris {
            // turn surfaces into meshes
            let filename_mat = format!(
                "materials/{}.vmt",
                tex_data_string_table[tex_data[*tex as usize].nameStringTableID as usize]
                    .get_filename(&mut buffer, tex_data_string_data)
            );

            let filename_tex = if let Some(mapped_file) = map.get(filename_mat.as_str()) {
                println!("Mapped {} to {}", filename_mat, mapped_file);
                let mut s = (*mapped_file).to_owned();
                s.make_ascii_lowercase();
                s.replace(".vmt", ".vtf")
            } else {
                format!(
                    "materials/{}.vtf",
                    tex_data_string_table[tex_data[*tex as usize].nameStringTableID as usize]
                        .get_filename(&mut buffer, tex_data_string_data)
                )
            };

            if let Ok(tex) = state.renderer().texture_dir().load_vtf(&filename_tex) {
                if !tex.ready_for_use() {
                    continue;
                }

                let mut mesh =
                    StateMesh::new(state.renderer(), wgpu::PrimitiveTopology::TriangleList);

                mesh.from_verts_and_tris(
                    instance.clone(),
                    bytemuck::cast_slice(&annotated_verts),
                    bytemuck::cast_slice(&tris),
                    tris.len() as u32,
                );

                mesh.load_tex(
                    instance.clone(),
                    &tex.get_high_res(state.renderer().device(), state.renderer().queue()),
                );
                state.add_mesh(mesh);
            } else {
                println!("Could not find texture for {}", filename_tex)
            }
        }
    }));
}
