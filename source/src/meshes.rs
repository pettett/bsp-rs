use std::collections::HashMap;

use common::vertex::{UVAlphaVertex, UVVertex, Vertex};
use glam::{ivec3, vec2, IVec3, Vec3, Vec4};

use crate::prelude::*;


#[derive(Default)]
pub struct MeshBuilder<V: Vertex + Default> {
    tris: Vec<u16>,
    //tri_map: HashMap<u16, u16>,
    verts: Vec<V>,
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
	pub fn tris(&self) -> &[u16] {
		&self.tris
	}
	
	pub fn verts(&self) -> &[V] {
			&self.verts
		}
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

pub fn build_meshes(
    faces: Box<[BSPFace]>,
    verts: &[Vec3],
    disp_verts: &[BSPDispVert],
    tex_info: &[BSPTexInfo],
    tex_data: &[BSPTexData],
    infos: &[BSPDispInfo],
    edges: &[BSPEdge],
    surf_edges: &[BSPSurfEdge],
) -> HashMap<i32, MeshBuilder<UVVertex>> {
    let mut textured_tris = HashMap::<i32, MeshBuilder<UVVertex>>::new();

    for (i_face, face) in faces.iter().enumerate() {
        let tex = tex_info[face.tex_info as usize];
        let i_texdata = tex.tex_data;
        let data = tex_data[i_texdata as usize];

		// get the mesh for this
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

