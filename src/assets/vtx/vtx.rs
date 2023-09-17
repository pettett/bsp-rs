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

// this structure is in <mod folder>/src/public/optimize.h
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
struct FileHeader {
    // file version as defined by OPTIMIZED_MODEL_FILE_VERSION (currently 7)
    version: i32,

    // hardware params that affect how the model is to be optimized.
    vert_cache_size: i32,
    max_bones_per_strip: u16,
    max_bones_per_tri: u16,
    max_bones_per_vert: i32,

    // must match checkSum in the .mdl
    check_sum: i32,

    num_lods: i32, // Also specified in ModelHeader's and should match

    // Offset to materialReplacementList Array. one of these for each LOD, 8 in total
    material_replacement_list_offset: i32,

    //Defines the size and location of the body part array
    num_body_parts: i32,
    body_part_offset: i32,
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
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
struct BodyPartHeader {
    //Model array
    num_models: i32,
    model_offset: i32,
}

// Model array

// The model array is a list of ModelHeader objects.

//     Size: numModels.
//     Location: bodyPartOffset.

// ModelHeader

// // This maps one to one with models in the mdl file.
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
struct ModelHeader {
    //LOD mesh array
    num_lods: i32, //This is also specified in FileHeader
    lod_offset: i32,
}

// LOD Mesh Array

// The LOD mesh array is a list of ModelLODHeader objects.

//     Size: num_lods.
//     Location: lodOffset.

// ModelLODHeader
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
struct ModelLODHeader {
    //Mesh array
    num_meshes: i32,
    mesh_offset: i32,
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
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
struct MeshHeader {
    num_strip_groups: i32,
    strip_group_header_offset: i32,
    flags: u8,
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
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
struct StripGroupHeader {
    // These are the arrays of all verts and indices for this mesh.  strips index into this.
    num_verts: i32,
    vert_offset: i32,

    num_indices: i32,
    index_offset: i32,

    num_strips: i32,
    strip_offset: i32,

    flags: u8,

    // The following fields are only present if MDL version is >=49
    // Points to an array of unsigned shorts (16 bits each)
    numopology_indices: i32,
    topology_offset: i32,
}

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
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
struct StripHeader {
    num_indices: i32,
    index_offset: i32,

    num_verts: i32,
    vert_offset: i32,

    num_bones: i8,

    flags: u8,

    num_bone_state_changes: i32,
    bone_state_change_offset: i32,

    // MDL Version 49 and up only
    numopology_indices: i32,
    topology_offset: i32,
}

// Like in StripGroupHeader, the last eight bytes/two i32 fields are present if the MDL file's header shows version 49 or higher. Presumably, (
// Todo:
// Verify

// ) these index into the parent strip group's list of topology indices, similar to vertices and indices.
// Indices & Vertex Groupings

// Each group (Indices and Vertices respectively), specify what position to read from the vertex pool, as well as the indices pool. These pools come from the parent StripGroupHeader object
// Vertex
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
struct Vertex {
    // these index into the mesh's vert[origMeshVertID]'s bones
    bone_weight_index: [u8; 3],
    num_bones: u8,

    orig_mesh_vert_id: u16,

    // for sw skinned verts, these are indices into the global list of bones
    // for hw skinned verts, these are hardware bone indices
    bone_id: [i8; 3],
}

// origMeshVertID defines the index of this vertex that is to be read from the linked .VVD file's vertex array
// Note:
// This value needs to be added to the total vertices read, since it is relative to the mesh and won't work as an absolute key.

// When parsing, note that a Vertex contains nine bytes of information, but will usually be padded to 10 bytes. Within the VTX file, the length of each vertex's data will still be 9. For example, one should call in C:

// fread(vertexBuf, vertexCount, 9, fileptr)

// Instead of:

// fread(vertexBuf, vertexCount, sizeof(Vertex), fileptr)

// The latter sample will cause data corruption, unless your model has only one vertex (unlikely).
