// This article's documentation is for anything that uses the Source engine. Click here for more information.
// VTX

// VTX is the extension for

// Source's proprietary mesh strip format. It stores hardware optimized material, skinning and triangle strip/fan information for each LOD of each mesh in the MDL.

// VTX files use a two-part file extension the first part varies depending upon the renderer, although StudioMDL usually makes identical VTX files for dx80, dx90, and sw.
// Extension 	Renderer
// .dx80.vtx 	DirectX 8 and lower
// .dx90.vtx 	DirectX 9 and higher
// .sw.vtx 	Software rendering
// .xbox.vtx 	Xbox

// The
// Left 4 Dead series originally used just .VTX files, as of the Sacrifice update for these two games they now use the .dx90.vtx for the Mac OSX and re-used the .VTX only for

// Portal 2 [Clarify].
// Confirm:

//     What is the software mesh file for? Source doesn't have a software renderer is it used by VRAD? It can safely be removed when running the game proper.
//     Is the Xbox mesh file used for the

// Original Xbox,
// Xbox 360, or both?
// What mesh file do other platforms, like
// PlayStation 3 and

//     Android use?

// Contents

//     1 File Structure
//         1.1 Header
//             1.1.1 Body array
//         1.2 BodyPartHeader
//             1.2.1 Model array
//         1.3 ModelHeader
//             1.3.1 LOD Mesh Array
//         1.4 ModelLODHeader
//             1.4.1 Mesh array
//             1.4.2 Switch Point
//         1.5 MeshHeader
//             1.5.1 Strip Group Array
//             1.5.2 Flags
//         1.6 StripGroupHeader
//             1.6.1 Vertex & Indices arrays
//             1.6.2 Strip Array
//         1.7 StripHeader
//             1.7.1 Indices & Vertex Groupings
//         1.8 Vertex
//     2 See also

// File Structure
// Header

use std::mem;

use crate::binaries::{BinArray, BinaryData};

pub struct VTX {
    pub header: VTXFileHeader,
    pub body: Vec<VTXBodyPart>,
}

pub struct VTXBodyPart(pub Vec<VTXModel>);

pub struct VTXModel(pub Vec<VTXModelLOD>);

pub struct VTXModelLOD(pub Vec<VTXMesh>);

pub struct VTXMesh {
    pub flags: u8,
    pub strip_groups: Vec<VTXStripGroup>,
}

impl BinaryData for VTX {
    fn read<R: std::io::Read + std::io::Seek>(
        buffer: &mut std::io::BufReader<R>,
        _max_size: Option<usize>,
    ) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let header = VTXFileHeader::read(buffer, None)?;

        let mut pos = mem::size_of::<VTXFileHeader>() as i64;

        let mut body = Vec::<VTXBodyPart>::new();
        let body_part_headers = header.body_parts.read(buffer, 0, &mut pos)?;

        //    println!("{:?}", b);
        // I'm honestly not sure if I need any of this.
        for (i, p) in body_part_headers {
            let model_headers = p.models.read(buffer, i, &mut pos)?;
            //    println!("{:?}", mds);

            let mut body_part = VTXBodyPart { 0: Vec::default() };

            for (i, model_header) in model_headers {
                let model_lod_headers = model_header.lods.read(buffer, i, &mut pos)?;
                //    println!("{:?}", l);

                let mut model = VTXModel { 0: Vec::default() };

                for (i, model_lod_header) in model_lod_headers {
                    //println!("{:?}", me);

                    let mut model_lod = VTXModelLOD { 0: Vec::default() };

                    let mesh_headers = model_lod_header.meshes.read(buffer, i, &mut pos)?;

                    for (i, mesh_header) in mesh_headers {
                        let strip_group_headers =
                            mesh_header.strip_groups.read(buffer, i, &mut pos)?;
                        //println!("{:?}", sgs);

                        let mut mesh = VTXMesh {
                            flags: mesh_header.flags,
                            strip_groups: Vec::default(),
                        };

                        for (i, strip_group_header) in strip_group_headers {
                            let indices = strip_group_header.indices.read_f(buffer, i, &mut pos)?;
                            let verts = strip_group_header.verts.read_f(buffer, i, &mut pos)?;

                            //let indices = bytemuck::zeroed_slice_box(1);
                            //println!("{:?}", indices);
                            let shs = strip_group_header.strip_groups.read(buffer, i, &mut pos)?;
                            let mut strip_group = VTXStripGroup {
                                indices,
                                verts,
                                head: strip_group_header,
                                strips: Default::default(),
                            };

                            for (_i, sh) in shs {
                                strip_group.strips.push(VTXStrip { header: sh });
                            }

                            //println!("{:?}", shs);

                            mesh.strip_groups.push(strip_group);
                        }
                        model_lod.0.push(mesh);
                    }
                    model.0.push(model_lod);
                }
                body_part.0.push(model);
            }
            body.push(body_part);
        }

