```rs
// Source Engine BSP.

struct AABB{
	min: Vec3,
	max: Vec3,
}

enum LumpType {
    ENTITIES                  = 0,
    PLANES                    = 1,
    TEXDATA                   = 2,
    VERTEXES                  = 3,
    VISIBILITY                = 4,
    NODES                     = 5,
    TEXINFO                   = 6,
    FACES                     = 7,
    LIGHTING                  = 8,
    LEAFS                     = 10,
    EDGES                     = 12,
    SURFEDGES                 = 13,
    MODELS                    = 14,
    WORLDLIGHTS               = 15,
    LEAFFACES                 = 16,
    DISPINFO                  = 26,
    VERTNORMALS               = 30,
    VERTNORMALINDICES         = 31,
    DISP_VERTS                = 33,
    GAME_LUMP                 = 35,
    LEAFWATERDATA             = 36,
    PRIMITIVES                = 37,
    PRIMINDICES               = 39,
    PAKFILE                   = 40,
    CUBEMAPS                  = 42,
    TEXDATA_STRING_DATA       = 43,
    TEXDATA_STRING_TABLE      = 44,
    OVERLAYS                  = 45,
    LEAF_AMBIENT_INDEX_HDR    = 51,
    LEAF_AMBIENT_INDEX        = 52,
    LIGHTING_HDR              = 53,
    WORLDLIGHTS_HDR           = 54,
    LEAF_AMBIENT_LIGHTING_HDR = 55,
    LEAF_AMBIENT_LIGHTING     = 56,
    FACES_HDR                 = 58,
}

struct SurfaceLightmapData {
    faceIndex: i32,
    // Size of a single lightmap.
    width: i32,
    height: i32,
    styles: Vec<i32>,
    samples: Option<Vec<u8>>,
    hasBumpmapSamples: bool,
    // Dynamic allocation
    pageIndex: i32,
    pagePosX: i32,
    pagePosY: i32,
}

struct Overlay {
    surfaceIndexes: Vec<i32>;
}

struct Surface {
    texName: String,
    onNode: bool,
    startIndex: i32,
    indexCount: i32,
    center: Option<Vec3>,

    // Whether we want TexCoord0 to be divided by the texture size. Needed for most BSP surfaces
    // using Texinfo mapping, but *not* wanted for Overlay surfaces. This might get rearranged if
    // we move overlays out of being BSP surfaces...
    wantsTexCoord0Scale: bool,

    // Since our surfaces are merged together from multiple BSP surfaces, we can have multiple
    // surface lightmaps, but they're guaranteed to have been packed into the same lightmap page.
    lightmapData: Vec<SurfaceLightmapData>,
    lightmapPackerPageIndex: i32,

    bbox: AABB;
}

enum TexinfoFlags {
    SKY2D     = 0x0002,
    SKY       = 0x0004,
    TRANS     = 0x0010,
    NODRAW    = 0x0080,
    NOLIGHT   = 0x0400,
    BUMPLIGHT = 0x0800,
}

struct Texinfo {
    textureMapping: TexinfoMapping,
    lightmapMapping: TexinfoMapping,
    flags: TexinfoFlags,

    // texdata
    texName: String,
}

struct TexinfoMapping {
    // 2x4 matrix for texture coordinates
    s: ReadonlyVec4;
    t: ReadonlyVec4;
}

fn calcTexCoord(dst: Vec2, v: ReadonlyVec3, m: TexinfoMapping) -> () {
    dst[0] = v[0]*m.s[0] + v[1]*m.s[1] + v[2]*m.s[2] + m.s[3];
    dst[1] = v[0]*m.t[0] + v[1]*m.t[1] + v[2]*m.t[2] + m.t[3];
}

// Place into the lightmap page.
fn calcLightmapTexcoords(dst: Vec2, uv: ReadonlyVec2, lightmapData: SurfaceLightmapData, lightmapPage: LightmapPackerPage) -> () {
    dst[0] = (uv[0] + lightmapData.pagePosX) / lightmapPage.width;
    dst[1] = (uv[1] + lightmapData.pagePosY) / lightmapPage.height;
}

struct BSPNode {
    plane: Plane;
    child0: i32;
    child1: i32;
    bbox: AABB;
    area: i32;
}

use AmbientCube = Vec<Color>;

struct BSPLeafAmbientSample {
    ambientCube: AmbientCube;
    pos: Vec3;
}

let enum BSPLeafContents {
    Solid     = 0x001,
    Water     = 0x010,
    TestWater = 0x100,
}

struct BSPLeaf {
    bbox: AABB,
    area: i32,
    cluster: i32,
    ambientLightSamples: Vec<BSPLeafAmbientSample>,
    faces: Vec<i32>,
    surfaces: Vec<i32>,
    leafwaterdata: i32,
    contents: BSPLeafContents,
}

struct BSPLeafWaterData {
    surfaceZ: i32,
    minZ: i32,
    surfaceMaterialName: String,
}

struct Model {
    bbox: AABB,
    headnode: i32,
    surfaces: Vec<i32>,
}

enum WorldLightType {
    Surface,
    Point,
    Spotlight,
    SkyLight,
    QuakeLight,
    SkyAmbient,
}

enum WorldLightFlags {
    InAmbientCube = 0x01,
}

struct WorldLight {
    pos: Vec3,
    intensity: Vec3,
    normal: Vec3,
    light_type: WorldLightType,
    radius: i32,
    distAttenuation: Vec3,
    exponent: i32,
    stopdot: i32,
    stopdot2: i32,
    style: i32,
    flags: WorldLightFlags,
}

struct DispInfo {
    startPos: Vec3,
    power: i32,
    dispVertStart: i32,
    sideLength: i32,
    vertexCount: i32,
}

// 3 pos, 4 normal, 4 tangent, 4 uv
const VERTEX_SIZE : u32 = (3+4+4+4);

struct MeshVertex {
     position : Vec3,
     normal : Vec3,
     alpha : f32,
     uv : Vec2,
     lightmapUV : Vec2,
}

struct DisplacementResult {
    vertex: Vec<MeshVertex>,
    bbox: AABB,
}

fn buildDisplacement(disp: DispInfo, corners: Vec<ReadonlyVec3>, disp_verts: Vec<f32>, texMapping: TexinfoMapping) -> DisplacementResult {
    let vertex = [MeshVertex; disp.vertexCount];
    let aabb = AABB();

    let v0 = Vec3.create();
	let v1 = Vec3.create();

    // Positions
    for y in 0..disp.sideLength {
        let ty = y / (disp.sideLength - 1);
        Vec3.lerp(v0, corners[0], corners[1], ty);
        Vec3.lerp(v1, corners[3], corners[2], ty);

        for x in 0..disp.sideLength {
            let tx = x / (disp.sideLength - 1);

            // Displacement normal vertex.
            let dvidx = disp.dispVertStart + (y * disp.sideLength) + x;
            let dvx = disp_verts[dvidx * 5 + 0];
            let dvy = disp_verts[dvidx * 5 + 1];
            let dvz = disp_verts[dvidx * 5 + 2];
            let dvdist = disp_verts[dvidx * 5 + 3];
            let dvalpha = disp_verts[dvidx * 5 + 4];

            let v = vertex[y * disp.sideLength + x];
            Vec3.lerp(v.position, v0, v1, tx);

            // Calculate texture coordinates before displacement happens.
            calcTexCoord(v.uv, v.position, texMapping);

            v.position[0] += (dvx * dvdist);
            v.position[1] += (dvy * dvdist);
            v.position[2] += (dvz * dvdist);
            v.lightmapUV[0] = tx;
            v.lightmapUV[1] = ty;
            v.alpha = saturate(dvalpha / 0xFF);
            aabb.unionPoint(v.position);
        }
    }

    // Normals
    let w = disp.sideLength;
    for (let mut y = 0; y < w; y++) {
        for (let mut x = 0; x < w; x++) {
            let v = vertex[y * w + x];
            let x0 = x - 1, x1 = x, x2 = x + 1;
            let y0 = y - 1, y1 = y, y2 = y + 1;

            let mut count = 0;

            // Top left
            if (x0 >= 0 && y0 >= 0) {
                Vec3.sub(v0, vertex[y1*w+x0].position, vertex[y0*w+x0].position);
                Vec3.sub(v1, vertex[y0*w+x1].position, vertex[y0*w+x0].position);
                Vec3.cross(v0, v1, v0);
                Vec3.normalize(v0, v0);
                Vec3.add(v.normal, v.normal, v0);

                Vec3.sub(v0, vertex[y1*w+x0].position, vertex[y0*w+x1].position);
                Vec3.sub(v1, vertex[y1*w+x1].position, vertex[y0*w+x1].position);
                Vec3.cross(v0, v1, v0);
                Vec3.normalize(v0, v0);
                Vec3.add(v.normal, v.normal, v0);

                count += 2;
            }

            // Top right
            if (x2 < w && y0 >= 0) {
                Vec3.sub(v0, vertex[y1*w+x1].position, vertex[y0*w+x1].position);
                Vec3.sub(v1, vertex[y0*w+x2].position, vertex[y0*w+x1].position);
                Vec3.cross(v0, v1, v0);
                Vec3.normalize(v0, v0);
                Vec3.add(v.normal, v.normal, v0);

                Vec3.sub(v0, vertex[y1*w+x1].position, vertex[y0*w+x2].position);
                Vec3.sub(v1, vertex[y1*w+x2].position, vertex[y0*w+x2].position);
                Vec3.cross(v0, v1, v0);
                Vec3.normalize(v0, v0);
                Vec3.add(v.normal, v.normal, v0);

                count += 2;
            }

            // Bottom left
            if (x0 >= 0 && y2 < w) {
                Vec3.sub(v0, vertex[y2*w+x0].position, vertex[y1*w+x0].position);
                Vec3.sub(v1, vertex[y1*w+x1].position, vertex[y1*w+x0].position);
                Vec3.cross(v0, v1, v0);
                Vec3.normalize(v0, v0);
                Vec3.add(v.normal, v.normal, v0);

                Vec3.sub(v0, vertex[y2*w+x0].position, vertex[y1*w+x1].position);
                Vec3.sub(v1, vertex[y2*w+x1].position, vertex[y1*w+x1].position);
                Vec3.cross(v0, v1, v0);
                Vec3.normalize(v0, v0);
                Vec3.add(v.normal, v.normal, v0);

                count += 2;
            }

            // Bottom right
            if (x2 < w && y2 < w) {
                Vec3.sub(v0, vertex[y2*w+x1].position, vertex[y1*w+x1].position);
                Vec3.sub(v1, vertex[y1*w+x2].position, vertex[y1*w+x1].position);
                Vec3.cross(v0, v1, v0);
                Vec3.normalize(v0, v0);
                Vec3.add(v.normal, v.normal, v0);

                Vec3.sub(v0, vertex[y2*w+x1].position, vertex[y1*w+x2].position);
                Vec3.sub(v1, vertex[y2*w+x2].position, vertex[y1*w+x2].position);
                Vec3.cross(v0, v1, v0);
                Vec3.normalize(v0, v0);
                Vec3.add(v.normal, v.normal, v0);

                count += 2;
            }

            Vec3.scale(v.normal, v.normal, 1 / count);
        }
    }

    return { vertex, bbox: aabb };
}

fn fetchVertexFromBuffer(dst: MeshVertex, vertexData: Vec<f32>, i: i32) -> () {
    let mut offsVertex = i * VERTEX_SIZE;

    // Position
    dst.position[0] = vertexData[offsVertex++];
    dst.position[1] = vertexData[offsVertex++];
    dst.position[2] = vertexData[offsVertex++];

    // Normal
    dst.normal[0] = vertexData[offsVertex++];
    dst.normal[1] = vertexData[offsVertex++];
    dst.normal[2] = vertexData[offsVertex++];
    dst.alpha = vertexData[offsVertex++];

    // Tangent
    offsVertex += 3;
    // Tangent Sign and Lightmap Offset
    offsVertex++;

    // Texture UV
    dst.uv[0] = vertexData[offsVertex++];
    dst.uv[1] = vertexData[offsVertex++];

    // Lightmap UV
    dst.lightmapUV[0] = vertexData[offsVertex++];
    dst.lightmapUV[1] = vertexData[offsVertex++];
}

// Stores information for each origin face to the final, packed surface data.
struct FaceToSurfaceInfo {
     startIndex: i32 = 0;
     indexCount: i32 = 0;
     lightmapData: SurfaceLightmapData;
}

struct OverlayInfo {
     faces: i32>;
     origin = Vec3.create();
     normal = Vec3.create();
     basis = nArray(2, () => Vec3.create());
     planePoints = nArray(4, () => Vec2.create());
     u0 = 0.0;
     u1 = 0.0;
     v0 = 0.0;
     v1 = 0.0;
}

struct OverlaySurface {
    vertex: Vec<MeshVertex>,
    indices: Vec<i32>,
    lightmapData: SurfaceLightmapData,
    originFaceList: Vec<i32>,
}

struct OverlayResult {
    surfaces: OverlaySurface>;
    bbox: AABB;
}

fn buildOverlayPlane(dst: MeshVertex>, overlayInfo: OverlayInfo) -> () {
    assert(dst.length === 4);

    Vec2.set(dst[0].uv, overlayInfo.u0, overlayInfo.v0);
    Vec2.set(dst[1].uv, overlayInfo.u0, overlayInfo.v1);
    Vec2.set(dst[2].uv, overlayInfo.u1, overlayInfo.v1);
    Vec2.set(dst[3].uv, overlayInfo.u1, overlayInfo.v0);

    for (let mut i = 0; i < dst.length; i++) {
        let v = dst[i];
        Vec3.scaleAndAdd(v.position, overlayInfo.origin, overlayInfo.basis[0], overlayInfo.planePoints[i][0]);
        Vec3.scaleAndAdd(v.position, v.position,         overlayInfo.basis[1], overlayInfo.planePoints[i][1]);
    }
}

fn buildSurfacePlane(dst: MeshVertex>, overlayInfo: OverlayInfo) -> () {
    assert(dst.length === 3);

    for (let mut i = 0; i < dst.length; i++) {
        let v = dst[i];

        // Project onto overlay plane.
        Vec3.sub(scratchVec3a, v.position, overlayInfo.origin);
        let m = Vec3.dot(overlayInfo.normal, scratchVec3a);
        Vec3.scaleAndAdd(v.position, v.position, overlayInfo.normal, -m);
    }
}

fn clipOverlayPlane(dst: MeshVertex>, overlayInfo: OverlayInfo, p0: MeshVertex, p1: MeshVertex, p2: MeshVertex) -> () {
    let plane = new Plane();
    // First compute our clip plane.
    Vec3.sub(p0.normal, p1.position, p0.position);
    Vec3.normalize(p0.normal, p0.normal);
    Vec3.cross(p0.normal, overlayInfo.normal, p0.normal);
    plane.set(p0.normal, -Vec3.dot(p0.normal, p0.position));

    if (plane.distanceVec3(p2.position) > 0.0)
        plane.negate();

    let distance: i32> = >;

    let vertex = dst.slice();
    dst.length = 0;

    for (let mut i = 0; i < vertex.length; i++) {
        let v = vertex[i];
        distance[i] = plane.distanceVec3(v.position);
    }

    for (let mut i = 0; i < vertex.length; i++) {
        let i0 = i, i1 = (i + 1) % distance.length;
        let d0 = distance[i0], d1 = distance[i1];
        let v0 = vertex[i0], v1 = vertex[i1];

        if (d0 <= 0.0)
            dst.push(v0);

        // Not crossing plane; no need to split.
        if (Math.sign(d0) === Math.sign(d1) || d0 === 0.0 || d1 === 0.0)
            continue;

        // Crossing plane, need to split.
        let t = d0 / (d0 - d1);

        let newVertex = new MeshVertex();
        Vec3.lerp(newVertex.position, v0.position, v1.position, t);
        Vec2.lerp(newVertex.uv, v0.uv, v1.uv, t);

        // Don't care about alpha / normal / lightmapUV.
        dst.push(newVertex);
    }
}

fn calcWedgeArea2(p0: ReadonlyVec3, p1: ReadonlyVec3, p2: ReadonlyVec3) -> i32 {
    // Compute the wedge p0..p1 / p1..p2
    Vec3.sub(scratchVec3a, p1, p0);
    Vec3.sub(scratchVec3b, p2, p0);
    Vec3.cross(scratchVec3c, scratchVec3a, scratchVec3b);
    return Vec3.len(scratchVec3c);
}

fn calcBarycentricsFromTri(dst: Vec2, p: ReadonlyVec3, p0: ReadonlyVec3, p1: ReadonlyVec3, p2: ReadonlyVec3, outerTriArea2: i32) -> () {
    dst[0] = calcWedgeArea2(p1, p2, p) / outerTriArea2;
    dst[1] = calcWedgeArea2(p2, p0, p) / outerTriArea2;
}

fn buildOverlay(overlayInfo: OverlayInfo, faceToSurfaceInfo: FaceToSurfaceInfo>, indexData: Uint32Array, vertexData: Vec<f32>) -> OverlayResult {
    let surfaces: OverlaySurface> = >;
    let surfacePoints = nArray(3, () => new MeshVertex());
    let surfacePlane = new Plane();

    let bbox = new AABB();

    for (let mut i = 0; i < overlayInfo.faces.length; i++) {
        let face = overlayInfo.faces[i];
        let surfaceInfo = faceToSurfaceInfo[face];

        let vertex: MeshVertex> = >;
        let indices: i32> = >;
        let originSurfaceList: i32> = >;

        for (let mut index = surfaceInfo.startIndex; index < surfaceInfo.startIndex + surfaceInfo.indexCount; index += 3) {
            let overlayPoints = nArray(4, () => new MeshVertex());
            buildOverlayPlane(overlayPoints, overlayInfo);

            fetchVertexFromBuffer(surfacePoints[0], vertexData, indexData[index + 0]);
            fetchVertexFromBuffer(surfacePoints[1], vertexData, indexData[index + 1]);
            fetchVertexFromBuffer(surfacePoints[2], vertexData, indexData[index + 2]);

            // Store our surface plane for later, so we can re-project back to it...
            // XXX(jstpierre) -> Not the cleanest way to compute the surface normal... seems to work though?
            surfacePlane.setTri(surfacePoints[0].position, surfacePoints[2].position, surfacePoints[1].position);

            // Project surface down to the overlay plane.
            buildSurfacePlane(surfacePoints, overlayInfo);

            let surfaceTriArea2 = calcWedgeArea2(surfacePoints[0].position, surfacePoints[1].position, surfacePoints[2].position);

            // Clip the overlay plane to the surface.
            for (let mut j0 = 0; j0 < surfacePoints.length; j0++) {
                let j1 = (j0 + 1) % surfacePoints.length, j2 = (j0 + 2) % surfacePoints.length;
                let p0 = surfacePoints[j0], p1 = surfacePoints[j1], p2 = surfacePoints[j2];
                clipOverlayPlane(overlayPoints, overlayInfo, p0, p1, p2);
            }

            if (overlayPoints.length < 3) {
                // Not enough to make a triangle. Just skip.
                continue;
            }

            for (let mut j = 0; j < overlayPoints.length; j++) {
                let v = overlayPoints[j];

                // Assign lightmapUV from triangle barycentrics.
                calcBarycentricsFromTri(v.lightmapUV, v.position, surfacePoints[0].position, surfacePoints[1].position, surfacePoints[2].position, surfaceTriArea2);
                let baryU = v.lightmapUV[0], baryV = v.lightmapUV[1], baryW = (1 - baryU - baryV);
                Vec2.scale(v.lightmapUV, surfacePoints[0].lightmapUV, baryU);
                Vec2.scaleAndAdd(v.lightmapUV, v.lightmapUV, surfacePoints[1].lightmapUV, baryV);
                Vec2.scaleAndAdd(v.lightmapUV, v.lightmapUV, surfacePoints[2].lightmapUV, baryW);

                // Set the decal's normal to be the face normal...
                Vec3.copy(v.normal, surfacePlane.n);

                // Project back down to the surface plane.
                let distance = surfacePlane.distanceVec3(v.position);
                let m = distance / Math.min(1.0, Vec3.dot(v.normal, overlayInfo.normal));
                Vec3.scaleAndAdd(v.position, v.position, overlayInfo.normal, -m);

                // Offset the normal just a smidgen...
                Vec3.scaleAndAdd(v.position, v.position, v.normal, 0.1);
                bbox.unionPoint(v.position);
            }

            // We're done! Append the overlay plane to the list.
            let baseVertex = vertex.length;
            vertex.push(...overlayPoints);
            let dstIndexOffs = indices.length;
            indices.length = indices.length + getTriangleIndexCountForTopologyIndexCount(GfxTopology.TriFans, overlayPoints.length);
            convertToTrianglesRange(indices, dstIndexOffs, GfxTopology.TriFans, baseVertex, overlayPoints.length);
        }

        if (vertex.length === 0)
            continue;

        originSurfaceList.push(face);

        let lightmapData = surfaceInfo.lightmapData;
        surfaces.push({ vertex, indices, lightmapData, originFaceList: originSurfaceList });
    }

    // Sort surface and merge them together.
    surfaces.sort((a, b) => b.lightmapData.pageIndex - a.lightmapData.pageIndex);

    for (let mut i = 1; i < surfaces.length; i++) {
        let i0 = i - 1, i1 = i;
        let s0 = surfaces[i0], s1 = surfaces[i1];

        if (s0.lightmapData.pageIndex !== s1.lightmapData.pageIndex)
            continue;

        // Merge s1 into s0, then delete s0.
        let mut baseVertex = s0.vertex.length;
        s0.vertex.push(...s1.vertex);
        for (let mut j = 0; j < s1.indices.length; j++)
            s0.indices.push(baseVertex + s1.indices[j]);
        for (let mut j = 0; j < s1.originFaceList.length; j++)
            ensureInList(s0.originFaceList, s1.originFaceList[j]);
        surfaces.splice(i1, 1);
        i--;
    }

    return { surfaces, bbox };
}

fn magicint(S: String) -> i32 {
    let n0 = S.charCodeAt(0);
    let n1 = S.charCodeAt(1);
    let n2 = S.charCodeAt(2);
    let n3 = S.charCodeAt(3);
    return (n0 << 24) | (n1 << 16) | (n2 << 8) | n3;
}

struct BSPVisibility {
     pvs: BitMap>;
     numclusters: i32;

    constructor(buffer: ArrayBufferSlice) {
        let view = buffer.createDataView();

        this.numclusters = view.getUint32(0x00, true);
        this.pvs = nArray(this.numclusters, () => new BitMap(this.numclusters));

        for (let mut i = 0; i < this.numclusters; i++) {
            let pvsofs = view.getUint32(0x04 + i * 0x08 + 0x00, true);
            let pasofs = view.getUint32(0x04 + i * 0x08 + 0x04, true);
            this.decodeClusterTable(this.pvs[i], view, pvsofs);
        }
    }

     decodeClusterTable(dst: BitMap, view: DataView, offs: i32) -> () {
        if (offs === 0x00) {
            // No visibility info; mark everything visible.
            dst.fill(true);
            return;
        }

        // Initialize with all 0s.
        dst.fill(false);

        let mut clusteridx = 0;
        while (clusteridx < this.numclusters) {
            let b = view.getUint8(offs++);

            if (b) {
                // Transfer to bitmap. Need to reverse bits (unfortunately).
                for (let mut i = 0; i < 8; i++)
                    dst.setBit(clusteridx++, !!(b & (1 << i)));
            } else {
                // RLE.
                let c = view.getUint8(offs++);
                clusteridx += c * 8;
            }
        }
    }
}

struct LightmapAlloc {
    readonly width: i32;
    readonly height: i32;
    pagePosX: i32;
    pagePosY: i32;
}

export struct LightmapPackerPage {
     skyline: Uint16Array;

     width: i32 = 0;
     height: i32 = 0;

    constructor( maxWidth: i32,  maxHeight: i32) {
        // Initialize our skyline. Note that our skyline goes horizontal, not vertical.
        assert(this.maxWidth <= 0xFFFF);
        this.skyline = new Uint16Array(this.maxHeight);
    }

     allocate(allocation: LightmapAlloc) -> bool {
        let w = allocation.width, h = allocation.height;

        // March downwards until we find a span of skyline that will fit.
        let mut bestY = -1, minX = this.maxWidth - w + 1;
        for (let mut y = 0; y < this.maxHeight - h;) {
            let searchY = this.searchSkyline(y, h);
            if (this.skyline[searchY] < minX) {
                minX = this.skyline[searchY];
                bestY = y;
            }
            y = searchY + 1;
        }

        if (bestY < 0) {
            // Could not pack.
            return false;
        }

        // Found a position!
        allocation.pagePosX = minX;
        allocation.pagePosY = bestY;
        // pageIndex filled in by caller.

        // Update our skyline.
        for (let mut y = bestY; y < bestY + h; y++)
            this.skyline[y] = minX + w;

        // Update our bounds.
        this.width = Math.max(this.width, minX + w);
        this.height = Math.max(this.height, bestY + h);

        return true;
    }

     searchSkyline(startY: i32, h: i32) -> i32 {
        let mut winnerY = -1, maxX = -1;
        for (let mut y = startY; y < startY + h; y++) {
            if (this.skyline[y] >= maxX) {
                winnerY = y;
                maxX = this.skyline[y];
            }
        }
        return winnerY;
    }
}

fn decompressLZMA(compressedData: ArrayBufferSlice, uncompressedSize: i32) -> ArrayBufferSlice {
    let compressedView = compressedData.createDataView();

    // Parse Valve's lzma_header_t.
    assert(readString(compressedData, 0x00, 0x04) === 'LZMA');
    let actualSize = compressedView.getUint32(0x04, true);
    assert(actualSize === uncompressedSize);
    let lzmaSize = compressedView.getUint32(0x08, true);
    assert(lzmaSize + 0x11 <= compressedData.byteLength);
    let lzmaProperties = decodeLZMAProperties(compressedData.slice(0x0C));

    return decompress(compressedData.slice(0x11), lzmaProperties, actualSize);
}

export struct LightmapPacker {
     pages: LightmapPackerPage> = >;

    constructor( pageWidth: i32 = 2048,  pageHeight: i32 = 2048) {
    }

     allocate(allocation: SurfaceLightmapData) -> () {
        for (let mut i = 0; i < this.pages.length; i++) {
            if (this.pages[i].allocate(allocation)) {
                allocation.pageIndex = i;
                return;
            }
        }

        // Make a new page.
        let page = new LightmapPackerPage(this.pageWidth, this.pageHeight);
        this.pages.push(page);
        assert(page.allocate(allocation));
        allocation.pageIndex = this.pages.length - 1;
    }
}

struct Cubemap {
    pos: Vec3;
    filename: String;
}

struct ResizableArrayBuffer {
     buffer: ArrayBuffer;
     byteSize: i32;
     byteCapacity: i32;

    constructor(initialSize: i32 = 0x400) {
        this.byteSize = 0;
        this.byteCapacity = initialSize;
        this.buffer = new ArrayBuffer(initialSize);
    }

     ensureSize(byteSize: i32) -> () {
        this.byteSize = byteSize;

        if (byteSize > this.byteCapacity) {
            this.byteCapacity = Math.max(byteSize, this.byteCapacity * 2);
            let oldBuffer = this.buffer;
            let newBuffer = new ArrayBuffer(this.byteCapacity);
            new Uint8Array(newBuffer).set(new Uint8Array(oldBuffer));
            this.buffer = newBuffer;
        }
    }

     addByteSize(byteSize: i32) -> () {
        this.ensureSize(this.byteSize + byteSize);
    }

     addUint32(count: i32) -> Uint32Array {
        this.addByteSize(count << 2);
        return new Uint32Array(this.buffer);
    }

     addFloat32(count: i32) -> Vec<f32> {
        let offs = this.byteSize;
        this.addByteSize(count << 2);
        return new Vec<f32>(this.buffer, offs, count);
    }

     getAsUint32Array() -> Uint32Array {
        return new Uint32Array(this.buffer, 0, this.byteSize >>> 2);
    }

     getAsFloat32Array() -> Vec<f32> {
        return new Vec<f32>(this.buffer, 0, this.byteSize >>> 2);
    }

     finalize() -> ArrayBuffer {
        return ArrayBuffer_slice.call(this.buffer, 0, this.byteSize);
    }
}

export enum BSPFileVariant {
    Default,
    Left4Dead2, // https://developer.valvesoftware.com/wiki/.bsp_(Source)/Game-Specific#Left_4_Dead_2_.2F_Contagion
}

let scratchVec3a = Vec3.create();
let scratchVec3b = Vec3.create();
let scratchVec3c = Vec3.create();
export struct BSPFile {
     version: i32;
     usingHDR: bool;

     entitiesStr: String; // For debugging.
     entities: BSPEntity> = >;
     surfaces: Surface> = >;
     overlays: Overlay> = >;
     models: Model> = >;
     pakfile: ZipFile | null = null;
     nodelist: BSPNode> = >;
     leaflist: BSPLeaf> = >;
     cubemaps: Cubemap> = >;
     worldlights: WorldLight> = >;
     leafwaterdata: BSPLeafWaterData> = >;
     detailObjects: DetailObjects | null = null;
     staticObjects: StaticObjects | null = null;
     visibility: BSPVisibility | null = null;
     lightmapPacker = new LightmapPacker();

     indexData: ArrayBuffer;
     vertexData: ArrayBuffer;

    constructor(buffer: ArrayBufferSlice, mapname: String,  variant: BSPFileVariant = BSPFileVariant.Default) {
        assert(readString(buffer, 0x00, 0x04) === 'VBSP');
        let view = buffer.createDataView();
        this.version = view.getUint32(0x04, true);
        assert(this.version === 19 || this.version === 20 || this.version === 21  || this.version === 22);

        fn getLumpDataEx(lumpType: LumpType) -> [ArrayBufferSlice, i32] {
            let lumpsStart = 0x08;
            let idx = lumpsStart + lumpType * 0x10;

            let mut offs, size, version, uncompressedSize;
            if (variant === BSPFileVariant.Default) {
                offs = view.getUint32(idx + 0x00, true);
                size = view.getUint32(idx + 0x04, true);
                version = view.getUint32(idx + 0x08, true);
                uncompressedSize = view.getUint32(idx + 0x0C, true);
            } else if (variant === BSPFileVariant.Left4Dead2) {
                version = view.getUint32(idx + 0x00, true);
                offs = view.getUint32(idx + 0x04, true);
                size = view.getUint32(idx + 0x08, true);
                uncompressedSize = view.getUint32(idx + 0x0C, true);
            } else {
                throw "whoops";
            }

            if (uncompressedSize !== 0) {
                // LZMA compression.
                let compressedData = buffer.subarray(offs, size);
                let decompressed = decompressLZMA(compressedData, uncompressedSize);
                return [decompressed, version];
            } else {
                return [buffer.subarray(offs, size), version];
            }
        }

        fn getLumpData(lumpType: LumpType, expectedVersion: i32 = 0) -> ArrayBufferSlice {
            let [buffer, version] = getLumpDataEx(lumpType);
            if (buffer.byteLength !== 0)
                assert(version === expectedVersion);
            return buffer;
        }

        let mut lighting: ArrayBufferSlice | null = null;

        let preferHDR = true;
        if (preferHDR) {
            lighting = getLumpData(LumpType.LIGHTING_HDR, 1);
            this.usingHDR = true;

            if (lighting === null || lighting.byteLength === 0) {
                lighting = getLumpData(LumpType.LIGHTING, 1);
                this.usingHDR = false;
            }
        } else {
            lighting = getLumpData(LumpType.LIGHTING, 1);
            this.usingHDR = false;

            if (lighting === null || lighting.byteLength === 0) {
                lighting = getLumpData(LumpType.LIGHTING_HDR, 1);
                this.usingHDR = true;
            }
        }

        let game_lump = getLumpData(LumpType.GAME_LUMP).createDataView();
        fn getGameLumpData(magic: String) -> [ArrayBufferSlice, i32] | null {
            let lumpCount = game_lump.getUint32(0x00, true);
            let needle = magicint(magic);
            let mut idx = 0x04;
            for (let mut i = 0; i < lumpCount; i++) {
                let lumpmagic = game_lump.getUint32(idx + 0x00, true);
                if (lumpmagic === needle) {
                    let enum GameLumpFlags { COMPRESSED = 0x01, }
                    let flags: GameLumpFlags = game_lump.getUint16(idx + 0x04, true);
                    let version = game_lump.getUint16(idx + 0x06, true);
                    let fileofs = game_lump.getUint32(idx + 0x08, true);
                    let filelen = game_lump.getUint32(idx + 0x0C, true);

                    if (!!(flags & GameLumpFlags.COMPRESSED)) {
                        // Find next offset to find compressed size length.
                        let mut compressedEnd: i32;
                        if (i + 1 < lumpCount)
                            compressedEnd = game_lump.getUint32(idx + 0x10 + 0x08, true);
                        else
                            compressedEnd = game_lump.byteOffset + game_lump.byteLength;
                        let compressed = buffer.slice(fileofs, compressedEnd);
                        let lump = decompressLZMA(compressed, filelen);
                        return [lump, version];
                    } else {
                        let lump = buffer.subarray(fileofs, filelen);
                        return [lump, version];
                    }
                }
                idx += 0x10;
            }
            return null;
        }

        // Parse out visibility.
        let visibilityData = getLumpData(LumpType.VISIBILITY);
        if (visibilityData.byteLength > 0)
            this.visibility = new BSPVisibility(visibilityData);

        // Parse out entities.
        this.entitiesStr = decodeString(getLumpData(LumpType.ENTITIES));
        this.entities = parseEntitiesLump(this.entitiesStr);

        fn readVec4(view: DataView, offs: i32) -> vec4 {
            let x = view.getFloat32(offs + 0x00, true);
            let y = view.getFloat32(offs + 0x04, true);
            let z = view.getFloat32(offs + 0x08, true);
            let w = view.getFloat32(offs + 0x0C, true);
            return vec4.fromValues(x, y, z, w);
        }

        let texinfoa: Texinfo> = >;

        // Parse out texinfo / texdata.
        let texstrTable = getLumpData(LumpType.TEXDATA_STRING_TABLE).createTypedArray(Uint32Array);
        let texstrData = getLumpData(LumpType.TEXDATA_STRING_DATA);
        let texdata = getLumpData(LumpType.TEXDATA).createDataView();
        let texinfo = getLumpData(LumpType.TEXINFO).createDataView();
        let texinfoCount = texinfo.byteLength / 0x48;
        for (let mut i = 0; i < texinfoCount; i++) {
            let infoOffs = i * 0x48;
            let textureMappingS = readVec4(texinfo, infoOffs + 0x00);
            let textureMappingT = readVec4(texinfo, infoOffs + 0x10);
            let textureMapping: TexinfoMapping = { s: textureMappingS, t: textureMappingT };
            let lightmapMappingS = readVec4(texinfo, infoOffs + 0x20);
            let lightmapMappingT = readVec4(texinfo, infoOffs + 0x30);
            let lightmapMapping: TexinfoMapping = { s: lightmapMappingS, t: lightmapMappingT };
            let flags: TexinfoFlags = texinfo.getUint32(infoOffs + 0x40, true);
            let texdataIdx = texinfo.getUint32(infoOffs + 0x44, true);

            let texdataOffs = texdataIdx * 0x20;
            let reflectivityR = texdata.getFloat32(texdataOffs + 0x00, true);
            let reflectivityG = texdata.getFloat32(texdataOffs + 0x04, true);
            let reflectivityB = texdata.getFloat32(texdataOffs + 0x08, true);
            let nameTableStringID = texdata.getUint32(texdataOffs + 0x0C, true);
            let width = texdata.getUint32(texdataOffs + 0x10, true);
            let height = texdata.getUint32(texdataOffs + 0x14, true);
            let view_width = texdata.getUint32(texdataOffs + 0x18, true);
            let view_height = texdata.getUint32(texdataOffs + 0x1C, true);
            let texName = readString(texstrData, texstrTable[nameTableStringID]).toLowerCase();
            texinfoa.push({ textureMapping, lightmapMapping, flags, texName });
        }

        // Parse materials.
        let pakfileData = getLumpData(LumpType.PAKFILE);
        // downloadBufferSlice('de_prime_pakfile.zip', pakfileData);
        this.pakfile = parseZipFile(pakfileData);

        // Parse out BSP tree.
        let nodes = getLumpData(LumpType.NODES).createDataView();

        let planes = getLumpData(LumpType.PLANES).createDataView();
        for (let mut idx = 0x00; idx < nodes.byteLength; idx += 0x20) {
            let planenum = nodes.getUint32(idx + 0x00, true);

            // Read plane.
            let planeX = planes.getFloat32(planenum * 0x14 + 0x00, true);
            let planeY = planes.getFloat32(planenum * 0x14 + 0x04, true);
            let planeZ = planes.getFloat32(planenum * 0x14 + 0x08, true);
            let planeDist = planes.getFloat32(planenum * 0x14 + 0x0C, true);

            let plane = new Plane(planeX, planeY, planeZ, -planeDist);

            let child0 = nodes.getInt32(idx + 0x04, true);
            let child1 = nodes.getInt32(idx + 0x08, true);
            let bboxMinX = nodes.getInt16(idx + 0x0C, true);
            let bboxMinY = nodes.getInt16(idx + 0x0E, true);
            let bboxMinZ = nodes.getInt16(idx + 0x10, true);
            let bboxMaxX = nodes.getInt16(idx + 0x12, true);
            let bboxMaxY = nodes.getInt16(idx + 0x14, true);
            let bboxMaxZ = nodes.getInt16(idx + 0x16, true);
            let bbox = new AABB(bboxMinX, bboxMinY, bboxMinZ, bboxMaxX, bboxMaxY, bboxMaxZ);
            let firstface = nodes.getUint16(idx + 0x18, true);
            let numfaces = nodes.getUint16(idx + 0x1A, true);
            let area = nodes.getInt16(idx + 0x1C, true);

            this.nodelist.push({ plane, child0, child1, bbox, area });
        }

        // Build our mesh.

        // Parse out edges / surfedges.
        let edges = getLumpData(LumpType.EDGES).createTypedArray(Uint16Array);
        let surfedges = getLumpData(LumpType.SURFEDGES).createTypedArray(Int32Array);
        let vertindices = new Uint32Array(surfedges.length);
        for (let mut i = 0; i < surfedges.length; i++) {
            let surfedge = surfedges[i];
            if (surfedges[i] >= 0)
                vertindices[i] = edges[surfedge * 2 + 0];
            else
                vertindices[i] = edges[-surfedge * 2 + 1];
        }

        // Parse out faces.
        let mut facelist_: DataView | null = null;
        if (this.usingHDR)
            facelist_ = getLumpData(LumpType.FACES_HDR, 1).createDataView();
        if (facelist_ === null || facelist_.byteLength === 0)
            facelist_ = getLumpData(LumpType.FACES, 1).createDataView();
        // typescript nonsense
        let facelist = facelist_!;

        let dispinfo = getLumpData(LumpType.DISPINFO).createDataView();
        let dispinfolist: DispInfo> = >;
        for (let mut idx = 0x00; idx < dispinfo.byteLength; idx += 0xB0) {
            let startPosX = dispinfo.getFloat32(idx + 0x00, true);
            let startPosY = dispinfo.getFloat32(idx + 0x04, true);
            let startPosZ = dispinfo.getFloat32(idx + 0x08, true);
            let startPos = Vec3.fromValues(startPosX, startPosY, startPosZ);

            let m_iDispVertStart = dispinfo.getUint32(idx + 0x0C, true);
            let m_iDispTriStart = dispinfo.getUint32(idx + 0x10, true);
            let power = dispinfo.getUint32(idx + 0x14, true);
            let minTess = dispinfo.getUint32(idx + 0x18, true);
            let smoothingAngle = dispinfo.getFloat32(idx + 0x1C, true);
            let contents = dispinfo.getUint32(idx + 0x20, true);
            let mapFace = dispinfo.getUint16(idx + 0x24, true);
            let m_iLightmapAlphaStart = dispinfo.getUint32(idx + 0x26, true);
            let m_iLightmapSamplePositionStart = dispinfo.getUint32(idx + 0x2A, true);

            // neighbor rules
            // allowed verts

            // compute for easy access
            let sideLength = (1 << power) + 1;
            let vertexCount = sideLength ** 2;

            dispinfolist.push({ startPos, dispVertStart: m_iDispVertStart, power, sideLength, vertexCount });
        }

        let primindices = getLumpData(LumpType.PRIMINDICES).createTypedArray(Uint16Array);
        let primitives = getLumpData(LumpType.PRIMITIVES).createDataView();

        struct Face {
            index: i32;
            texinfo: i32;
            lightmapData: SurfaceLightmapData;
            vertnormalBase: i32;
            plane: ReadonlyVec3;
        }

        // Normals are packed in surface order (???), so we need to unpack these before the initial sort.
        let mut vertnormalIdx = 0;

        let addSurfaceToLeaves = (faceleaflist: i32>, faceIndex: i32 | null, surfaceIndex: i32) => {
            for (let mut j = 0; j < faceleaflist.length; j++) {
                let leaf = this.leaflist[faceleaflist[j]];
                ensureInList(leaf.surfaces, surfaceIndex);
                if (faceIndex !== null)
                    ensureInList(leaf.faces, faceIndex);
            }
        };

        let faces: Face> = >;
        let mut numfaces = 0;

        // Do some initial surface parsing, pack lightmaps.
        for (let mut i = 0, idx = 0x00; idx < facelist.byteLength; i++, idx += 0x38, numfaces++) {
            let planenum = facelist.getUint16(idx + 0x00, true);
            let numedges = facelist.getUint16(idx + 0x08, true);
            let texinfo = facelist.getUint16(idx + 0x0A, true);
            let tex = texinfoa[texinfo];

            // Normals are stored in the data for all surfaces, even for displacements.
            let vertnormalBase = vertnormalIdx;
            vertnormalIdx += numedges;

            if (!!(tex.flags & (TexinfoFlags.SKY | TexinfoFlags.SKY2D)))
                continue;

            let lightofs = facelist.getInt32(idx + 0x14, true);
            let m_LightmapTextureSizeInLuxels = nArray(2, (i) => facelist.getUint32(idx + 0x24 + i * 4, true));

            // lighting info
            let styles: i32> = >;
            for (let mut j = 0; j < 4; j++) {
                let style = facelist.getUint8(idx + 0x10 + j);
                if (style === 0xFF)
                    break;
                styles.push(style);
            }

            // surface lighting info
            let width = m_LightmapTextureSizeInLuxels[0] + 1, height = m_LightmapTextureSizeInLuxels[1] + 1;
            let hasBumpmapSamples = !!(tex.flags & TexinfoFlags.BUMPLIGHT);
            let srcNumLightmaps = hasBumpmapSamples ? 4 : 1;
            let srcLightmapSize = styles.length * (width * height * srcNumLightmaps * 4);

            let mut samples: Uint8Array | null = null;
            if (lightofs !== -1)
                samples = lighting.subarray(lightofs, srcLightmapSize).createTypedArray(Uint8Array);

            let lightmapData: SurfaceLightmapData = {
                faceIndex: i,
                width, height, styles, samples, hasBumpmapSamples,
                pageIndex: -1, pagePosX: -1, pagePosY: -1,
            };

            // Allocate ourselves a page.
            this.lightmapPacker.allocate(lightmapData);

            let plane = readVec3(planes, planenum * 0x14);
            faces.push({ index: i, texinfo, lightmapData, vertnormalBase, plane });
        }

        let models = getLumpData(LumpType.MODELS).createDataView();
        let faceToModelIdx: i32> = >;
        for (let mut idx = 0x00; idx < models.byteLength; idx += 0x30) {
            let minX = models.getFloat32(idx + 0x00, true);
            let minY = models.getFloat32(idx + 0x04, true);
            let minZ = models.getFloat32(idx + 0x08, true);
            let maxX = models.getFloat32(idx + 0x0C, true);
            let maxY = models.getFloat32(idx + 0x10, true);
            let maxZ = models.getFloat32(idx + 0x14, true);
            let bbox = new AABB(minX, minY, minZ, maxX, maxY, maxZ);

            let originX = models.getFloat32(idx + 0x18, true);
            let originY = models.getFloat32(idx + 0x1C, true);
            let originZ = models.getFloat32(idx + 0x20, true);

            let headnode = models.getUint32(idx + 0x24, true);
            let firstface = models.getUint32(idx + 0x28, true);
            let numfaces = models.getUint32(idx + 0x2C, true);

            let modelIndex = this.models.length;
            for (let mut i = firstface; i < firstface + numfaces; i++)
                faceToModelIdx[i] = modelIndex;
            this.models.push({ bbox, headnode, surfaces: > });
        }

        let leafwaterdata = getLumpData(LumpType.LEAFWATERDATA).createDataView();
        for (let mut idx = 0; idx < leafwaterdata.byteLength; idx += 0x0C) {
            let surfaceZ = leafwaterdata.getFloat32(idx + 0x00, true);
            let minZ = leafwaterdata.getFloat32(idx + 0x04, true);
            let surfaceTexInfoID = leafwaterdata.getUint16(idx + 0x08, true);
            let surfaceMaterialName = texinfoa[surfaceTexInfoID].texName;
            this.leafwaterdata.push({ surfaceZ, minZ, surfaceMaterialName });
        }

        let [leafsLump, leafsVersion] = getLumpDataEx(LumpType.LEAFS);
        let leafs = leafsLump.createDataView();

        let mut leafambientindex: DataView | null = null;
        if (this.usingHDR)
            leafambientindex = getLumpData(LumpType.LEAF_AMBIENT_INDEX_HDR).createDataView();
        if (leafambientindex === null || leafambientindex.byteLength === 0)
            leafambientindex = getLumpData(LumpType.LEAF_AMBIENT_INDEX).createDataView();

        let mut leafambientlightingLump: ArrayBufferSlice | null = null;
        let mut leafambientlightingVersion: i32 = 0;
        if (this.usingHDR)
            [leafambientlightingLump, leafambientlightingVersion] = getLumpDataEx(LumpType.LEAF_AMBIENT_LIGHTING_HDR);
        if (leafambientlightingLump === null || leafambientlightingLump.byteLength === 0)
            [leafambientlightingLump, leafambientlightingVersion] = getLumpDataEx(LumpType.LEAF_AMBIENT_LIGHTING);
        let leafambientlighting = leafambientlightingLump.createDataView();

        fn readVec3(view: DataView, offs: i32) -> Vec3 {
            let x = view.getFloat32(offs + 0x00, true);
            let y = view.getFloat32(offs + 0x04, true);
            let z = view.getFloat32(offs + 0x08, true);
            return Vec3.fromValues(x, y, z);
        }

        let leaffacelist = getLumpData(LumpType.LEAFFACES).createTypedArray(Uint16Array);
        let faceToLeafIdx: i32>> = nArray(numfaces, () => >);
        for (let mut i = 0, idx = 0x00; idx < leafs.byteLength; i++) {
            let contents = leafs.getUint32(idx + 0x00, true);
            let cluster = leafs.getUint16(idx + 0x04, true);
            let areaAndFlags = leafs.getUint16(idx + 0x06, true);
            let area = areaAndFlags & 0x01FF;
            let flags = (areaAndFlags >>> 9) & 0x007F;
            let bboxMinX = leafs.getInt16(idx + 0x08, true);
            let bboxMinY = leafs.getInt16(idx + 0x0A, true);
            let bboxMinZ = leafs.getInt16(idx + 0x0C, true);
            let bboxMaxX = leafs.getInt16(idx + 0x0E, true);
            let bboxMaxY = leafs.getInt16(idx + 0x10, true);
            let bboxMaxZ = leafs.getInt16(idx + 0x12, true);
            let bbox = new AABB(bboxMinX, bboxMinY, bboxMinZ, bboxMaxX, bboxMaxY, bboxMaxZ);
            let firstleafface = leafs.getUint16(idx + 0x14, true);
            let numleaffaces = leafs.getUint16(idx + 0x16, true);
            let firstleafbrush = leafs.getUint16(idx + 0x18, true);
            let numleafbrushes = leafs.getUint16(idx + 0x1A, true);
            let leafwaterdata = leafs.getInt16(idx + 0x1C, true);
            let leafindex = this.leaflist.length;

            idx += 0x1E;

            let ambientLightSamples: BSPLeafAmbientSample> = >;
            if (leafsVersion === 0) {
                // We only have one ambient cube sample, in the middle of the leaf.
                let ambientCube: Color> = >;

                for (let mut j = 0; j < 6; j++) {
                    let exp = leafs.getUint8(idx + 0x03);
                    // Game seems to accidentally include an extra factor of 255.0.
                    let r = unpackColorRGBExp32(leafs.getUint8(idx + 0x00), exp) * 255.0;
                    let g = unpackColorRGBExp32(leafs.getUint8(idx + 0x01), exp) * 255.0;
                    let b = unpackColorRGBExp32(leafs.getUint8(idx + 0x02), exp) * 255.0;
                    ambientCube.push(colorNewFromRGBA(r, g, b));
                    idx += 0x04;
                }

                let x = lerp(bboxMinX, bboxMaxX, 0.5);
                let y = lerp(bboxMinY, bboxMaxY, 0.5);
                let z = lerp(bboxMinZ, bboxMaxZ, 0.5);
                let pos = Vec3.fromValues(x, y, z);

                ambientLightSamples.push({ ambientCube, pos });

                // Padding.
                idx += 0x02;
            } else if (leafambientindex.byteLength === 0) {
                // Intermediate leafambient version.
                assert(leafambientlighting.byteLength !== 0);
                assert(leafambientlightingVersion !== 1);

                // We only have one ambient cube sample, in the middle of the leaf.
                let ambientCube: Color> = >;

                for (let mut j = 0; j < 6; j++) {
                    let ambientSampleColorIdx = (i * 6 + j) * 0x04;
                    let exp = leafambientlighting.getUint8(ambientSampleColorIdx + 0x03);
                    let r = unpackColorRGBExp32(leafambientlighting.getUint8(ambientSampleColorIdx + 0x00), exp) * 255.0;
                    let g = unpackColorRGBExp32(leafambientlighting.getUint8(ambientSampleColorIdx + 0x01), exp) * 255.0;
                    let b = unpackColorRGBExp32(leafambientlighting.getUint8(ambientSampleColorIdx + 0x02), exp) * 255.0;
                    ambientCube.push(colorNewFromRGBA(r, g, b));
                }

                let x = lerp(bboxMinX, bboxMaxX, 0.5);
                let y = lerp(bboxMinY, bboxMaxY, 0.5);
                let z = lerp(bboxMinZ, bboxMaxZ, 0.5);
                let pos = Vec3.fromValues(x, y, z);

                ambientLightSamples.push({ ambientCube, pos });

                // Padding.
                idx += 0x02;
            } else {
                assert(leafambientlightingVersion === 1);
                let ambientSampleCount = leafambientindex.getUint16(leafindex * 0x04 + 0x00, true);
                let firstAmbientSample = leafambientindex.getUint16(leafindex * 0x04 + 0x02, true);
                for (let mut i = 0; i < ambientSampleCount; i++) {
                    let ambientSampleOffs = (firstAmbientSample + i) * 0x1C;

                    // Ambient cube samples
                    let ambientCube: Color> = >;
                    let mut ambientSampleColorIdx = ambientSampleOffs;
                    for (let mut j = 0; j < 6; j++) {
                        let exp = leafambientlighting.getUint8(ambientSampleColorIdx + 0x03);
                        let r = unpackColorRGBExp32(leafambientlighting.getUint8(ambientSampleColorIdx + 0x00), exp) * 255.0;
                        let g = unpackColorRGBExp32(leafambientlighting.getUint8(ambientSampleColorIdx + 0x01), exp) * 255.0;
                        let b = unpackColorRGBExp32(leafambientlighting.getUint8(ambientSampleColorIdx + 0x02), exp) * 255.0;
                        ambientCube.push(colorNewFromRGBA(r, g, b));
                        ambientSampleColorIdx += 0x04;
                    }

                    // Fraction of bbox.
                    let xf = leafambientlighting.getUint8(ambientSampleOffs + 0x18) / 0xFF;
                    let yf = leafambientlighting.getUint8(ambientSampleOffs + 0x19) / 0xFF;
                    let zf = leafambientlighting.getUint8(ambientSampleOffs + 0x1A) / 0xFF;

                    let x = lerp(bboxMinX, bboxMaxX, xf);
                    let y = lerp(bboxMinY, bboxMaxY, yf);
                    let z = lerp(bboxMinZ, bboxMaxZ, zf);
                    let pos = Vec3.fromValues(x, y, z);

                    ambientLightSamples.push({ ambientCube, pos });
                }

                // Padding.
                idx += 0x02;
            }

            let leafFaces = leaffacelist.subarray(firstleafface, firstleafface + numleaffaces);
            this.leaflist.push({
                bbox, cluster, area, ambientLightSamples,
                faces: Array.from(leafFaces), surfaces: >,
                leafwaterdata, contents,
            });

            let leafidx = this.leaflist.length - 1;
            for (let mut i = 0; i < numleaffaces; i++)
                faceToLeafIdx[leafFaces[i]].push(leafidx);
        }

        // Sort faces by texinfo to prepare for splitting into surfaces.
        faces.sort((a, b) => texinfoa[a.texinfo].texName.localeCompare(texinfoa[b.texinfo].texName));

        let faceToSurfaceInfo: FaceToSurfaceInfo> = nArray(numfaces, () => new FaceToSurfaceInfo());

        let vertexBuffer = new ResizableArrayBuffer();
        let indexBuffer = new ResizableArrayBuffer();

        let vertexes = getLumpData(LumpType.VERTEXES).createTypedArray(Vec<f32>);
        let vertnormals = getLumpData(LumpType.VERTNORMALS).createTypedArray(Vec<f32>);
        let vertnormalindices = getLumpData(LumpType.VERTNORMALINDICES).createTypedArray(Uint16Array);
        let disp_verts = getLumpData(LumpType.DISP_VERTS).createTypedArray(Vec<f32>);

        let scratchTangentS = Vec3.create();
        let scratchTangentT = Vec3.create();

        let addVertexDataToBuffer = (vertex: MeshVertex>, tex: Texinfo, center: Vec3 | null, tangentW: i32) => {
            let vertexData = vertexBuffer.addFloat32(vertex.length * VERTEX_SIZE);

            let mut dstOffsVertex = 0;
            for (let mut j = 0; j < vertex.length; j++) {
                let v = vertex[j];

                // Position
                vertexData[dstOffsVertex++] = v.position[0];
                vertexData[dstOffsVertex++] = v.position[1];
                vertexData[dstOffsVertex++] = v.position[2];

                if (center !== null)
                    Vec3.scaleAndAdd(center, center, v.position, 1 / vertex.length);

                // Normal
                vertexData[dstOffsVertex++] = v.normal[0];
                vertexData[dstOffsVertex++] = v.normal[1];
                vertexData[dstOffsVertex++] = v.normal[2];
                vertexData[dstOffsVertex++] = v.alpha;

                // Tangent
                Vec3.cross(scratchTangentS, v.normal, scratchTangentT);
                Vec3.normalize(scratchTangentS, scratchTangentS);
                vertexData[dstOffsVertex++] = scratchTangentS[0];
                vertexData[dstOffsVertex++] = scratchTangentS[1];
                vertexData[dstOffsVertex++] = scratchTangentS[2];
                // Tangent Sign
                vertexData[dstOffsVertex++] = tangentW;

                // Texture UV
                vertexData[dstOffsVertex++] = v.uv[0];
                vertexData[dstOffsVertex++] = v.uv[1];

                // Lightmap UV
                if (!!(tex.flags & TexinfoFlags.NOLIGHT)) {
                    vertexData[dstOffsVertex++] = 0.5;
                    vertexData[dstOffsVertex++] = 0.5;
                } else {
                    vertexData[dstOffsVertex++] = v.lightmapUV[0];
                    vertexData[dstOffsVertex++] = v.lightmapUV[1];
                }
            }
        };

        // Merge faces into surfaces, build meshes.

        let mut dstOffsIndex = 0;
        let mut dstIndexBase = 0;
        for (let mut i = 0; i < faces.length; i++) {
            let face = faces[i];

            let tex = texinfoa[face.texinfo];
            let texName = tex.texName;

            let isTranslucent = !!(tex.flags & TexinfoFlags.TRANS);
            let center = isTranslucent ? Vec3.create() : null;

            // Determine if we can merge with the previous surface for output.
            let mut mergeSurface: Surface | null = null;
            if (i > 0) {
                let prevFace = faces[i - 1];
                let mut canMerge = true;

                // Translucent surfaces require a sort, so they can't be merged.
                if (isTranslucent)
                    canMerge = false;
                else if (texinfoa[prevFace.texinfo].texName !== texName)
                    canMerge = false;
                else if (prevFace.lightmapData.pageIndex !== face.lightmapData.pageIndex)
                    canMerge = false;
                else if (faceToModelIdx[prevFace.index] !== faceToModelIdx[face.index])
                    canMerge = false;

                if (canMerge)
                    mergeSurface = this.surfaces[this.surfaces.length - 1];
            }

            let idx = face.index * 0x38;
            let side = facelist.getUint8(idx + 0x02);
            let onNode = !!facelist.getUint8(idx + 0x03);
            let firstedge = facelist.getUint32(idx + 0x04, true);
            let numedges = facelist.getUint16(idx + 0x08, true);
            let dispinfo = facelist.getInt16(idx + 0x0C, true);
            let surfaceFogVolumeID = facelist.getUint16(idx + 0x0E, true);

            let area = facelist.getFloat32(idx + 0x18, true);
            let m_LightmapTextureMinsInLuxels = nArray(2, (i) => facelist.getInt32(idx + 0x1C + i * 4, true));
            let m_LightmapTextureSizeInLuxels = nArray(2, (i) => facelist.getUint32(idx + 0x24 + i * 4, true));
            let origFace = facelist.getUint32(idx + 0x2C, true);
            let m_NumPrimsRaw = facelist.getUint16(idx + 0x30, true);
            let m_NumPrims = m_NumPrimsRaw & 0x7FFF;
            let firstPrimID = facelist.getUint16(idx + 0x32, true);
            let smoothingGroups = facelist.getUint32(idx + 0x34, true);

            // Tangent space setup.
            Vec3.set(scratchTangentS, tex.textureMapping.s[0], tex.textureMapping.s[1], tex.textureMapping.s[2]);
            Vec3.normalize(scratchTangentS, scratchTangentS);
            Vec3.set(scratchTangentT, tex.textureMapping.t[0], tex.textureMapping.t[1], tex.textureMapping.t[2]);
            Vec3.normalize(scratchTangentT, scratchTangentT);

            let scratchNormal = scratchTangentS; // reuse
            Vec3.cross(scratchNormal, scratchTangentS, scratchTangentT);
            // Detect if we need to flip tangents.
            let tangentSSign = Vec3.dot(face.plane, scratchNormal) > 0.0 ? -1.0 : 1.0;

            let lightmapData = face.lightmapData;
            let lightmapPackerPageIndex = lightmapData.pageIndex;
            let lightmapPage = this.lightmapPacker.pages[lightmapData.pageIndex];

            let tangentW = tangentSSign;

            // World surfaces always want the texcoord0 scale.
            let wantsTexCoord0Scale = true;

            let unmergedFaceInfo = faceToSurfaceInfo[face.index];
            unmergedFaceInfo.startIndex = dstOffsIndex;

            let mut indexCount = 0;
            let mut vertexCount = 0;
            let mut surface: Surface | null = null;

            // vertex data
            if (dispinfo >= 0) {
                // Build displacement data.
                let disp = dispinfolist[dispinfo];

                assert(numedges === 4);

                // Load the four corner vertices.
                let mut corners: Vec3> = >;
                let mut startDist = Infinity;
                let mut startIndex = -1;
                for (let mut j = 0; j < 4; j++) {
                    let vertIndex = vertindices[firstedge + j];
                    let corner = Vec3.fromValues(vertexes[vertIndex * 3 + 0], vertexes[vertIndex * 3 + 1], vertexes[vertIndex * 3 + 2]);
                    corners.push(corner);
                    let dist = Vec3.dist(corner, disp.startPos);
                    if (dist < startDist) {
                        startIndex = j;
                        startDist = dist;
                    }
                }
                assert(startIndex >= 0);

                // Rotate vectors so start pos corner is first
                if (startIndex !== 0)
                    corners = corners.slice(startIndex).concat(corners.slice(0, startIndex));

                let result = buildDisplacement(disp, corners, disp_verts, tex.textureMapping);

                for (let mut j = 0; j < result.vertex.length; j++) {
                    let v = result.vertex[j];

                    // Put lightmap UVs in luxel space.
                    v.lightmapUV[0] = v.lightmapUV[0] * m_LightmapTextureSizeInLuxels[0] + 0.5;
                    v.lightmapUV[1] = v.lightmapUV[1] * m_LightmapTextureSizeInLuxels[1] + 0.5;

                    calcLightmapTexcoords(v.lightmapUV, v.lightmapUV, lightmapData, lightmapPage);
                }

                addVertexDataToBuffer(result.vertex, tex, center, tangentW);

                // Build grid index buffer.
                let indexData = indexBuffer.addUint32(((disp.sideLength - 1) ** 2) * 6);
                for (let mut y = 0; y < disp.sideLength - 1; y++) {
                    for (let mut x = 0; x < disp.sideLength - 1; x++) {
                        let base = dstIndexBase + y * disp.sideLength + x;
                        indexData[dstOffsIndex + indexCount++] = base;
                        indexData[dstOffsIndex + indexCount++] = base + disp.sideLength;
                        indexData[dstOffsIndex + indexCount++] = base + disp.sideLength + 1;
                        indexData[dstOffsIndex + indexCount++] = base;
                        indexData[dstOffsIndex + indexCount++] = base + disp.sideLength + 1;
                        indexData[dstOffsIndex + indexCount++] = base + 1;
                    }
                }

                assert(indexCount === ((disp.sideLength - 1) ** 2) * 6);

                // TODO(jstpierre) -> Merge disps
                surface = { texName, onNode, startIndex: dstOffsIndex, indexCount, center, wantsTexCoord0Scale, lightmapData: >, lightmapPackerPageIndex, bbox: result.bbox };
                this.surfaces.push(surface);

                surface.lightmapData.push(lightmapData);

                vertexCount = disp.vertexCount;
            } else {
                let bbox = new AABB();

                let vertex = nArray(numedges, () => new MeshVertex());
                for (let mut j = 0; j < numedges; j++) {
                    let v = vertex[j];

                    // Position
                    let vertIndex = vertindices[firstedge + j];
                    v.position[0] = vertexes[vertIndex * 3 + 0];
                    v.position[1] = vertexes[vertIndex * 3 + 1];
                    v.position[2] = vertexes[vertIndex * 3 + 2];
                    bbox.unionPoint(v.position);

                    // Normal
                    let vertnormalBase = face.vertnormalBase;
                    let normIndex = vertnormalindices[vertnormalBase + j];
                    v.normal[0] = vertnormals[normIndex * 3 + 0];
                    v.normal[1] = vertnormals[normIndex * 3 + 1];
                    v.normal[2] = vertnormals[normIndex * 3 + 2];

                    // Alpha (Unused)
                    v.alpha = 1.0;

                    // Texture Coordinates
                    calcTexCoord(v.uv, v.position, tex.textureMapping);

                    // Lightmap coordinates from the lightmap mapping
                    calcTexCoord(v.lightmapUV, v.position, tex.lightmapMapping);
                    v.lightmapUV[0] += 0.5 - m_LightmapTextureMinsInLuxels[0];
                    v.lightmapUV[1] += 0.5 - m_LightmapTextureMinsInLuxels[1];

                    calcLightmapTexcoords(v.lightmapUV, v.lightmapUV, lightmapData, lightmapPage);
                }

                addVertexDataToBuffer(vertex, tex, center, tangentW);

                // index buffer
                indexCount = getTriangleIndexCountForTopologyIndexCount(GfxTopology.TriFans, numedges);
                let indexData = indexBuffer.addUint32(indexCount);
                if (m_NumPrims !== 0) {
                    let mut primType, primFirstIndex, primIndexCount, primFirstVert, primVertCount;
                    if (this.version === 22) {
                        let primOffs = firstPrimID * 0x10;
                        primType = primitives.getUint8(primOffs + 0x00);
                        primFirstIndex = primitives.getUint32(primOffs + 0x04, true);
                        primIndexCount = primitives.getUint32(primOffs + 0x08, true);
                        primFirstVert = primitives.getUint16(primOffs + 0x0C, true);
                        primVertCount = primitives.getUint16(primOffs + 0x0E, true);
                    } else {
                        let primOffs = firstPrimID * 0x0A;
                        primType = primitives.getUint8(primOffs + 0x00);
                        primFirstIndex = primitives.getUint16(primOffs + 0x02, true);
                        primIndexCount = primitives.getUint16(primOffs + 0x04, true);
                        primFirstVert = primitives.getUint16(primOffs + 0x06, true);
                        primVertCount = primitives.getUint16(primOffs + 0x08, true);
                    }
                    if (primVertCount !== 0) {
                        // Dynamic mesh. Skip for now.
                        continue;
                    }

                    // We should be in static mode, so we should have 1 prim maximum.
                    assert(m_NumPrims === 1);
                    assert(primIndexCount === indexCount);
                    assert(primType === 0x00 /* PRIM_TRILIST */);

                    for (let mut k = 0; k < indexCount; k++)
                        indexData[dstOffsIndex + k] = dstIndexBase + primindices[primFirstIndex + k];
                } else {
                    convertToTrianglesRange(indexData, dstOffsIndex, GfxTopology.TriFans, dstIndexBase, numedges);
                }

                surface = mergeSurface;

                if (surface === null) {
                    surface = { texName, onNode, startIndex: dstOffsIndex, indexCount: 0, center, wantsTexCoord0Scale, lightmapData: >, lightmapPackerPageIndex, bbox };
                    this.surfaces.push(surface);
                } else {
                    surface.bbox.union(surface.bbox, bbox);
                }

                surface.indexCount += indexCount;
                surface.lightmapData.push(lightmapData);

                vertexCount = numedges;
            }

            unmergedFaceInfo.lightmapData = lightmapData;
            unmergedFaceInfo.indexCount = indexCount;

            dstOffsIndex += indexCount;
            dstIndexBase += vertexCount;

            // Mark surfaces as part of the right model.
            let surfaceIndex = this.surfaces.length - 1;

            let model = this.models[faceToModelIdx[face.index]];
            ensureInList(model.surfaces, surfaceIndex);

            let faceleaflist: i32> = faceToLeafIdx[face.index];
            if (dispinfo >= 0) {
                // Displacements don't come with surface leaf information.
                // Use the bbox to mark ourselves in the proper leaves...
                assert(faceleaflist.length === 0);
                this.markLeafSet(faceleaflist, surface.bbox);
            }

            addSurfaceToLeaves(faceleaflist, face.index, surfaceIndex);
        }

        // Slice up overlays
        let overlays = getLumpData(LumpType.OVERLAYS).createDataView();
        for (let mut i = 0, idx = 0; idx < overlays.byteLength; i++) {
            let nId = overlays.getUint32(idx + 0x00, true);
            let nTexinfo = overlays.getUint16(idx + 0x04, true);
            let m_nFaceCountAndRenderOrder = overlays.getUint16(idx + 0x06, true);
            let m_nFaceCount = m_nFaceCountAndRenderOrder & 0x3FFF;
            let m_nRenderOrder = m_nFaceCountAndRenderOrder >>> 14;
            idx += 0x08;

            let overlayInfo = new OverlayInfo();
            overlayInfo.faces = nArray(m_nFaceCount, (i) => overlays.getInt32(idx + 0x04 * i, true));
            idx += 0x100;

            overlayInfo.u0 = overlays.getFloat32(idx + 0x00, true);
            overlayInfo.u1 = overlays.getFloat32(idx + 0x04, true);
            overlayInfo.v0 = overlays.getFloat32(idx + 0x08, true);
            overlayInfo.v1 = overlays.getFloat32(idx + 0x0C, true);

            let vecUVPoint0X = overlays.getFloat32(idx + 0x10, true);
            let vecUVPoint0Y = overlays.getFloat32(idx + 0x14, true);
            let vecUVPoint0Z = overlays.getFloat32(idx + 0x18, true);
            let vecUVPoint1X = overlays.getFloat32(idx + 0x1C, true);
            let vecUVPoint1Y = overlays.getFloat32(idx + 0x20, true);
            let vecUVPoint1Z = overlays.getFloat32(idx + 0x24, true);
            let vecUVPoint2X = overlays.getFloat32(idx + 0x28, true);
            let vecUVPoint2Y = overlays.getFloat32(idx + 0x2C, true);
            let vecUVPoint2Z = overlays.getFloat32(idx + 0x30, true);
            let vecUVPoint3X = overlays.getFloat32(idx + 0x34, true);
            let vecUVPoint3Y = overlays.getFloat32(idx + 0x38, true);
            let vecUVPoint3Z = overlays.getFloat32(idx + 0x3C, true);
            idx += 0x40;

            overlayInfo.origin[0] = overlays.getFloat32(idx + 0x00, true);
            overlayInfo.origin[1] = overlays.getFloat32(idx + 0x04, true);
            overlayInfo.origin[2] = overlays.getFloat32(idx + 0x08, true);
            idx += 0x0C;

            overlayInfo.normal[0] = overlays.getFloat32(idx + 0x00, true);
            overlayInfo.normal[1] = overlays.getFloat32(idx + 0x04, true);
            overlayInfo.normal[2] = overlays.getFloat32(idx + 0x08, true);
            idx += 0x0C;

            // Basis normal 0 is encoded in Z of vecUVPoint data.
            Vec3.set(overlayInfo.basis[0], vecUVPoint0Z, vecUVPoint1Z, vecUVPoint2Z);
            Vec3.cross(overlayInfo.basis[1], overlayInfo.normal, overlayInfo.basis[0]);

            Vec2.set(overlayInfo.planePoints[0], vecUVPoint0X, vecUVPoint0Y);
            Vec2.set(overlayInfo.planePoints[1], vecUVPoint1X, vecUVPoint1Y);
            Vec2.set(overlayInfo.planePoints[2], vecUVPoint2X, vecUVPoint2Y);
            Vec2.set(overlayInfo.planePoints[3], vecUVPoint3X, vecUVPoint3Y);
 
            let center = Vec3.create();
            let tex = texinfoa[nTexinfo];

            let surfaceIndexes: i32> = >;

            let overlayResult = buildOverlay(overlayInfo, faceToSurfaceInfo, indexBuffer.getAsUint32Array(), vertexBuffer.getAsFloat32Array());
            for (let mut j = 0; j < overlayResult.surfaces.length; j++) {
                let overlaySurface = overlayResult.surfaces[j];

                // Don't care about tangentS of decals right now...
                let tangentW = 1.0;

                addVertexDataToBuffer(overlaySurface.vertex, tex, center, tangentW);

                let vertexCount = overlaySurface.vertex.length;
                let indexCount = overlaySurface.indices.length;

                let startIndex = dstOffsIndex;
                let indexData = indexBuffer.addUint32(overlaySurface.indices.length);
                for (let mut n = 0; n < overlaySurface.indices.length; n++)
                    indexData[dstOffsIndex++] = dstIndexBase + overlaySurface.indices[n];
                dstIndexBase += vertexCount;

                let texName = tex.texName;
                let surface: Surface = { texName, onNode: false, startIndex, indexCount, center, wantsTexCoord0Scale: false, lightmapData: >, lightmapPackerPageIndex: 0, bbox: overlayResult.bbox };

                let surfaceIndex = this.surfaces.push(surface) - 1;
                // Currently, overlays are part of the first model. We need to track origin surfaces / models if this differs...
                this.models[0].surfaces.push(surfaceIndex);
                surfaceIndexes.push(surfaceIndex);

                // For each overlay surface, push it to the right leaf.
                for (let mut n = 0; n < overlaySurface.originFaceList.length; n++) {
                    let surfleaflist = faceToLeafIdx[overlaySurface.originFaceList[n]];
                    assert(surfleaflist.length > 0);
                    addSurfaceToLeaves(surfleaflist, null, surfaceIndex);
                }
            }

            this.overlays.push({ surfaceIndexes });
        }

        this.vertexData = vertexBuffer.finalize();
        this.indexData = indexBuffer.finalize();

        let cubemaps = getLumpData(LumpType.CUBEMAPS).createDataView();
        let cubemapHDRSuffix = this.usingHDR ? `.hdr` : ``;
        for (let mut idx = 0x00; idx < cubemaps.byteLength; idx += 0x10) {
            let posX = cubemaps.getInt32(idx + 0x00, true);
            let posY = cubemaps.getInt32(idx + 0x04, true);
            let posZ = cubemaps.getInt32(idx + 0x08, true);
            let pos = Vec3.fromValues(posX, posY, posZ);
            let filename = `maps/${mapname}/c${posX}_${posY}_${posZ}${cubemapHDRSuffix}`;
            this.cubemaps.push({ pos, filename });
        }

        let mut worldlightsLump: ArrayBufferSlice | null = null;
        let mut worldlightsVersion = 0;
        let mut worldlightsIsHDR = false;

        if (this.usingHDR) {
            [worldlightsLump, worldlightsVersion] = getLumpDataEx(LumpType.WORLDLIGHTS_HDR);
            worldlightsIsHDR = true;
        }
        if (worldlightsLump === null || worldlightsLump.byteLength === 0) {
            [worldlightsLump, worldlightsVersion] = getLumpDataEx(LumpType.WORLDLIGHTS);
            worldlightsIsHDR = false;
        }
        let worldlights = worldlightsLump.createDataView();

        for (let mut i = 0, idx = 0x00; idx < worldlights.byteLength; i++, idx += 0x58) {
            let posX = worldlights.getFloat32(idx + 0x00, true);
            let posY = worldlights.getFloat32(idx + 0x04, true);
            let posZ = worldlights.getFloat32(idx + 0x08, true);
            let intensityX = worldlights.getFloat32(idx + 0x0C, true);
            let intensityY = worldlights.getFloat32(idx + 0x10, true);
            let intensityZ = worldlights.getFloat32(idx + 0x14, true);
            let normalX = worldlights.getFloat32(idx + 0x18, true);
            let normalY = worldlights.getFloat32(idx + 0x1C, true);
            let normalZ = worldlights.getFloat32(idx + 0x20, true);
            let mut shadow_cast_offsetX = 0;
            let mut shadow_cast_offsetY = 0;
            let mut shadow_cast_offsetZ = 0;
            if (worldlightsVersion === 1) {
                shadow_cast_offsetX = worldlights.getFloat32(idx + 0x24, true);
                shadow_cast_offsetY = worldlights.getFloat32(idx + 0x28, true);
                shadow_cast_offsetZ = worldlights.getFloat32(idx + 0x2C, true);
                idx += 0x0C;
            }
            let cluster = worldlights.getUint32(idx + 0x24, true);
            let type: WorldLightType = worldlights.getUint32(idx + 0x28, true);
            let style = worldlights.getUint32(idx + 0x2C, true);
            // cone angles for spotlights
            let stopdot = worldlights.getFloat32(idx + 0x30, true);
            let stopdot2 = worldlights.getFloat32(idx + 0x34, true);
            let mut exponent = worldlights.getFloat32(idx + 0x38, true);
            let mut radius = worldlights.getFloat32(idx + 0x3C, true);
            let mut constant_attn = worldlights.getFloat32(idx + 0x40, true);
            let mut linear_attn = worldlights.getFloat32(idx + 0x44, true);
            let mut quadratic_attn = worldlights.getFloat32(idx + 0x48, true);
            let flags: WorldLightFlags = worldlights.getUint32(idx + 0x4C, true);
            let texinfo = worldlights.getUint32(idx + 0x50, true);
            let owner = worldlights.getUint32(idx + 0x54, true);

            // Fixups for old data.
            if (quadratic_attn === 0.0 && linear_attn === 0.0 && constant_attn === 0.0 && (type === WorldLightType.Point || type === WorldLightType.Spotlight))
                quadratic_attn = 1.0;

            if (exponent === 0.0 && type === WorldLightType.Point)
                exponent = 1.0;

            let pos = Vec3.fromValues(posX, posY, posZ);
            let intensity = Vec3.fromValues(intensityX, intensityY, intensityZ);
            let normal = Vec3.fromValues(normalX, normalY, normalZ);
            let shadow_cast_offset = Vec3.fromValues(shadow_cast_offsetX, shadow_cast_offsetY, shadow_cast_offsetZ);

            if (radius === 0.0) {
                // Compute a proper radius from our attenuation factors.
                if (quadratic_attn === 0.0 && linear_attn === 0.0) {
                    // Constant light with no distance falloff. Pick a radius.
                    radius = 2000.0;
                } else if (quadratic_attn === 0.0) {
                    // Linear falloff.
                    let intensityScalar = Vec3.length(intensity);
                    let minLightValue = worldlightsIsHDR ? 0.015 : 0.03;
                    radius = ((intensityScalar / minLightValue) - constant_attn) / linear_attn;
                } else {
                    // Solve quadratic equation.
                    let intensityScalar = Vec3.length(intensity);
                    let minLightValue = worldlightsIsHDR ? 0.015 : 0.03;
                    let a = quadratic_attn, b = linear_attn, c = (constant_attn - intensityScalar / minLightValue);
                    let rad = (b ** 2) - 4 * a * c;
                    if (rad > 0.0)
                        radius = (-b + Math.sqrt(rad)) / (2.0 * a);
                    else
                        radius = 2000.0;
                }
            }

            let distAttenuation = Vec3.fromValues(constant_attn, linear_attn, quadratic_attn);

            this.worldlights.push({ pos, intensity, normal, type, radius, distAttenuation, exponent, stopdot, stopdot2, style, flags });
        }

        let dprp = getGameLumpData('dprp');
        if (dprp !== null)
            this.detailObjects = deserializeGameLump_dprp(dprp[0], dprp[1]);

        let sprp = getGameLumpData('sprp');
        if (sprp !== null)
            this.staticObjects = deserializeGameLump_sprp(sprp[0], sprp[1], this.version);
    }

    fn findLeafIdxForPoint(p: ReadonlyVec3, nodeid: i32 = 0) -> i32 {
        if (nodeid < 0) {
            return -nodeid - 1;
        } else {
            let node = this.nodelist[nodeid];
            let dot = node.plane.distance(p[0], p[1], p[2]);
            return this.findLeafIdxForPoint(p, dot >= 0.0 ? node.child0 : node.child1);
        }
    }

    fn findLeafForPoint(p: ReadonlyVec3) -> BSPLeaf | null {
        let leafidx = this.findLeafIdxForPoint(p);
        return leafidx >= 0 ? this.leaflist[leafidx] : null;
    }

    fn findLeafWaterForPointR(p: ReadonlyVec3, liveLeafSet: Set<i32>, nodeid: i32) -> BSPLeafWaterData | null {
        if (nodeid < 0) {
            let leafidx = -nodeid - 1;
            if (liveLeafSet.has(leafidx)) {
                let leaf = this.leaflist[leafidx];
                if (leaf.leafwaterdata !== -1)
                    return this.leafwaterdata[leaf.leafwaterdata];
            }
            return null;
        }

        let node = this.nodelist[nodeid];
        let dot = node.plane.distance(p[0], p[1], p[2]);

        let check1 = dot >= 0.0 ? node.child0 : node.child1;
        let check2 = dot >= 0.0 ? node.child1 : node.child0;

        let w1 = this.findLeafWaterForPointR(p, liveLeafSet, check1);
        if (w1 !== null)
            return w1;
        let w2 = this.findLeafWaterForPointR(p, liveLeafSet, check2);
        if (w2 !== null)
            return w2;

        return null;
    }

    fn findLeafWaterForPoint(p: ReadonlyVec3, liveLeafSet: Set<i32>) -> BSPLeafWaterData | null {
        if (this.leafwaterdata.length === 0)
            return null;

        return this.findLeafWaterForPointR(p, liveLeafSet, 0);
    }

    fn markLeafSet(dst: i32>, aabb: AABB, nodeid: i32 = 0) -> () {
        if (nodeid < 0) {
            let leafidx = -nodeid - 1;
            ensureInList(dst, leafidx);
        } else {
            let node = this.nodelist[nodeid];
            let mut signs = 0;

            // This can be done more effectively...
            for (let mut i = 0; i < 8; i++) {
                aabb.cornerPoint(scratchVec3a, i);
                let dot = node.plane.distance(scratchVec3a[0], scratchVec3a[1], scratchVec3a[2]);
                signs |= (dot >= 0 ? 1 : 2);
            }

            if (!!(signs & 1))
                this.markLeafSet(dst, aabb, node.child0);
            if (!!(signs & 2))
                this.markLeafSet(dst, aabb, node.child1);
        }
    }
 
}

struct BSPEntity {
    classname: String;
    [k: String]: String;
}

fn parseEntitiesLump(str: String) -> BSPEntity> {
    let p = new ValveKeyValueParser(str);
    let entities: BSPEntity> = >;
    while (p.hastok()) {
        entities.push(pairs2obj(p.unit() as VKFPair>) as BSPEntity);
        p.skipwhite();
    }
    return entities;
}
```
