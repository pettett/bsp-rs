use std::mem;

use crate::{
    assets::studio::mdl_headers::{mstudiomesh_t, mstudiomodel_t},
    binaries::BinaryData,
};

use super::mdl_headers::{self, mstudiobodyparts_t, mstudiotexture_t};

#[derive(Debug)]
pub struct MDL {
    pub header: mdl_headers::MDLHeader,
    pub body: Vec<(i64, mstudiobodyparts_t)>,
    pub text: Vec<(i64, mstudiotexture_t)>,
}

impl BinaryData for MDL {
    fn read<R: std::io::Read + std::io::Seek>(
        buffer: &mut std::io::BufReader<R>,
        max_size: Option<usize>,
    ) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let header = mdl_headers::MDLHeader::read(buffer, None)?;

        let mut pos = mem::size_of::<mdl_headers::MDLHeader>() as i64;

        let text = header.texture.read(buffer, 0, &mut pos)?;

        for (i, t) in &text {
            println!("{:?}", t);
            println!("{}", t.name_offset.read_str(buffer, *i, &mut pos)?)
        }

        let body = header.bodypart.read(buffer, 0, &mut pos)?;

        for (i, t) in &body {
            println!("{:?}", t);
            let models: Vec<(i64, mstudiomodel_t)> =
                t.modelindex.read_array(buffer, *i, &mut pos, t.nummodels)?;

            for (ii, model) in &models {
                println!("{:?}", model);

                assert!(model.vertexindex % 0x30 == 0);
                assert!(model.tangentsindex % 0x10 == 0);
                let first_vertex = (model.vertexindex / 0x30) | 0;
                let first_tangent = (model.tangentsindex / 0x10) | 0;

                println!("{first_vertex} {first_tangent}");

                let meshes: Vec<(i64, mstudiomesh_t)> = model.meshes.read(buffer, *ii, &mut pos)?;

                println!("{:?}", meshes);
            }

            let name = t.name_index.read_str(buffer, *i, &mut pos)?;

            println!("{}", name);
        }

        Ok(Self { header, body, text })
    }
}