        Ok(Self { header, body })
    }
}

pub struct VTXStripGroup {
    pub head: StripGroupHeader,
    pub strips: Vec<VTXStrip>,
    pub indices: Box<[u16]>,
    pub verts: Box<[VTXVertex]>,
}

#[derive(Debug)]
pub struct VTXStrip {
    pub header: StripHeader,
}

// this structure is in <mod folder>/src/public/optimize.h
#[repr(C, packed)]
#[derive(bytemuck::Zeroable)]
pub struct VTXFileHeader {
    // file version as defined by OPTIMIZED_MODEL_FILE_VERSION (currently 7)
    pub version: i32,

    // hardware params that affect how the model is to be optimized.
    pub vert_cache_size: i32,
    pub max_bones_per_strip: u16,
    pub max_bones_per_tri: u16,
    pub max_bones_per_vert: i32,

    // must match checkSum in the .mdl
    pub check_sum: i32,

    pub num_lods: i32, // Also specified in ModelHeader's and should match

    // Offset to materialReplacementList Array. one of these for each LOD, 8 in total
    pub material_replacement_list_offset: i32,

    //Defines the size and location of the body part array
    body_parts: BinArray<BodyPartHeader>,
}

// This is the header structure for the current VERSION7 .vtx file
// Body array

// The body array is a list of BodyPartHeader objects.

//     Size: numBodyParts.
//     Location: bodyPartOffset.

// Note:
// Since this value is in the header it can be interpreted as an absolute location
// BodyPartHeader
#[repr(C, packed)]
#[derive(bytemuck::Zeroable)]
struct BodyPartHeader {
    //Model array
    models: BinArray<ModelHeader>,
}

// Model array

// The model array is a list of ModelHeader objects.

//     Size: numModels.
//     Location: bodyPartOffset.

// ModelHeader

// // This maps one to one with models in the mdl file.
#[repr(C, packed)]
#[derive(bytemuck::Zeroable)]
struct ModelHeader {
    //LOD mesh array
    lods: BinArray<ModelLODHeader>,
}

// LOD Mesh Array

// The LOD mesh array is a list of ModelLODHeader objects.

//     Size: num_lods.
//     Location: lodOffset.

// ModelLODHeader
#[repr(C, packed)]
#[derive(bytemuck::Zeroable)]
struct ModelLODHeader {
    //Mesh array
    meshes: BinArray<VTXMeshHeader>,
    switch_point: f32,
}

// Mesh array

// The mesh array is a list of MeshHeader objects.

//     Size: numMeshes.
//     Location: meshOffset.

// Switch Point

// The point at which the engine should switch to this LOD mesh is defined by switchPoint.
// MeshHeader

#[repr(C, packed)]
#[derive(bytemuck::Zeroable)]
struct VTXMeshHeader {
    strip_groups: BinArray<StripGroupHeader>,
    pub flags: u8,
}
// Strip Group Array

