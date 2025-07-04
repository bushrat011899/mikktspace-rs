/*!
 *  Copyright (C) 2011 by Morten S. Mikkelsen
 *
 *  This software is provided 'as-is', without any express or implied
 *  warranty.  In no event will the authors be held liable for any damages
 *  arising from the use of this software.
 *
 *  Permission is granted to anyone to use this software for any purpose,
 *  including commercial applications, and to alter it and redistribute it
 *  freely, subject to the following restrictions:
 *
 *  1. The origin of this software must not be misrepresented; you must not
 *     claim that you wrote the original software. If you use this software
 *     in a product, an acknowledgment in the product documentation would be
 *     appreciated but is not required.
 *  2. Altered source versions must be plainly marked as such, and must not be
 *     misrepresented as being the original software.
 *  3. This notice may not be removed or altered from any source distribution.
 */

#![expect(non_snake_case)]

use alloc::{vec, vec::Vec};
use core::{
    ffi::{c_double, c_float, c_int, c_uchar, c_uint, c_ulong},
    ops::Index,
};

use crate::{math::*, MikkTSpaceInterface};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct STSpace {
    pub vOs: SVec3,
    pub fMagS: c_float,
    pub vOt: SVec3,
    pub fMagT: c_float,
    /// this is to average back into quads.
    pub iCounter: c_int,
    pub bOrient: bool,
}

