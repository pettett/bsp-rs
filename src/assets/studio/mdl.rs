use std::mem;

use crate::{
    assets::studio::mdl_headers::{StudioMesh, StudioModel},
    binaries::BinaryData,
};

use super::mdl_headers::{self, StudioBodyparts, Texture};

pub struct MDL {
    //pub header: mdl_headers::MDLHeader,
    pub version: u32,
    pub body: Vec<MDLBodyPart>,
    pub textures: Vec<MDLTexture>,
}

pub struct MDLBodyPart {
    pub name: String,
    pub head: StudioBodyparts,
    pub models: Vec<MDLModel>,
}
#[derive(Debug)]
pub struct MDLTexture {
    pub name: String,
}

pub struct MDLModel {
    pub head: StudioModel,
    pub meshes: Vec<MDLMesh>,
}

pub struct MDLMesh {
    pub head: StudioMesh,
}

impl BinaryData for MDL {
    fn read<R: std::io::Read + std::io::Seek>(
        buffer: &mut std::io::BufReader<R>,
        _max_size: Option<usize>,
    ) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let header = mdl_headers::MDLHeader::read(buffer, None)?;

        let mut pos = mem::size_of::<mdl_headers::MDLHeader>() as i64;

        let text = header.texture.read(buffer, 0, &mut pos)?;
        let mut textures = Vec::<MDLTexture>::new();

        for (i, t) in text {
            textures.push(MDLTexture {
                name: t
                    .name_offset
                    .read_str(buffer, i, &mut pos)?
                    .to_ascii_lowercase(),
            });
        }

        // for (i, t) in &text {
        //     println!("{:?}", t);
        //     println!("{}", t.name_offset.read_str(buffer, *i, &mut pos)?)
        // }

        let body = header.bodypart.read(buffer, 0, &mut pos)?;
        let mut parts = Vec::<MDLBodyPart>::new();
        for (i, t) in body {
            let mut models = Vec::<MDLModel>::new();

            //println!("{:?}", t);
            let model_heads: Vec<(i64, StudioModel)> =
                t.modelindex.read_array(buffer, i, &mut pos, t.nummodels)?;

            for (ii, model) in model_heads {
                //println!("{:?}", model);

                assert!(model.vertexindex % 0x30 == 0);
                assert!(model.tangentsindex % 0x10 == 0);

                let _v = model.vertexindex;

                let _first_vertex = (model.vertexindex / 0x30) | 0;
                let _first_tangent = (model.tangentsindex / 0x10) | 0;

                //println!("{first_vertex} {first_tangent}");

                let mesh_heads: Vec<(i64, StudioMesh)> = model.meshes.read(buffer, ii, &mut pos)?;
                let mut meshes = Vec::<MDLMesh>::new();

                for (_iii, mesh) in mesh_heads {
                    meshes.push(MDLMesh { head: mesh })
                }

                //println!("{:?}", meshes);
                models.push(MDLModel {
                    head: model,
                    meshes,
                });
            }

            let name = t.name_index.read_str(buffer, i, &mut pos)?;

            //println!("{}", name);
            parts.push(MDLBodyPart {
                name,
                head: t,
                models,
            });
        }

        Ok(Self {
            body: parts,
            version: header.version,
            textures,
        })
    }
}
