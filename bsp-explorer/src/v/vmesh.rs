 
use std::{collections::HashMap, fs::File, io::BufReader, sync::Arc};

use bevy_ecs::component::Component;
use common::{vinstance::StateInstance, vpath::{VPath, VSplitPath, VLocalPath}, vshader::VShader, vertex::{UVVertex, Vertex, UVAlphaVertex}, vtexture::VTexture};
use glam::{vec3, Vec3};

use source::{prelude::{LightingData, BSPEdge, BSPHeader}, game_data::GameData, vpk::VPKFile, studio::vvd::Fixup};
use wgpu::util::DeviceExt;

use super::{vrenderer::VRenderer};
#[derive(Component)]
pub struct VMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_format: wgpu::IndexFormat,
    texture_bind_group: HashMap<u32, wgpu::BindGroup>,
    shader: Arc<VShader>,
    num_indices: u32,
    //puffin_ui : puffin_imgui::ProfilerUi,
}

impl VMesh {
    pub fn draw<'a>(
        &'a self,
        state: &'a VRenderer,
        render_pass: &mut wgpu::RenderPass<'a>,
        lighting: &'a LightingData,
    ) {
        // 1.

        self.shader.draw( render_pass);

        render_pass.set_bind_group(0, state.camera_bind_group(), &[]);
        render_pass.set_bind_group(1, &lighting.buffer.bind_group, &[]);

        self.draw_inner(render_pass, 1, None);
    }

    pub fn draw_instanced<'a>(
        &'a self,
        state: &'a VRenderer,
        render_pass: &mut wgpu::RenderPass<'a>,
        instance_count: u32,
        instance_buffer: &'a wgpu::Buffer,
    ) {
        // 1.

        self.shader.draw( render_pass);

        render_pass.set_bind_group(0, state.camera_bind_group(), &[]);

        self.draw_inner(render_pass, instance_count, Some(instance_buffer));
    }

    fn draw_inner<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        instance_count: u32,
        instance_buffer: Option<&'a wgpu::Buffer>,
    ) {
        // 1.

        for (i, tex) in &self.texture_bind_group {
            render_pass.set_bind_group(*i + self.shader.tex_bind_start(), tex, &[]);
        }

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        if let Some(instance_buffer) = instance_buffer {
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        }
        render_pass.set_index_buffer(self.index_buffer.slice(..), self.index_format);

        render_pass.draw_indexed(0..self.num_indices, 0, 0..instance_count);
    }

    pub fn new_box(device: &wgpu::Device, min: Vec3, max: Vec3, shader: Arc<VShader>) -> Self {
        Self::new(
            device,
            &[
                vec3(min.x, min.y, min.z),
                vec3(max.x, min.y, min.z),
                vec3(min.x, max.y, min.z),
                vec3(max.x, max.y, min.z),
                vec3(min.x, min.y, max.z),
                vec3(max.x, min.y, max.z),
                vec3(min.x, max.y, max.z),
                vec3(max.x, max.y, max.z),
            ],
            &[
                0u16, 1, 1, 3, 3, 2, 2, 0, 4, 5, 5, 7, 7, 6, 6, 4, 0, 4, 1, 5, 2, 6, 3, 7,
            ],
            shader,
        )
    }
    pub fn new_empty(device: &wgpu::Device, shader: Arc<VShader>) -> Self {
        Self::new::<UVVertex>(device, &[], &[], shader)
    }
    pub fn new<V: Vertex + bytemuck::Pod>(
        device: &wgpu::Device,
        verts_data: &[V],
        indices_data: &[u16],
        shader: Arc<VShader>,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(verts_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        VMesh {
            vertex_buffer,
            index_buffer,
            num_indices: indices_data.len() as u32,
            texture_bind_group: Default::default(),
            shader,
            index_format: wgpu::IndexFormat::Uint16,
        }
    }

    pub fn load_debug_edges(
        &mut self,
        device: &wgpu::Device,
        header: &BSPHeader,
        buffer: &mut BufReader<File>,
    ) {
        let edges = header.get_lump::<BSPEdge>(buffer);
        let verts = header.get_lump::<Vec3>(buffer);

        let mut annotated_verts = bytemuck::zeroed_slice_box::<UVVertex>(verts.len());

        for i in 0..verts.len() {
            annotated_verts[i].position = verts[i];
        }

        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&annotated_verts[..]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&edges[..]),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Update the value stored in this mesh

        self.num_indices = edges.len() as u32;
        self.index_format = wgpu::IndexFormat::Uint16;
    }

    pub fn from_verts_and_tris(
        &mut self,
        device: &wgpu::Device,
        verts: &[u8],
        tris: &[u8],
        num_indicies: u32,
    ) {
        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: verts,
            usage: wgpu::BufferUsages::VERTEX,
        });

        self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: tris,
            usage: wgpu::BufferUsages::INDEX,
        });

        // Update the value stored in this mesh

        self.num_indices = num_indicies;
        self.index_format = wgpu::IndexFormat::Uint16;
    }

    pub fn load_tex(&mut self, device: &wgpu::Device, bind_index: u32, texture: &VTexture) {
        self.texture_bind_group.insert(
            bind_index,
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.shader.texture_bind_group_layout(bind_index).unwrap(),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(texture.view()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(texture.sampler()),
                    },
                ],
                label: Some("diffuse_bind_group"),
            }),
        );
    }
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