impl STSpace {
    pub const ZERO: STSpace = STSpace {
        vOs: SVec3::ZERO,
        fMagS: 0.,
        vOt: SVec3::ZERO,
        fMagT: 0.,
        iCounter: 0,
        bOrient: false,
    };
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct STriInfo {
    pub FaceNeighbors: [c_int; 3],
    pub AssignedGroup: [Option<usize>; 3],

    /// normalized first order face derivative
    pub vOs: SVec3,
    /// normalized first order face derivative
    pub vOt: SVec3,

    /// original magnitude of vOs
    pub fMagS: c_float,
    /// original magnitude of vOs
    pub fMagT: c_float,

    /// determines if the current and the next triangle are a quad.
    pub iOrgFaceNumber: c_int,

    pub iFlag: c_int,
    pub iTSpacesOffs: c_int,
    pub vert_num: [c_uchar; 4],
}

impl STriInfo {
    pub const ZERO: STriInfo = STriInfo {
        FaceNeighbors: [0; 3],
        AssignedGroup: [None; 3],
        vOs: SVec3::ZERO,
        vOt: SVec3::ZERO,
        fMagS: 0.,
        fMagT: 0.,
        iOrgFaceNumber: 0,
        iFlag: 0,
        iTSpacesOffs: 0,
        vert_num: [0; 4],
    };
}

#[derive(Clone)]
#[repr(C)]
pub struct SGroup {
    pub id: usize,
    pub pFaceIndices: Vec<c_int>,
    pub iVertexRepresentitive: c_int,
    pub bOrientPreservering: bool,
}

impl SGroup {
    pub const ZERO: SGroup = SGroup {
        id: 0,
        pFaceIndices: Vec::new(),
        iVertexRepresentitive: 0,
        bOrientPreservering: false,
    };
}

#[derive(Clone, PartialEq)]
#[repr(C)]
pub struct SSubGroup {
    pub pTriMembers: Vec<c_int>,
}

impl SSubGroup {
    pub const ZERO: SSubGroup = SSubGroup {
        pTriMembers: Vec::new(),
    };
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SEdge {
    pub i0: c_int,
    pub i1: c_int,
    pub f: c_int,
}

impl SEdge {
    pub const ZERO: SEdge = SEdge { i0: 0, i1: 0, f: 0 };
}

impl Index<usize> for SEdge {
    type Output = c_int;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.i0,
            1 => &self.i1,
            2 => &self.f,
            _ => panic!(),
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct STmpVert {
    pub vert: SVec3,
    pub index: c_int,
}

impl STmpVert {
    pub const ZERO: STmpVert = STmpVert {
        vert: SVec3::ZERO,
        index: 0,
    };
}

pub const INTERNAL_RND_SORT_SEED: c_int = 39871946 as c_int;
pub const MARK_DEGENERATE: c_int = 1 as c_int;
pub const QUAD_ONE_DEGEN_TRI: c_int = 2 as c_int;
pub const GROUP_WITH_ANY: c_int = 4 as c_int;
pub const ORIENT_PRESERVING: c_int = 8 as c_int;

fn MakeIndex(iFace: c_int, iVert: c_int) -> c_int {
    assert!(iVert >= 0 as c_int && iVert < 4 as c_int && iFace >= 0 as c_int);
    iFace << 2 as c_int | iVert & 0x3 as c_int
}

fn IndexToData(iIndexIn: c_int) -> (c_int, c_int) {
    (iIndexIn >> 2 as c_int, iIndexIn & 0x3 as c_int)
}

fn AvgTSpace(pTS0: STSpace, pTS1: STSpace) -> STSpace {
    let mut ts_res: STSpace = STSpace {
        vOs: SVec3::ZERO,
        fMagS: 0.,
        vOt: SVec3::ZERO,
        fMagT: 0.,
        iCounter: 0,
        bOrient: false,
    };

    // this if is important. Due to floating point precision
    // averaging when ts0==ts1 will cause a slight difference
    // which results in tangent space splits later on
    if pTS0.fMagS == pTS1.fMagS
        && pTS0.fMagT == pTS1.fMagT
        && (pTS0.vOs == pTS1.vOs)
        && (pTS0.vOt == pTS1.vOt)
    {
        ts_res.fMagS = pTS0.fMagS;
        ts_res.fMagT = pTS0.fMagT;
        ts_res.vOs = pTS0.vOs;
        ts_res.vOt = pTS0.vOt;
    } else {
        ts_res.fMagS = 0.5f32 * (pTS0.fMagS + pTS1.fMagS);
        ts_res.fMagT = 0.5f32 * (pTS0.fMagT + pTS1.fMagT);
        ts_res.vOs = pTS0.vOs + pTS1.vOs;
        ts_res.vOt = pTS0.vOt + pTS1.vOt;
        ts_res.vOs.normalize_or_zero();
        ts_res.vOt.normalize_or_zero();
    }
    ts_res
}

pub fn genTangSpaceDefault<I: MikkTSpaceInterface>(pContext: &mut I) -> bool {
    genTangSpace(pContext, 180.0f32)
}

pub fn genTangSpace<I: MikkTSpaceInterface>(pContext: &mut I, fAngularThreshold: c_float) -> bool {
    let mut iNrTrianglesIn: c_int = 0 as c_int;
    let fThresCos: c_float = cos(deg_to_rad(fAngularThreshold) as c_double) as c_float;

    // count triangles on supported faces
    let iNrFaces: c_int = pContext.get_num_faces() as c_int;
    let mut f = 0 as c_int;
    while f < iNrFaces {
        let verts: c_int = pContext.get_num_vertices_of_face(f as usize) as c_int;
        if verts == 3 as c_int {
            iNrTrianglesIn += 1;
        } else if verts == 4 as c_int {
            iNrTrianglesIn += 2 as c_int;
        }
        f += 1;
    }
    if iNrTrianglesIn <= 0 as c_int {
        return false;
    }

    // allocate memory for an index list
    let mut piTriListIn: Vec<c_int> = vec![0; (iNrTrianglesIn as c_ulong).wrapping_mul(3) as usize];
    let mut pTriInfos: Vec<STriInfo> = vec![STriInfo::ZERO; iNrTrianglesIn as usize];

    // make an initial triangle --> face index list
    let iNrTSPaces = GenerateInitialVerticesIndexList(
        &mut pTriInfos,
        &mut piTriListIn,
        pContext,
        iNrTrianglesIn,
    );

    // make a welded index list of identical positions and attributes (pos, norm, texc)
    GenerateSharedVerticesIndexList(&mut piTriListIn, pContext, iNrTrianglesIn);

    // Mark all degenerate triangles
    let iTotTris = iNrTrianglesIn;
    let mut iDegenTriangles = 0 as c_int;
    let mut t = 0 as c_int;
    while t < iTotTris {
        let i0: c_int = piTriListIn[(t * 3 as c_int + 0 as c_int) as usize];
        let i1: c_int = piTriListIn[(t * 3 as c_int + 1 as c_int) as usize];
        let i2: c_int = piTriListIn[(t * 3 as c_int + 2 as c_int) as usize];
        let p0: SVec3 = GetPosition(pContext, i0);
        let p1: SVec3 = GetPosition(pContext, i1);
        let p2: SVec3 = GetPosition(pContext, i2);
        if (p0 == p1) || (p0 == p2) || (p1 == p2) {
            // degenerate
            pTriInfos[t as usize].iFlag |= MARK_DEGENERATE;
            iDegenTriangles += 1;
        }
        t += 1;
    }
    iNrTrianglesIn = iTotTris - iDegenTriangles;

    // mark all triangle pairs that belong to a quad with only one
    // good triangle. These need special treatment in DegenEpilogue().
    // Additionally, move all good triangles to the start of
    // pTriInfos[] and piTriListIn[] without changing order and
    // put the degenerate triangles last.
    DegenPrologue(&mut pTriInfos, &mut piTriListIn, iNrTrianglesIn, iTotTris);

    // evaluate triangle level attributes and neighbor list
    InitTriInfo(&mut pTriInfos, &piTriListIn, pContext, iNrTrianglesIn);

    // based on the 4 rules, identify groups based on connectivity
    let iNrMaxGroups = iNrTrianglesIn * 3 as c_int;
    let mut pGroups: Vec<SGroup> = vec![SGroup::ZERO; iNrMaxGroups as usize];
    let iNrActiveGroups =
        Build4RuleGroups(&mut pTriInfos, &mut pGroups, &piTriListIn, iNrTrianglesIn);

    let mut psTspace: Vec<STSpace> = vec![STSpace::ZERO; iNrTSPaces as usize];
    t = 0 as c_int;
    while t < iNrTSPaces {
        psTspace[t as usize] = STSpace {
            vOs: SVec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            fMagS: 1.0,
            vOt: SVec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            fMagT: 1.0,
            ..STSpace::ZERO
        };
        t += 1;
    }

    // make tspaces, each group is split up into subgroups if necessary
    // based on fAngularThreshold. Finally a tangent space is made for
    // every resulting subgroup
    let bRes = GenerateTSpaces(
        &mut psTspace,
        &pTriInfos,
        &pGroups,
        iNrActiveGroups,
        &piTriListIn,
        fThresCos,
        pContext,
    );
    // if an allocation in GenerateTSpaces() failed
    if !bRes {
        return false;
    }

    // degenerate quads with one good triangle will be fixed by copying a space from
    // the good triangle to the coinciding vertex.
    // all other degenerate triangles will just copy a space from any good triangle
    // with the same welded index in piTriListIn[].
    DegenEpilogue(
        &mut psTspace,
        &pTriInfos,
        &piTriListIn,
        pContext,
        iNrTrianglesIn,
        iTotTris,
    );
    let mut index = 0 as c_int;
    f = 0 as c_int;
    while f < iNrFaces {
        let verts_0: c_int = pContext.get_num_vertices_of_face(f as usize) as c_int;
        if !(verts_0 != 3 as c_int && verts_0 != 4 as c_int) {
            // I've decided to let degenerate triangles and group-with-anythings
            // vary between left/right hand coordinate systems at the vertices.
            // All healthy triangles on the other hand are built to always be either or.

            /*// force the coordinate system orientation to be uniform for every face.
            // (this is already the case for good triangles but not for
            // degenerate ones and those with bGroupWithAnything==true)
            bool bOrient = psTspace[index].bOrient;
            if (psTspace[index].iCounter == 0)	// tspace was not derived from a group
            {
                // look for a space created in GenerateTSpaces() by iCounter>0
                bool bNotFound = true;
                int i=1;
                while (i<verts && bNotFound)
                {
                    if (psTspace[index+i].iCounter > 0) bNotFound=false;
                    else ++i;
                }
                if (!bNotFound) bOrient = psTspace[index+i].bOrient;
            }*/

            // set data
            let mut i = 0 as c_int;
            while i < verts_0 {
                let pTSpace = &psTspace[index as usize];
                let tang: [c_float; 3] = [pTSpace.vOs.x, pTSpace.vOs.y, pTSpace.vOs.z];
                let bitang: [c_float; 3] = [pTSpace.vOt.x, pTSpace.vOt.y, pTSpace.vOt.z];

                pContext.set_tspace(
                    tang,
                    bitang,
                    pTSpace.fMagS,
                    pTSpace.fMagT,
                    pTSpace.bOrient,
                    f as usize,
                    i as usize,
                );
                index += 1;
                i += 1;
            }
        }
        f += 1;
    }
    true
}

const CELLS: c_int = 2048 as c_int;

// it is IMPORTANT that this function is called to evaluate the hash since
// inlining could potentially reorder instructions and generate different
// results for the same effective input value fVal.
#[inline(never)]
fn FindGridCell(fMin: c_float, fMax: c_float, fVal: c_float) -> c_int {
    let fIndex: c_float = CELLS as c_float * ((fVal - fMin) / (fMax - fMin));
    let iIndex: c_int = fIndex as c_int;
    if iIndex < CELLS {
        if iIndex >= 0 as c_int {
            iIndex
        } else {
            0 as c_int
        }
    } else {
        CELLS - 1 as c_int
    }
}

fn GenerateSharedVerticesIndexList<I: MikkTSpaceInterface>(
    piTriList_in_and_out: &mut [c_int],
    pContext: &I,
    iNrTrianglesIn: c_int,
) {
    // Generate bounding box
    let mut vMin: SVec3 = GetPosition(pContext, 0 as c_int);
    let mut vMax: SVec3 = vMin;
    let mut i = 1 as c_int;
    while i < iNrTrianglesIn * 3 as c_int {
        let index: c_int = piTriList_in_and_out[i as usize];
        let vP: SVec3 = GetPosition(pContext, index);
        if vMin.x > vP.x {
            vMin.x = vP.x;
        } else if vMax.x < vP.x {
            vMax.x = vP.x;
        }
        if vMin.y > vP.y {
            vMin.y = vP.y;
        } else if vMax.y < vP.y {
            vMax.y = vP.y;
        }
        if vMin.z > vP.z {
            vMin.z = vP.z;
        } else if vMax.z < vP.z {
            vMax.z = vP.z;
        }
        i += 1;
    }
    let vDim = vMax - vMin;
    let mut iChannel = 0 as c_int;
    let mut fMin = vMin.x;
    let mut fMax = vMax.x;
    if vDim.y > vDim.x && vDim.y > vDim.z {
        iChannel = 1 as c_int;
        fMin = vMin.y;
        fMax = vMax.y;
    } else if vDim.z > vDim.x {
        iChannel = 2 as c_int;
        fMin = vMin.z;
        fMax = vMax.z;
    }

    // if /* can't allocate? */ {
    //     GenerateSharedVerticesIndexListSlow(piTriList_in_and_out, pContext, iNrTrianglesIn);
    //     return;
    // }

    // make allocations
    let mut piHashTable: Vec<c_int> = vec![0; (iNrTrianglesIn as c_ulong).wrapping_mul(3) as usize];
    let mut piHashCount: Vec<c_int> = vec![0; CELLS as usize];
    let mut piHashOffsets: Vec<c_int> = vec![0; CELLS as usize];
    let mut piHashCount2: Vec<c_int> = vec![0; CELLS as usize];

    // count amount of elements in each cell unit
    i = 0 as c_int;
    while i < iNrTrianglesIn * 3 as c_int {
        let index_0: c_int = piTriList_in_and_out[i as usize];
        let vP_0: SVec3 = GetPosition(pContext, index_0);
        let fVal: c_float = if iChannel == 0 as c_int {
            vP_0.x
        } else if iChannel == 1 as c_int {
            vP_0.y
        } else {
            vP_0.z
        };
        let iCell: c_int = FindGridCell(fMin, fMax, fVal);
        let fresh0 = &mut piHashCount[iCell as usize];
        *fresh0 += 1;
        i += 1;
    }

    // evaluate start index of each cell.
    piHashOffsets[0 as c_int as usize] = 0 as c_int;
    let mut k = 1 as c_int;
    while k < CELLS {
        piHashOffsets[k as usize] =
            piHashOffsets[(k - 1 as c_int) as usize] + piHashCount[(k - 1 as c_int) as usize];
        k += 1;
    }

    // insert vertices
    i = 0 as c_int;
    while i < iNrTrianglesIn * 3 as c_int {
        let index_1: c_int = piTriList_in_and_out[i as usize];
        let vP_1: SVec3 = GetPosition(pContext, index_1);
        let fVal_0: c_float = if iChannel == 0 as c_int {
            vP_1.x
        } else if iChannel == 1 as c_int {
            vP_1.y
        } else {
            vP_1.z
        };
        let iCell_0: c_int = FindGridCell(fMin, fMax, fVal_0);
        assert!(piHashCount2[iCell_0 as usize] < piHashCount[iCell_0 as usize]);
        let pTable = &mut piHashTable
            [piHashOffsets[iCell_0 as usize] as usize + piHashCount2[iCell_0 as usize] as usize];
        *pTable = i; // vertex i has been inserted.
        let fresh1 = &mut piHashCount2[iCell_0 as usize];
        *fresh1 += 1;
        i += 1;
    }

    // verify the count
    k = 0 as c_int;
    while k < CELLS {
        assert!(piHashCount2[k as usize] == piHashCount[k as usize]);
        k += 1;
    }

    // find maximum amount of entries in any hash entry
    let mut iMaxCount = piHashCount[0 as c_int as usize];
    k = 1 as c_int;
    while k < CELLS {
        if iMaxCount < piHashCount[k as usize] {
            iMaxCount = piHashCount[k as usize];
        }
        k += 1;
    }

    // complete the merge
    let mut pTmpVert: Vec<STmpVert> = vec![STmpVert::ZERO; iMaxCount as usize];
    k = 0 as c_int;
    while k < CELLS {
        let iEntries: c_int = piHashCount[k as usize];
        if iEntries >= 2 as c_int {
            // if /* couldn't allocate pTmpVert? */ {
            //     MergeVertsSlow(
            //         piTriList_in_and_out,
            //         pContext,
            //         pTable_0 as *const c_int,
            //         iEntries,
            //     );
            // }
            let mut e = 0 as c_int;
            while e < iEntries {
                let i_0: c_int = piHashTable[piHashOffsets[k as usize] as usize + e as usize];
                let vP_2: SVec3 = GetPosition(pContext, piTriList_in_and_out[i_0 as usize]);
                pTmpVert[e as usize].vert = vP_2;
                pTmpVert[e as usize].index = i_0;
                e += 1;
            }
            MergeVertsFast(
                piTriList_in_and_out,
                &mut pTmpVert,
                pContext,
                0 as c_int,
                iEntries - 1 as c_int,
            );
        }
        k += 1;
    }
}

fn MergeVertsFast<I: MikkTSpaceInterface>(
    piTriList_in_and_out: &mut [c_int],
    pTmpVert: &mut [STmpVert],
    pContext: &I,
    iL_in: c_int,
    iR_in: c_int,
) {
    // make bbox
    let mut fvMin: [c_float; 3] = [0.; 3];
    let mut fvMax: [c_float; 3] = [0.; 3];

    let mut c = 0 as c_int;
    while c < 3 as c_int {
        fvMin[c as usize] = pTmpVert[iL_in as usize].vert[c as usize];
        fvMax[c as usize] = fvMin[c as usize];
        c += 1;
    }
    let mut l = iL_in + 1 as c_int;
    while l <= iR_in {
        c = 0 as c_int;
        while c < 3 as c_int {
            if fvMin[c as usize] > pTmpVert[l as usize].vert[c as usize] {
                fvMin[c as usize] = pTmpVert[l as usize].vert[c as usize];
            }
            if fvMax[c as usize] < pTmpVert[l as usize].vert[c as usize] {
                fvMax[c as usize] = pTmpVert[l as usize].vert[c as usize];
            }
            c += 1;
        }
        l += 1;
    }

    let dx = fvMax[0 as c_int as usize] - fvMin[0 as c_int as usize];
    let dy = fvMax[1 as c_int as usize] - fvMin[1 as c_int as usize];
    let dz = fvMax[2 as c_int as usize] - fvMin[2 as c_int as usize];

    let mut channel = 0 as c_int;
    if dy > dx && dy > dz {
        channel = 1 as c_int;
    } else if dz > dx {
        channel = 2 as c_int;
    }

    let fSep = 0.5f32 * (fvMax[channel as usize] + fvMin[channel as usize]);

    // stop if all vertices are NaNs
    if fSep.is_finite() as i32 == 0 {
        return;
    }

    // terminate recursion when the separation/average value
    // is no longer strictly between fMin and fMax values.
    if fSep >= fvMax[channel as usize] || fSep <= fvMin[channel as usize] {
        // complete the weld
        l = iL_in;
        while l <= iR_in {
            let i: c_int = pTmpVert[l as usize].index;
            let index: c_int = piTriList_in_and_out[i as usize];
            let vP: SVec3 = GetPosition(pContext, index);
            let vN: SVec3 = GetNormal(pContext, index);
            let vT: SVec3 = GetTexCoord(pContext, index);

            let mut bNotFound: bool = true;
            let mut l2: c_int = iL_in;
            let mut i2rec: c_int = -(1 as c_int);
            while l2 < l && bNotFound {
                let i2: c_int = pTmpVert[l2 as usize].index;
                let index2: c_int = piTriList_in_and_out[i2 as usize];
                let vP2: SVec3 = GetPosition(pContext, index2);
                let vN2: SVec3 = GetNormal(pContext, index2);
                let vT2: SVec3 = GetTexCoord(pContext, index2);
                i2rec = i2;

                //if (vP==vP2 && vN==vN2 && vT==vT2)
                if vP.x == vP2.x
                    && vP.y == vP2.y
                    && vP.z == vP2.z
                    && vN.x == vN2.x
                    && vN.y == vN2.y
                    && vN.z == vN2.z
                    && vT.x == vT2.x
                    && vT.y == vT2.y
                    && vT.z == vT2.z
                {
                    bNotFound = false;
                } else {
                    l2 += 1;
                }
            }

            // merge if previously found
            if !bNotFound {
                piTriList_in_and_out[i as usize] = piTriList_in_and_out[i2rec as usize];
            }

            l += 1;
        }
    } else {
        let mut iL: c_int = iL_in;
        let mut iR: c_int = iR_in;
        assert!(iR_in - iL_in > 0 as c_int, "at least 2 entries");

        // separate (by fSep) all points between iL_in and iR_in in pTmpVert[]
        while iL < iR {
            let mut bReadyLeftSwap: bool = false;
            let mut bReadyRightSwap: bool = false;
            while !bReadyLeftSwap && iL < iR {
                assert!(iL >= iL_in && iL <= iR_in);
                #[expect(clippy::neg_cmp_op_on_partial_ord)]
                {
                    bReadyLeftSwap = !(pTmpVert[iL as usize].vert[channel as usize] < fSep);
                }
                if !bReadyLeftSwap {
                    iL += 1;
                }
            }
            while !bReadyRightSwap && iL < iR {
                assert!(iR >= iL_in && iR <= iR_in);
                bReadyRightSwap = pTmpVert[iR as usize].vert[channel as usize] < fSep;
                if !bReadyRightSwap {
                    iR -= 1;
                }
            }
            assert!(iL < iR || !(bReadyLeftSwap && bReadyRightSwap));

            if bReadyLeftSwap && bReadyRightSwap {
                let sTmp: STmpVert = pTmpVert[iL as usize];
                assert!(iL < iR);
                pTmpVert[iL as usize] = pTmpVert[iR as usize];
                pTmpVert[iR as usize] = sTmp;
                iL += 1;
                iR -= 1;
            }
        }

        assert!(iL == iR + 1 as c_int || iL == iR);
        if iL == iR {
            let bReadyRightSwap_0: bool = pTmpVert[iR as usize].vert[channel as usize] < fSep;
            if bReadyRightSwap_0 {
                iL += 1;
            } else {
                iR -= 1;
            }
        }

        // only need to weld when there is more than 1 instance of the (x,y,z)
        if iL_in < iR {
            // weld all left of fSep
            MergeVertsFast(piTriList_in_and_out, pTmpVert, pContext, iL_in, iR);
        }
        if iL < iR_in {
            // weld all right of (or equal to) fSep
            MergeVertsFast(piTriList_in_and_out, pTmpVert, pContext, iL, iR_in);
        }
    };
}

fn GenerateInitialVerticesIndexList<I: MikkTSpaceInterface>(
    pTriInfos: &mut [STriInfo],
    piTriList_out: &mut [c_int],
    pContext: &I,
    iNrTrianglesIn: c_int,
) -> c_int {
    let mut iTSpacesOffs: c_int = 0 as c_int;
    let mut iDstTriIndex: c_int = 0 as c_int;
    let mut f = 0 as c_int;
    while f < pContext.get_num_faces() as c_int {
        let verts: c_int = pContext.get_num_vertices_of_face(f as usize) as c_int;
        if !(verts != 3 as c_int && verts != 4 as c_int) {
            pTriInfos[iDstTriIndex as usize].iOrgFaceNumber = f;
            pTriInfos[iDstTriIndex as usize].iTSpacesOffs = iTSpacesOffs;
            if verts == 3 as c_int {
                let pVerts = &mut pTriInfos[iDstTriIndex as usize].vert_num;
                pVerts[0] = 0 as c_int as c_uchar;
                pVerts[1] = 1 as c_int as c_uchar;
                pVerts[2] = 2 as c_int as c_uchar;
                piTriList_out[(iDstTriIndex * 3 as c_int + 0 as c_int) as usize] =
                    MakeIndex(f, 0 as c_int);
                piTriList_out[(iDstTriIndex * 3 as c_int + 1 as c_int) as usize] =
                    MakeIndex(f, 1 as c_int);
                piTriList_out[(iDstTriIndex * 3 as c_int + 2 as c_int) as usize] =
                    MakeIndex(f, 2 as c_int);
                iDstTriIndex += 1;
            } else {
                pTriInfos[(iDstTriIndex + 1 as c_int) as usize].iOrgFaceNumber = f;
                pTriInfos[(iDstTriIndex + 1 as c_int) as usize].iTSpacesOffs = iTSpacesOffs;

                // need an order independent way to evaluate
                // tspace on quads. This is done by splitting
                // along the shortest diagonal.
                let i0: c_int = MakeIndex(f, 0 as c_int);
                let i1: c_int = MakeIndex(f, 1 as c_int);
                let i2: c_int = MakeIndex(f, 2 as c_int);
                let i3: c_int = MakeIndex(f, 3 as c_int);
                let T0: SVec3 = GetTexCoord(pContext, i0);
                let T1: SVec3 = GetTexCoord(pContext, i1);
                let T2: SVec3 = GetTexCoord(pContext, i2);
                let T3: SVec3 = GetTexCoord(pContext, i3);
                let distSQ_02: c_float = (T2 - T0).length_squared();
                let distSQ_13: c_float = (T3 - T1).length_squared();
                let bQuadDiagIs_02 = if distSQ_02 < distSQ_13 {
                    true
                } else if distSQ_13 < distSQ_02 {
                    false
                } else {
                    let P0: SVec3 = GetPosition(pContext, i0);
                    let P1: SVec3 = GetPosition(pContext, i1);
                    let P2: SVec3 = GetPosition(pContext, i2);
                    let P3: SVec3 = GetPosition(pContext, i3);
                    let distSQ_02_0: c_float = (P2 - P0).length_squared();
                    let distSQ_13_0: c_float = (P3 - P1).length_squared();
                    distSQ_13_0 >= distSQ_02_0
                };
                if bQuadDiagIs_02 {
                    let pVerts_A = &mut pTriInfos[iDstTriIndex as usize].vert_num;
                    pVerts_A[0] = 0 as c_int as c_uchar;
                    pVerts_A[1] = 1 as c_int as c_uchar;
                    pVerts_A[2] = 2 as c_int as c_uchar;
                    piTriList_out[(iDstTriIndex * 3 as c_int + 0 as c_int) as usize] = i0;
                    piTriList_out[(iDstTriIndex * 3 as c_int + 1 as c_int) as usize] = i1;
                    piTriList_out[(iDstTriIndex * 3 as c_int + 2 as c_int) as usize] = i2;
                    iDstTriIndex += 1;
                    let pVerts_B = &mut pTriInfos[iDstTriIndex as usize].vert_num;
                    pVerts_B[0] = 0 as c_int as c_uchar;
                    pVerts_B[1] = 2 as c_int as c_uchar;
                    pVerts_B[2] = 3 as c_int as c_uchar;
                    piTriList_out[(iDstTriIndex * 3 as c_int + 0 as c_int) as usize] = i0;
                    piTriList_out[(iDstTriIndex * 3 as c_int + 1 as c_int) as usize] = i2;
                    piTriList_out[(iDstTriIndex * 3 as c_int + 2 as c_int) as usize] = i3;
                    iDstTriIndex += 1;
                } else {
                    let pVerts_A_0 = &mut pTriInfos[iDstTriIndex as usize].vert_num;
                    pVerts_A_0[0] = 0 as c_int as c_uchar;
                    pVerts_A_0[1] = 1 as c_int as c_uchar;
                    pVerts_A_0[2] = 3 as c_int as c_uchar;
                    piTriList_out[(iDstTriIndex * 3 as c_int + 0 as c_int) as usize] = i0;
                    piTriList_out[(iDstTriIndex * 3 as c_int + 1 as c_int) as usize] = i1;
                    piTriList_out[(iDstTriIndex * 3 as c_int + 2 as c_int) as usize] = i3;
                    iDstTriIndex += 1;
                    let pVerts_B_0 = &mut pTriInfos[iDstTriIndex as usize].vert_num;
                    pVerts_B_0[0] = 1 as c_int as c_uchar;
                    pVerts_B_0[1] = 2 as c_int as c_uchar;
                    pVerts_B_0[2] = 3 as c_int as c_uchar;
                    piTriList_out[(iDstTriIndex * 3 as c_int + 0 as c_int) as usize] = i1;
                    piTriList_out[(iDstTriIndex * 3 as c_int + 1 as c_int) as usize] = i2;
                    piTriList_out[(iDstTriIndex * 3 as c_int + 2 as c_int) as usize] = i3;
                    iDstTriIndex += 1;
                }
            }
            iTSpacesOffs += verts;
            assert!(iDstTriIndex <= iNrTrianglesIn);
        }
        f += 1;
    }

    let mut t = 0 as c_int;
    while t < iNrTrianglesIn {
        pTriInfos[t as usize].iFlag = 0 as c_int;
        t += 1;
    }

    // return total amount of tspaces
    iTSpacesOffs
}

fn GetPosition<I: MikkTSpaceInterface>(pContext: &I, index: c_int) -> SVec3 {
    let mut res: SVec3 = SVec3::ZERO;
    let (iF, iI) = IndexToData(index);
    let pos = pContext.get_position(iF as usize, iI as usize);
    res.x = pos[0 as c_int as usize];
    res.y = pos[1 as c_int as usize];
    res.z = pos[2 as c_int as usize];
    res
}

fn GetNormal<I: MikkTSpaceInterface>(pContext: &I, index: c_int) -> SVec3 {
    let mut res: SVec3 = SVec3::ZERO;
    let (iF, iI) = IndexToData(index);
    let norm = pContext.get_normal(iF as usize, iI as usize);
    res.x = norm[0 as c_int as usize];
    res.y = norm[1 as c_int as usize];
    res.z = norm[2 as c_int as usize];
    res
}

fn GetTexCoord<I: MikkTSpaceInterface>(pContext: &I, index: c_int) -> SVec3 {
    let mut res: SVec3 = SVec3::ZERO;
    let (iF, iI) = IndexToData(index);
    let texc = pContext.get_tex_coord(iF as usize, iI as usize);
    res.x = texc[0 as c_int as usize];
    res.y = texc[1 as c_int as usize];
    res.z = 1.0f32;
    res
}

/// returns the texture area times 2
fn CalcTexArea<I: MikkTSpaceInterface>(pContext: &I, indices: &[c_int]) -> c_float {
    let t1: SVec3 = GetTexCoord(pContext, indices[0]);
    let t2: SVec3 = GetTexCoord(pContext, indices[1]);
    let t3: SVec3 = GetTexCoord(pContext, indices[2]);

    let t21x: c_float = t2.x - t1.x;
    let t21y: c_float = t2.y - t1.y;
    let t31x: c_float = t3.x - t1.x;
    let t31y: c_float = t3.y - t1.y;

    let fSignedAreaSTx2: c_float = t21x * t31y - t21y * t31x;
    if fSignedAreaSTx2 < 0 as c_int as c_float {
        -fSignedAreaSTx2
    } else {
        fSignedAreaSTx2
    }
}

fn InitTriInfo<I: MikkTSpaceInterface>(
    pTriInfos: &mut [STriInfo],
    piTriListIn: &[c_int],
    pContext: &I,
    iNrTrianglesIn: c_int,
) {
    let mut t: c_int = 0 as c_int;
    // pTriInfos[f].iFlag is cleared in GenerateInitialVerticesIndexList() which is called before this function.

    // generate neighbor info list
    let mut f = 0 as c_int;
    while f < iNrTrianglesIn {
        let mut i = 0 as c_int;
        while i < 3 as c_int {
            pTriInfos[f as usize].FaceNeighbors[i as usize] = -(1 as c_int);
            let fresh2 = &mut pTriInfos[f as usize].AssignedGroup[i as usize];
            *fresh2 = None;
            pTriInfos[f as usize].vOs.x = 0.0f32;
            pTriInfos[f as usize].vOs.y = 0.0f32;
            pTriInfos[f as usize].vOs.z = 0.0f32;
            pTriInfos[f as usize].vOt.x = 0.0f32;
            pTriInfos[f as usize].vOt.y = 0.0f32;
            pTriInfos[f as usize].vOt.z = 0.0f32;
            pTriInfos[f as usize].fMagS = 0 as c_int as c_float;
            pTriInfos[f as usize].fMagT = 0 as c_int as c_float;

            // assumed bad
            pTriInfos[f as usize].iFlag |= GROUP_WITH_ANY;

            i += 1;
        }
        f += 1;
    }

    // evaluate first order derivatives
    f = 0 as c_int;
    while f < iNrTrianglesIn {
        // initial values
        let v1: SVec3 = GetPosition(
            pContext,
            piTriListIn[(f * 3 as c_int + 0 as c_int) as usize],
        );
        let v2: SVec3 = GetPosition(
            pContext,
            piTriListIn[(f * 3 as c_int + 1 as c_int) as usize],
        );
        let v3: SVec3 = GetPosition(
            pContext,
            piTriListIn[(f * 3 as c_int + 2 as c_int) as usize],
        );
        let t1: SVec3 = GetTexCoord(
            pContext,
            piTriListIn[(f * 3 as c_int + 0 as c_int) as usize],
        );
        let t2: SVec3 = GetTexCoord(
            pContext,
            piTriListIn[(f * 3 as c_int + 1 as c_int) as usize],
        );
        let t3: SVec3 = GetTexCoord(
            pContext,
            piTriListIn[(f * 3 as c_int + 2 as c_int) as usize],
        );

        let t21x: c_float = t2.x - t1.x;
        let t21y: c_float = t2.y - t1.y;
        let t31x: c_float = t3.x - t1.x;
        let t31y: c_float = t3.y - t1.y;
        let d1: SVec3 = v2 - v1;
        let d2: SVec3 = v3 - v1;
        let fSignedAreaSTx2: c_float = t21x * t31y - t21y * t31x;
        let vOs: SVec3 = (t31y * d1) - (t21y * d2); // eq 18
        let vOt: SVec3 = (-t31x * d1) + (t21x * d2); // eq 19

        pTriInfos[f as usize].iFlag |= if fSignedAreaSTx2 > 0 as c_int as c_float {
            ORIENT_PRESERVING
        } else {
            0 as c_int
        };

        if not_zero(fSignedAreaSTx2) {
            let fAbsArea: c_float = fabsf(fSignedAreaSTx2);
            let fLenOs: c_float = vOs.length();
            let fLenOt: c_float = vOt.length();
            let fS: c_float = if pTriInfos[f as usize].iFlag & ORIENT_PRESERVING == 0 as c_int {
                -1.0f32
            } else {
                1.0f32
            };
            if not_zero(fLenOs) {
                pTriInfos[f as usize].vOs = (fS / fLenOs) * vOs;
            }
            if not_zero(fLenOt) {
                pTriInfos[f as usize].vOt = (fS / fLenOt) * vOt;
            }

            // evaluate magnitudes prior to normalization of vOs and vOt
            pTriInfos[f as usize].fMagS = fLenOs / fAbsArea;
            pTriInfos[f as usize].fMagT = fLenOt / fAbsArea;

            // if this is a good triangle
            if not_zero(pTriInfos[f as usize].fMagS) && not_zero(pTriInfos[f as usize].fMagT) {
                pTriInfos[f as usize].iFlag &= !GROUP_WITH_ANY;
            }
        }
        f += 1;
    }

    // force otherwise healthy quads to a fixed orientation
    while t < iNrTrianglesIn - 1 as c_int {
        let iFO_a: c_int = pTriInfos[t as usize].iOrgFaceNumber;
        let iFO_b: c_int = pTriInfos[(t + 1 as c_int) as usize].iOrgFaceNumber;
        if iFO_a == iFO_b {
            // this is a quad
            let bIsDeg_a: bool = pTriInfos[t as usize].iFlag & MARK_DEGENERATE != 0 as c_int;
            let bIsDeg_b: bool =
                pTriInfos[(t + 1 as c_int) as usize].iFlag & MARK_DEGENERATE != 0 as c_int;

            // bad triangles should already have been removed by
            // DegenPrologue(), but just in case check bIsDeg_a and bIsDeg_a are false
            if !(bIsDeg_a || bIsDeg_b) {
                let bOrientA: bool = pTriInfos[t as usize].iFlag & ORIENT_PRESERVING != 0 as c_int;
                let bOrientB: bool =
                    pTriInfos[(t + 1 as c_int) as usize].iFlag & ORIENT_PRESERVING != 0 as c_int;

                // if this happens the quad has extremely bad mapping!!
                if bOrientA != bOrientB {
                    let mut bChooseOrientFirstTri: bool = false;
                    #[expect(clippy::if_same_then_else)]
                    if pTriInfos[(t + 1 as c_int) as usize].iFlag & GROUP_WITH_ANY != 0 as c_int {
                        bChooseOrientFirstTri = true;
                    } else if CalcTexArea(
                        pContext,
                        &piTriListIn[{
                            let a = (t * 3 as c_int + 0 as c_int) as usize;
                            let b = a + 3;
                            a..b
                        }],
                    ) >= CalcTexArea(
                        pContext,
                        &piTriListIn[{
                            let a = ((t + 1 as c_int) * 3 as c_int + 0 as c_int) as usize;
                            let b = a + 3;
                            a..b
                        }],
                    ) {
                        bChooseOrientFirstTri = true;
                    }

                    // force match
                    let t0: c_int = if bChooseOrientFirstTri {
                        t
                    } else {
                        t + 1 as c_int
                    };
                    let t1_0: c_int = if bChooseOrientFirstTri {
                        t + 1 as c_int
                    } else {
                        t
                    };

                    // clear first
                    pTriInfos[t1_0 as usize].iFlag &= !ORIENT_PRESERVING;
                    // copy bit
                    pTriInfos[t1_0 as usize].iFlag |=
                        pTriInfos[t0 as usize].iFlag & ORIENT_PRESERVING;
                }
            }
            t += 2 as c_int;
        } else {
            t += 1;
        }
    }

    // if /* can't allocate */ {
    //     BuildNeighborsSlow(pTriInfos, piTriListIn, iNrTrianglesIn);
    // }

    // match up edge pairs
    let mut pEdges: Vec<SEdge> =
        vec![SEdge::ZERO; (iNrTrianglesIn as c_ulong).wrapping_mul(3) as usize];
    let vert_count = pEdges.len();
    let face_count = vert_count / 3;
    BuildNeighborsFast(
        &mut pTriInfos[..face_count],
        &mut pEdges,
        &piTriListIn[..vert_count],
        iNrTrianglesIn,
    );
}

fn Build4RuleGroups(
    pTriInfos: &mut [STriInfo],
    pGroups: &mut [SGroup],
    piTriListIn: &[c_int],
    iNrTrianglesIn: c_int,
) -> c_int {
    let iNrMaxGroups: c_int = iNrTrianglesIn * 3 as c_int;
    let mut iNrActiveGroups: c_int = 0 as c_int;

    let mut f = 0 as c_int;
    while f < iNrTrianglesIn {
        let mut i = 0 as c_int;
        while i < 3 as c_int {
            // if not assigned to a group
            if pTriInfos[f as usize].iFlag & GROUP_WITH_ANY == 0 as c_int
                && pTriInfos[f as usize].AssignedGroup[i as usize].is_none()
            {
                let vert_index: c_int = piTriListIn[(f * 3 as c_int + i) as usize];
                assert!(iNrActiveGroups < iNrMaxGroups);
                pTriInfos[f as usize].AssignedGroup[i as usize] = Some(iNrActiveGroups as usize);
                let this_group = &mut pGroups[iNrActiveGroups as usize];
                this_group.id = iNrActiveGroups as usize;
                this_group.iVertexRepresentitive = vert_index;
                this_group.bOrientPreservering =
                    pTriInfos[f as usize].iFlag & ORIENT_PRESERVING != 0 as c_int;
                this_group.pFaceIndices = Vec::new();
                iNrActiveGroups += 1;

                AddTriToGroup(this_group, f);
                let bOrPre = pTriInfos[f as usize].iFlag & ORIENT_PRESERVING != 0 as c_int;
                let neigh_indexL = pTriInfos[f as usize].FaceNeighbors[i as usize];
                let neigh_indexR = pTriInfos[f as usize].FaceNeighbors[(if i > 0 as c_int {
                    i - 1 as c_int
                } else {
                    2 as c_int
                }) as usize];

                if neigh_indexL >= 0 as c_int {
                    // neighbor
                    let bAnswer: bool =
                        AssignRecur(piTriListIn, pTriInfos, neigh_indexL, this_group);
                    let bOrPre2: bool =
                        pTriInfos[neigh_indexL as usize].iFlag & ORIENT_PRESERVING != 0 as c_int;
                    let bDiff: bool = bOrPre != bOrPre2;
                    assert!(bAnswer || bDiff);
                }
                if neigh_indexR >= 0 as c_int {
                    // neighbor
                    let bAnswer_0: bool =
                        AssignRecur(piTriListIn, pTriInfos, neigh_indexR, this_group);
                    let bOrPre2_0: bool =
                        pTriInfos[neigh_indexR as usize].iFlag & ORIENT_PRESERVING != 0 as c_int;
                    let bDiff_0: bool = bOrPre != bOrPre2_0;
                    assert!(bAnswer_0 || bDiff_0);
                }
            }
            i += 1;
        }
        f += 1;
    }

    iNrActiveGroups
}

fn AddTriToGroup(pGroup: &mut SGroup, iTriIndex: c_int) {
    pGroup.pFaceIndices.push(iTriIndex);
}

fn AssignRecur(
    piTriListIn: &[c_int],
    psTriInfos: &mut [STriInfo],
    iMyTriIndex: c_int,
    pGroup: &mut SGroup,
) -> bool {
    let pMyTriInfo = &mut psTriInfos[iMyTriIndex as usize];

    // track down vertex
    let iVertRep: c_int = pGroup.iVertexRepresentitive;
    let pVerts = &piTriListIn[{
        let a = (3 as c_int * iMyTriIndex + 0 as c_int) as usize;
        let b = a + 3;
        a..b
    }];
    let mut i: c_int = -(1 as c_int);
    if pVerts[0] == iVertRep {
        i = 0 as c_int;
    } else if pVerts[1] == iVertRep {
        i = 1 as c_int;
    } else if pVerts[2] == iVertRep {
        i = 2 as c_int;
    }
    assert!(i >= 0 as c_int && i < 3 as c_int);

    // early out
    if pMyTriInfo.AssignedGroup[i as usize] == Some(pGroup.id) {
        return true;
    } else if (pMyTriInfo.AssignedGroup[i as usize]).is_some() {
        return false;
    }
    if pMyTriInfo.iFlag & GROUP_WITH_ANY != 0 as c_int
        && (pMyTriInfo.AssignedGroup[0 as c_int as usize]).is_none()
        && (pMyTriInfo.AssignedGroup[1 as c_int as usize]).is_none()
        && (pMyTriInfo.AssignedGroup[2 as c_int as usize]).is_none()
    {
        // first to group with a group-with-anything triangle
        // determines it's orientation.
        // This is the only existing order dependency in the code!!
        pMyTriInfo.iFlag &= !ORIENT_PRESERVING;
        pMyTriInfo.iFlag |= if pGroup.bOrientPreservering {
            ORIENT_PRESERVING
        } else {
            0 as c_int
        };
    }
    let bOrient: bool = pMyTriInfo.iFlag & ORIENT_PRESERVING != 0 as c_int;
    if bOrient != pGroup.bOrientPreservering {
        return false;
    }

    AddTriToGroup(&mut *pGroup, iMyTriIndex);
    pMyTriInfo.AssignedGroup[i as usize] = Some(pGroup.id);

    let neigh_indexL: c_int = pMyTriInfo.FaceNeighbors[i as usize];
    let neigh_indexR: c_int = pMyTriInfo.FaceNeighbors[(if i > 0 as c_int {
        i - 1 as c_int
    } else {
        2 as c_int
    }) as usize];
    if neigh_indexL >= 0 as c_int {
        AssignRecur(piTriListIn, psTriInfos, neigh_indexL, pGroup);
    }
    if neigh_indexR >= 0 as c_int {
        AssignRecur(piTriListIn, psTriInfos, neigh_indexR, pGroup);
    }

    true
}

fn GenerateTSpaces<I: MikkTSpaceInterface>(
    psTspace: &mut [STSpace],
    pTriInfos: &[STriInfo],
    pGroups: &[SGroup],
    iNrActiveGroups: c_int,
    piTriListIn: &[c_int],
    fThresCos: c_float,
    pContext: &I,
) -> bool {
    let mut iMaxNrFaces: c_int = 0 as c_int;
    let mut g = 0 as c_int;
    while g < iNrActiveGroups {
        if iMaxNrFaces < pGroups[g as usize].pFaceIndices.len() as c_int {
            iMaxNrFaces = pGroups[g as usize].pFaceIndices.len() as c_int;
        }
        g += 1;
    }

    if iMaxNrFaces == 0 as c_int {
        return true;
    }

    // make initial allocations
    let mut pSubGroupTspace: Vec<STSpace> = vec![STSpace::ZERO; iMaxNrFaces as usize];
    let mut pUniSubGroups: Vec<SSubGroup> = vec![SSubGroup::ZERO; iMaxNrFaces as usize];
    let mut g = 0 as c_int;
    while g < iNrActiveGroups {
        let pGroup = &pGroups[g as usize];
        let mut iUniqueSubGroups: c_int = 0 as c_int;

        // triangles
        let mut i = 0 as c_int;
        while i < pGroup.pFaceIndices.len() as c_int {
            // triangle number
            let f: c_int = (pGroup.pFaceIndices)[i as usize];
            let mut tmp_group: SSubGroup = SSubGroup::ZERO;
            let index = if pTriInfos[f as usize].AssignedGroup[0 as c_int as usize]
                == Some(g as usize)
            {
                0 as c_int
            } else if pTriInfos[f as usize].AssignedGroup[1 as c_int as usize] == Some(g as usize) {
                1 as c_int
            } else if pTriInfos[f as usize].AssignedGroup[2 as c_int as usize] == Some(g as usize) {
                2 as c_int
            } else {
                panic!()
            };

            let iVertIndex = piTriListIn[(f * 3 as c_int + index) as usize];
            assert!(iVertIndex == pGroup.iVertexRepresentitive);

            // is normalized already
            let n = GetNormal(pContext, iVertIndex);

            // project
            let mut vOs = pTriInfos[f as usize].vOs - ((n.dot(pTriInfos[f as usize].vOs)) * n);
            let mut vOt = pTriInfos[f as usize].vOt - ((n.dot(pTriInfos[f as usize].vOt)) * n);
            vOs.normalize_or_zero();
            vOt.normalize_or_zero();

            // original face number
            let iOF_1 = pTriInfos[f as usize].iOrgFaceNumber;

            let mut j = 0 as c_int;
            while j < pGroup.pFaceIndices.len() as c_int {
                // triangle number
                let t: c_int = (pGroup.pFaceIndices)[j as usize];
                let iOF_2: c_int = pTriInfos[t as usize].iOrgFaceNumber;

                // project
                let mut vOs2: SVec3 =
                    pTriInfos[t as usize].vOs - ((n.dot(pTriInfos[t as usize].vOs)) * n);
                let mut vOt2: SVec3 =
                    pTriInfos[t as usize].vOt - ((n.dot(pTriInfos[t as usize].vOt)) * n);
                vOs2.normalize_or_zero();
                vOt2.normalize_or_zero();

                let bAny: bool = (pTriInfos[f as usize].iFlag | pTriInfos[t as usize].iFlag)
                    & GROUP_WITH_ANY
                    != 0 as c_int;
                // make sure triangles which belong to the same quad are joined.
                let bSameOrgFace: bool = iOF_1 == iOF_2;

                let fCosS: c_float = vOs.dot(vOs2);
                let fCosT: c_float = vOt.dot(vOt2);

                assert!(f != t || bSameOrgFace, "sanity check");
                if bAny || bSameOrgFace || fCosS > fThresCos && fCosT > fThresCos {
                    tmp_group.pTriMembers.push(t);
                }

                j += 1;
            }

            // sort pTmpMembers
            tmp_group.pTriMembers.sort();

            // look for an existing match
            let mut bFound = false;
            let mut l = 0 as c_int;
            while l < iUniqueSubGroups && !bFound {
                bFound = tmp_group == pUniSubGroups[l as usize];
                if !bFound {
                    l += 1;
                }
            }

            // assign tangent space index
            assert!(bFound || l == iUniqueSubGroups);

            // if no match was found we allocate a new subgroup
            if !bFound {
                // insert new subgroup
                pUniSubGroups[iUniqueSubGroups as usize].pTriMembers =
                    tmp_group.pTriMembers.clone();
                pSubGroupTspace[iUniqueSubGroups as usize] = EvalTspace(
                    &tmp_group.pTriMembers,
                    piTriListIn,
                    pTriInfos,
                    pContext,
                    pGroup.iVertexRepresentitive,
                );
                iUniqueSubGroups += 1;
            }

            // output tspace
            let iOffs: c_int = pTriInfos[f as usize].iTSpacesOffs;
            let iVert: c_int = pTriInfos[f as usize].vert_num[index as usize] as c_int;
            let pTS_out = &mut psTspace[(iOffs + iVert) as usize];
            assert!(pTS_out.iCounter < 2 as c_int);
            assert!(
                (pTriInfos[f as usize].iFlag & 8 as c_int != 0 as c_int)
                    == pGroup.bOrientPreservering
            );
            if pTS_out.iCounter == 1 as c_int {
                *pTS_out = AvgTSpace(*pTS_out, pSubGroupTspace[l as usize]);
                // update counter
                pTS_out.iCounter = 2 as c_int;
                pTS_out.bOrient = pGroup.bOrientPreservering;
            } else {
                assert!(pTS_out.iCounter == 0 as c_int);
                *pTS_out = pSubGroupTspace[l as usize];
                // update counter
                pTS_out.iCounter = 1 as c_int;
                pTS_out.bOrient = pGroup.bOrientPreservering;
            }

            i += 1;
        }

        g += 1;
    }

    true
}

fn EvalTspace<I: MikkTSpaceInterface>(
    face_indices: &[c_int],
    piTriListIn: &[c_int],
    pTriInfos: &[STriInfo],
    pContext: &I,
    iVertexRepresentitive: c_int,
) -> STSpace {
    let iFaces = face_indices.len() as c_int;
    let mut res: STSpace = STSpace {
        vOs: SVec3::ZERO,
        fMagS: 0.,
        vOt: SVec3::ZERO,
        fMagT: 0.,
        iCounter: 0,
        bOrient: false,
    };
    let mut fAngleSum: c_float = 0 as c_int as c_float;
    res.vOs.x = 0.0f32;
    res.vOs.y = 0.0f32;
    res.vOs.z = 0.0f32;
    res.vOt.x = 0.0f32;
    res.vOt.y = 0.0f32;
    res.vOt.z = 0.0f32;
    res.fMagS = 0 as c_int as c_float;
    res.fMagT = 0 as c_int as c_float;

    let mut face = 0 as c_int;
    while face < iFaces {
        let f: c_int = face_indices[face as usize];

        // only valid triangles get to add their contribution
        if pTriInfos[f as usize].iFlag & GROUP_WITH_ANY == 0 as c_int {
            let i = if piTriListIn[(3 as c_int * f + 0 as c_int) as usize] == iVertexRepresentitive
            {
                0 as c_int
            } else if piTriListIn[(3 as c_int * f + 1 as c_int) as usize] == iVertexRepresentitive {
                1 as c_int
            } else if piTriListIn[(3 as c_int * f + 2 as c_int) as usize] == iVertexRepresentitive {
                2 as c_int
            } else {
                panic!()
            };

            // project
            let index = piTriListIn[(3 as c_int * f + i) as usize];
            let n = GetNormal(pContext, index);
            let mut vOs = pTriInfos[f as usize].vOs - ((n.dot(pTriInfos[f as usize].vOs)) * n);
            let mut vOt = pTriInfos[f as usize].vOt - (n.dot(pTriInfos[f as usize].vOt) * n);
            vOs.normalize_or_zero();
            vOt.normalize_or_zero();

            let i2 = piTriListIn[(3 as c_int * f
                + (if i < 2 as c_int {
                    i + 1 as c_int
                } else {
                    0 as c_int
                })) as usize];
            let i1 = piTriListIn[(3 as c_int * f + i) as usize];
            let i0 = piTriListIn[(3 as c_int * f
                + (if i > 0 as c_int {
                    i - 1 as c_int
                } else {
                    2 as c_int
                })) as usize];

            let p0 = GetPosition(pContext, i0);
            let p1 = GetPosition(pContext, i1);
            let p2 = GetPosition(pContext, i2);
            let mut v1 = p0 - p1;
            let mut v2 = p2 - p1;

            // project
            v1 = v1 - ((n.dot(v1)) * n);
            v1.normalize_or_zero();
            v2 = v2 - ((n.dot(v2)) * n);
            v2.normalize_or_zero();

            // weight contribution by the angle
            // between the two edge vectors
            let mut fCos = v1.dot(v2);
            fCos = if fCos > 1 as c_int as c_float {
                1 as c_int as c_float
            } else if fCos < -(1 as c_int) as c_float {
                -(1 as c_int) as c_float
            } else {
                fCos
            };
            let fAngle = acos(fCos as c_double) as c_float;
            let fMagS = pTriInfos[f as usize].fMagS;
            let fMagT = pTriInfos[f as usize].fMagT;

            res.vOs = res.vOs + (fAngle * vOs);
            res.vOt = res.vOt + (fAngle * vOt);
            res.fMagS += fAngle * fMagS;
            res.fMagT += fAngle * fMagT;
            fAngleSum += fAngle;
        }
        face += 1;
    }

    // normalize
    res.vOs.normalize_or_zero();
    res.vOt.normalize_or_zero();
    if fAngleSum > 0 as c_int as c_float {
        res.fMagS /= fAngleSum;
        res.fMagT /= fAngleSum;
    }

    res
}

fn BuildNeighborsFast(
    pTriInfos: &mut [STriInfo],
    pEdges: &mut [SEdge],
    piTriListIn: &[c_int],
    iNrTrianglesIn: c_int,
) {
    // build array of edges
    let uSeed: c_uint = INTERNAL_RND_SORT_SEED as c_uint;
    let mut f = 0 as c_int;
    while f < iNrTrianglesIn {
        let mut i = 0 as c_int;
        while i < 3 as c_int {
            let i0: c_int = piTriListIn[(f * 3 as c_int + i) as usize];
            let i1: c_int = piTriListIn[(f * 3 as c_int
                + (if i < 2 as c_int {
                    i + 1 as c_int
                } else {
                    0 as c_int
                })) as usize];

            // put minimum index in i0
            pEdges[(f * 3 as c_int + i) as usize].i0 = i0.min(i1);
            // put maximum index in i1
            pEdges[(f * 3 as c_int + i) as usize].i1 = i0.max(i1);
            // record face number
            pEdges[(f * 3 as c_int + i) as usize].f = f;

            i += 1;
        }

        f += 1;
    }

    // sort over all edges by i0, this is the pricy one.
    QuickSortEdges(
        pEdges,
        0 as c_int,
        iNrTrianglesIn * 3 as c_int - 1 as c_int,
        0 as c_int,
        uSeed,
    );
    let iEntries = iNrTrianglesIn * 3 as c_int;
    let mut iCurStartIndex = 0 as c_int;
    let mut i = 1 as c_int;
    while i < iEntries {
        if pEdges[iCurStartIndex as usize].i0 != pEdges[i as usize].i0 {
            let iL: c_int = iCurStartIndex;
            let iR: c_int = i - 1 as c_int;
            iCurStartIndex = i;
            QuickSortEdges(pEdges, iL, iR, 1 as c_int, uSeed);
        }
        i += 1;
    }
    iCurStartIndex = 0 as c_int;
    let mut i = 1 as c_int;
    while i < iEntries {
        if pEdges[iCurStartIndex as usize].i0 != pEdges[i as usize].i0
            || pEdges[iCurStartIndex as usize].i1 != pEdges[i as usize].i1
        {
            let iL_0: c_int = iCurStartIndex;
            let iR_0: c_int = i - 1 as c_int;
            iCurStartIndex = i;
            QuickSortEdges(pEdges, iL_0, iR_0, 2 as c_int, uSeed);
        }
        i += 1;
    }

    // pair up, adjacent triangles
    let mut i = 0 as c_int;
    while i < iEntries {
        let i0_0: c_int = pEdges[i as usize].i0;
        let i1_0: c_int = pEdges[i as usize].i1;
        let f_0: c_int = pEdges[i as usize].f;

        let mut edgenum_B: c_int = 0 as c_int;

        // resolve index ordering and edge_num
        let (edgenum_A, i0_A, i1_A) = get_edge(
            &piTriListIn[{
                let a = (f_0 * 3 as c_int) as usize;
                let b = a + 3;
                a..b
            }],
            i0_0,
            i1_0,
        )
        .unwrap();
        let bUnassigned_A =
            pTriInfos[f_0 as usize].FaceNeighbors[edgenum_A as usize] == -(1 as c_int);

        if bUnassigned_A {
            // get true index ordering
            let mut j: c_int = i + 1 as c_int;
            let mut bNotFound: bool = true;
            while j < iEntries
                && i0_0 == pEdges[j as usize].i0
                && i1_0 == pEdges[j as usize].i1
                && bNotFound
            {
                let t = pEdges[j as usize].f;
                // flip i0_B and i1_B
                // resolve index ordering and edge_num
                let (edgenum, i1_B, i0_B) = get_edge(
                    &piTriListIn[{
                        let a = (t * 3 as c_int) as usize;
                        let b = a + 3;
                        a..b
                    }],
                    pEdges[j as usize].i0,
                    pEdges[j as usize].i1,
                )
                .unwrap();
                edgenum_B = edgenum;
                let bUnassigned_B =
                    pTriInfos[t as usize].FaceNeighbors[edgenum_B as usize] == -(1 as c_int);

                if i0_A == i0_B && i1_A == i1_B && bUnassigned_B {
                    bNotFound = false;
                } else {
                    j += 1;
                }
            }

            if !bNotFound {
                let t_0: c_int = pEdges[j as usize].f;
                pTriInfos[f_0 as usize].FaceNeighbors[edgenum_A as usize] = t_0;
                pTriInfos[t_0 as usize].FaceNeighbors[edgenum_B as usize] = f_0;
            }
        }

        i += 1;
    }
}
/// Note that this method _should_ be able to be replaced with `[T]::sort` and an
/// appropriate implementation of [`Ord`] for [`SEdge`].
/// However, in initial testing this caused incorrect results, indicating this sort
/// may not be implemented correctly.
/// Further testing is required.
fn QuickSortEdges(
    pSortBuffer: &mut [SEdge],
    iLeft: c_int,
    iRight: c_int,
    channel: c_int,
    mut uSeed: c_uint,
) {
    let iElems: c_int = iRight - iLeft + 1 as c_int;
    #[expect(clippy::comparison_chain)]
    if iElems < 2 as c_int {
        return;
    } else if iElems == 2 as c_int {
        if pSortBuffer[iLeft as usize][channel as usize]
            > pSortBuffer[iRight as usize][channel as usize]
        {
            pSortBuffer.swap(iLeft as usize, iRight as usize);
        }
        return;
    }
    let mut t = uSeed & 31 as c_int as c_uint;
    t = uSeed.wrapping_shl(t) | uSeed.wrapping_shr((32 as c_int as c_uint).wrapping_sub(t));
    uSeed = uSeed.wrapping_add(t).wrapping_add(3 as c_int as c_uint);
    let mut iL = iLeft;
    let mut iR = iRight;
    let n = iR - iL + 1 as c_int;
    assert!(n >= 0 as c_int);
    let index = uSeed.wrapping_rem(n as c_uint) as c_int;
    let iMid = pSortBuffer[(index + iL) as usize][channel as usize];
    loop {
        while pSortBuffer[iL as usize][channel as usize] < iMid {
            iL += 1;
        }
        while pSortBuffer[iR as usize][channel as usize] > iMid {
            iR -= 1;
        }
        if iL <= iR {
            pSortBuffer.swap(iL as usize, iR as usize);
            iL += 1;
            iR -= 1;
        }
        if iL > iR {
            break;
        }
    }
    if iLeft < iR {
        QuickSortEdges(pSortBuffer, iLeft, iR, channel, uSeed);
    }
    if iL < iRight {
        QuickSortEdges(pSortBuffer, iL, iRight, channel, uSeed);
    }
}

/// Finds the index of the edge `(i0_in, i1_in)` within `indices`, additionally
/// returning `i0_in` and `i1_in` in the same order as they are stored within `indices`.
fn get_edge(indices: &[c_int], i0: c_int, i1: c_int) -> Option<(c_int, c_int, c_int)> {
    indices
        .iter()
        .copied()
        .zip(indices.iter().copied().cycle().skip(1))
        .enumerate()
        .find(|&(_, (a, b))| (a.min(b), a.max(b)) == (i0.min(i1), i0.max(i1)))
        .map(|(edgenum, (a, b))| (edgenum as c_int, a as c_int, b as c_int))
}

fn DegenPrologue(
    pTriInfos: &mut [STriInfo],
    piTriList_out: &mut [c_int],
    iNrTrianglesIn: c_int,
    iTotTris: c_int,
) {
    // locate quads with only one good triangle
    let mut t: c_int = 0 as c_int;
    while t < iTotTris - 1 as c_int {
        let iFO_a: c_int = pTriInfos[t as usize].iOrgFaceNumber;
        let iFO_b: c_int = pTriInfos[(t + 1 as c_int) as usize].iOrgFaceNumber;
        if iFO_a == iFO_b {
            // this is a quad
            let bIsDeg_a: bool = pTriInfos[t as usize].iFlag & MARK_DEGENERATE != 0 as c_int;
            let bIsDeg_b: bool =
                pTriInfos[(t + 1 as c_int) as usize].iFlag & MARK_DEGENERATE != 0 as c_int;
            if bIsDeg_a ^ bIsDeg_b {
                pTriInfos[t as usize].iFlag |= QUAD_ONE_DEGEN_TRI;
                pTriInfos[(t + 1 as c_int) as usize].iFlag |= QUAD_ONE_DEGEN_TRI;
            }
            t += 2 as c_int;
        } else {
            t += 1;
        }
    }

    // reorder list so all degen triangles are moved to the back
    // without reordering the good triangles
    let mut iNextGoodTriangleSearchIndex = 1 as c_int;
    let mut t = 0 as c_int;
    let mut bStillFindingGoodOnes = true;
    while t < iNrTrianglesIn && bStillFindingGoodOnes {
        let bIsGood: bool = pTriInfos[t as usize].iFlag & MARK_DEGENERATE == 0 as c_int;
        if bIsGood {
            if iNextGoodTriangleSearchIndex < t + 2 as c_int {
                iNextGoodTriangleSearchIndex = t + 2 as c_int;
            }
        } else {
            // search for the first good triangle.
            let mut bJustADegenerate: bool = true;
            while bJustADegenerate && iNextGoodTriangleSearchIndex < iTotTris {
                let bIsGood_0: bool = pTriInfos[iNextGoodTriangleSearchIndex as usize].iFlag
                    & MARK_DEGENERATE
                    == 0 as c_int;
                if bIsGood_0 {
                    bJustADegenerate = false;
                } else {
                    iNextGoodTriangleSearchIndex += 1;
                }
            }

            let t0 = t;
            let t1 = iNextGoodTriangleSearchIndex;
            iNextGoodTriangleSearchIndex += 1;
            assert!(iNextGoodTriangleSearchIndex > t + 1 as c_int);

            // swap triangle t0 and t1
            if !bJustADegenerate {
                let mut i = 0 as c_int;
                while i < 3 as c_int {
                    piTriList_out.swap(
                        (t0 * 3 as c_int + i) as usize,
                        (t1 * 3 as c_int + i) as usize,
                    );
                    i += 1;
                }
                pTriInfos.swap(t0 as usize, t1 as usize);
            } else {
                // this is not supposed to happen
                bStillFindingGoodOnes = false;
            }
        }
        if bStillFindingGoodOnes {
            t += 1;
        }
    }

    assert!(bStillFindingGoodOnes, "code will still work");
    assert!(iNrTrianglesIn == t);
}

fn DegenEpilogue<I: MikkTSpaceInterface>(
    psTspace: &mut [STSpace],
    pTriInfos: &[STriInfo],
    piTriListIn: &[c_int],
    pContext: &I,
    iNrTrianglesIn: c_int,
    iTotTris: c_int,
) {
    // deal with degenerate triangles
    // punishment for degenerate triangles is O(N^2)
    let mut t = iNrTrianglesIn;
    while t < iTotTris {
        // degenerate triangles on a quad with one good triangle are skipped
        // here but processed in the next loop
        let bSkip: bool = pTriInfos[t as usize].iFlag & QUAD_ONE_DEGEN_TRI != 0 as c_int;

        if !bSkip {
            let mut i = 0 as c_int;
            while i < 3 as c_int {
                let index1: c_int = piTriListIn[(t * 3 as c_int + i) as usize];
                // search through the good triangles
                let mut bNotFound: bool = true;
                let mut j: c_int = 0 as c_int;
                while bNotFound && j < 3 as c_int * iNrTrianglesIn {
                    let index2: c_int = piTriListIn[j as usize];
                    if index1 == index2 {
                        bNotFound = false;
                    } else {
                        j += 1;
                    }
                }

                if !bNotFound {
                    let iTri: c_int = j / 3 as c_int;
                    let iVert: c_int = j % 3 as c_int;
                    let iSrcVert: c_int =
                        pTriInfos[iTri as usize].vert_num[iVert as usize] as c_int;
                    let iSrcOffs: c_int = pTriInfos[iTri as usize].iTSpacesOffs;
                    let iDstVert: c_int = pTriInfos[t as usize].vert_num[i as usize] as c_int;
                    let iDstOffs: c_int = pTriInfos[t as usize].iTSpacesOffs;

                    // copy tspace
                    psTspace[(iDstOffs + iDstVert) as usize] =
                        psTspace[(iSrcOffs + iSrcVert) as usize];
                }

                i += 1;
            }
        }

        t += 1;
    }

    // deal with degenerate quads with one good triangle
    t = 0 as c_int;
    while t < iNrTrianglesIn {
        // this triangle belongs to a quad where the
        // other triangle is degenerate
        if pTriInfos[t as usize].iFlag & QUAD_ONE_DEGEN_TRI != 0 as c_int {
            let pV: [u8; 4] = pTriInfos[t as usize].vert_num;
            let iFlag: c_int = (1 as c_int) << pV[0] as c_int
                | (1 as c_int) << pV[1] as c_int
                | (1 as c_int) << pV[2] as c_int;
            let mut iMissingIndex: c_int = 0 as c_int;
            if iFlag & 2 as c_int == 0 as c_int {
                iMissingIndex = 1 as c_int;
            } else if iFlag & 4 as c_int == 0 as c_int {
                iMissingIndex = 2 as c_int;
            } else if iFlag & 8 as c_int == 0 as c_int {
                iMissingIndex = 3 as c_int;
            }

            let iOrgF = pTriInfos[t as usize].iOrgFaceNumber;
            let vDstP = GetPosition(pContext, MakeIndex(iOrgF, iMissingIndex));
            let mut bNotFound_0 = true;
            let mut i_0 = 0 as c_int;
            while bNotFound_0 && i_0 < 3 as c_int {
                let iVert_0: c_int = pV[i_0 as usize] as c_int;
                let vSrcP: SVec3 = GetPosition(pContext, MakeIndex(iOrgF, iVert_0));
                if vSrcP == vDstP {
                    let iOffs: c_int = pTriInfos[t as usize].iTSpacesOffs;
                    psTspace[(iOffs + iMissingIndex) as usize] =
                        psTspace[(iOffs + iVert_0) as usize];
                    bNotFound_0 = false;
                } else {
                    i_0 += 1;
                }
            }
            assert!(!bNotFound_0);
        }

        t += 1;
    }
}