// The strip group array is a list of StripGroupHeader objects.

//     Size: numStripGroups.
//     Location: stripGroupHeaderOffset.

// Flags

// The u8 flags value can be read from this table:
// Value 	Meaning
// 0x01 	STRIPGROUP_IS_FLEXED
// 0x02 	STRIPGROUP_IS_HWSKINNED
// 0x04 	STRIPGROUP_IS_DELTA_FLEXED
// 0x08 	STRIPGROUP_SUPPRESS_HW_MORPH
// StripGroupHeader

#[repr(C, packed)]
#[derive(bytemuck::Zeroable)]
pub struct StripGroupHeader {
    // These are the arrays of all verts and indices for this mesh.  strips index into this.
    verts: BinArray<VTXVertex>,

    indices: BinArray<u16>,

    strip_groups: BinArray<StripHeader>,

    pub flags: OptimizeStripFlags,

    // The following fields are only present if MDL version is >=49
    // Points to an array of unsigned shorts (16 bits each)
    pub numopology_indices: i32,
    pub topology_offset: i32,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum OptimizeStripFlags {
    None = 0,
    IsTriList = 0x01,
    IsTriStrip = 0x02,
}
unsafe impl bytemuck::Zeroable for OptimizeStripFlags {}
// MDL versions 49 and above (found in

// Counter-Strike: Global Offensive) have an extra two i32 fields (totaling 8 bytes). This is not reflected in the VTX header version, which remains at 7.
// Todo:
// What do these indices do?
// Vertex & Indices arrays

// Indices Array - This is a set of u16 integers that index the position of the real vertex data in the .VVD's vertex array

//     Size: numIndices
//     Location: indexOffset

// Vertex Array - The vertex array inside the .VTX file holds some extra information related to skinning

//     Size: numVerts
//     Location: vertOffset

// Strip Array

// The strip array is a list of StripHeader objects

//     Size: numStrips
//     Location: stripOffset

// StripHeader

// // A strip is a piece of a stripgroup which is divided by bones
#[repr(C, packed)]
#[derive(Debug, bytemuck::Zeroable)]
pub struct StripHeader {
    pub num_indices: i32,
    pub index_offset: i32,

    pub num_verts: i32,
    pub vert_offset: i32,

    pub num_bones: i8,

    pub flags: u8,

    pub num_bone_state_changes: i32,
    pub bone_state_change_offset: i32,

    // MDL Version 49 and up only
    pub numopology_indices: i32,
    pub topology_offset: i32,
}

// Like in StripGroupHeader, the last eight bytes/two i32 fields are present if the MDL file's header shows version 49 or higher. Presumably, (
// Todo:
// Verify

// ) these index into the parent strip group's list of topology indices, similar to vertices and indices.
// Indices & Vertex Groupings

// Each group (Indices and Vertices respectively), specify what position to read from the vertex pool, as well as the indices pool. These pools come from the parent StripGroupHeader object
// Vertex
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct VTXVertex {
    // these index into the mesh's vert[origMeshVertID]'s bones
    pub bone_weight_index: [u8; 3],
    pub num_bones: u8,

    pub orig_mesh_vert_id: u16,

    // for sw skinned verts, these are indices into the global list of bones
    // for hw skinned verts, these are hardware bone indices
    pub bone_id: [i8; 3],
}

// origMeshVertID defines the index of this vertex that is to be read from the linked .VVD file's vertex array
// Note:
// This value needs to be added to the total vertices read, since it is relative to the mesh and won't work as an absolute key.

// When parsing, note that a Vertex contains nine bytes of information, but will usually be padded to 10 bytes. Within the VTX file, the length of each vertex's data will still be 9. For example, one should call in C:

// fread(vertexBuf, vertexCount, 9, fileptr)

// Instead of:

// fread(vertexBuf, vertexCount, sizeof(Vertex), fileptr)

// The latter sample will cause data corruption, unless your model has only one vertex (unlikely).