pub fn load_vmesh(
    mdl_path: &dyn VPath,
    instance: &StateInstance,
    shader_tex: Arc<VShader>,
    game_data: &GameData,
) -> Result<VMesh, &'static str> {
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

            for s in &strip_group.strips {
                let ind_start = s.header.index_offset as usize;
                let ind_count = s.header.num_indices as usize;

                let mut m = VMesh::new(
                    &instance.device,
                    &verts[..],
                    &indices[ind_start..ind_start + ind_count],
                    shader_tex,
                );

                let mat_path = VSplitPath::new(&mat_dir, &mdl.textures[0].name, "vmt");

                let vmt = game_data
                    .load(&mat_path, VPKFile::vmt)
                    .ok_or("No VMT material file")?;

                let tex_path = {
                    let Some(tex_path) = vmt.get_basetex() else {
                        return Err("Could not find texture param in vmt");
                    };

                    tex_path.replace('\\', "/")
                };

                let vtf_path = VLocalPath::new("materials", &tex_path, "vtf");

                let vtf = game_data
                    .load(&vtf_path, VPKFile::vtf)
                    .ok_or("No VTF texture file")?;

                let vtex = vtf
                    .get_high_res(instance)
                    .as_ref()
                    .map_err(|()| "Failed to load high res")?;

                m.load_tex(&instance.device, 0, vtex);

                return Ok(m);
            }
        }
    }
    Err("No mesh in LODs")
}

#[cfg(test)]
mod mdl_tests { 
	use common::vpath::VGlobalPath;
use source::vpk::VPKDirectory;
 
    use std::path::PathBuf;

    //const PATH: &str = "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Portal 2\\portal2\\maps\\sp_a2_laser_intro.bsp";

    #[test]
    fn test_misc_dir() {
        let dir = VPKDirectory::load(Default::default(),PathBuf::from(
            "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_misc_dir.vpk",
            
        ))
        .unwrap();
        for (_d, files) in &dir.files["mdl"] {
            for (_file, data) in files {
                let Ok(mdl) = data.load_mdl(&dir) else {
                    continue;
                };
                assert!(mdl.version < 100);
            }
        }
    }

    #[test]
    fn test_single_vtx_misc_dir() {
        let dir = VPKDirectory::load(Default::default(),PathBuf::from(
            "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_misc_dir.vpk",
        ))
        .unwrap();

        let _vtx = dir
            .load_vtx(&VGlobalPath::from("models/props_c17/bench01a.dx90.vtx"))
            .unwrap();
    }

    #[test]
    fn test_single_vvd_misc_dir() {
        let dir = VPKDirectory::load(Default::default(),PathBuf::from(
            "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_misc_dir.vpk",
        ))
        .unwrap();

        let vvd = dir
            .load_vvd(&VGlobalPath::from("models/props_c17/bench01a.vvd"))
            .unwrap();

        for t in vvd.tangents.iter() {
            assert!(t.w == 0.0 || t.w == -1.0 || t.w == 1.0)
        }
        println!("Tangents all good!");
    }

    #[test]
    fn test_single_misc_dir() {
        let dir = VPKDirectory::load(Default::default(),PathBuf::from(
            "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_misc_dir.vpk",
        ))
        .unwrap();

        let vtx = dir
            .load_vtx(&VGlobalPath::from("models/props_c17/bench01a.dx90.vtx"))
            .unwrap();

        let mdl = dir
            .load_mdl(&VGlobalPath::from("models/props_c17/bench01a.mdl"))
            .unwrap();

        let _vvd = dir
            .load_vvd(&VGlobalPath::from("models/props_c17/bench01a.vvd"))
            .unwrap();

        //print!("{:?}", mdl.text);

        assert_eq!(mdl.body.len(), vtx.body[0].0.len());

        let vtx_lod0 = &vtx.body[0].0[0].0;

        for m in vtx_lod0 {
            for _sg in &m.0 {
                //5!("{:?}", sg.indices);
            }
        }
    }

    // #[test]
    // fn test_misc_dir_p2() {
    //     let dir = VPKDirectory::load(PathBuf::from(
    //         "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Portal 2\\portal2\\pak01_dir.vpk",
    //     ))
    //     .unwrap();

    //     let mut shaders = HashSet::new();

    //     for (d, files) in &dir.files["vmt"] {
    //         for (_file, data) in files {
    //             let Ok(Some(vmt)) = data.load_vmt(&dir) else {
    //                 continue;
    //             };
    //             shaders.insert(vmt.shader.to_ascii_lowercase());
    //         }
    //     }
    //     println!("{:?}", shaders);
    // }

    // #[test]
    // fn test_maps() {
    //     let (header, mut buffer) = BSPHeader::load(Path::new(PATH)).unwrap();

    //     let pak_header = header.get_lump_header(LumpType::PakFile);

    //     let dir: VPKDirectory = pak_header.read_binary(&mut buffer).unwrap();

    //     for (d, files) in &dir.files["vmt"] {
    //         println!("DIR: {}", d);
    //         for (_file, data) in files {
    //             let Ok(Some(vmt)) = data.load_vmt(&dir) else {
    //                 continue;
    //             };
    //             println!("{:?}", vmt.data);
    //         }
    //     }
    // }
}
