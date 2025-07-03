#![expect(non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use core::ffi::{c_double, c_float, c_int, c_uchar, c_uint, c_ulong, c_void};

use super::math::*;

extern "C" {
    fn memcpy(_: *mut c_void, _: *const c_void, _: c_ulong) -> *mut c_void;
    fn memset(_: *mut c_void, _: c_int, _: c_ulong) -> *mut c_void;
    fn malloc(_: c_ulong) -> *mut c_void;
    fn free(_: *mut c_void);
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SMikkTSpaceContext {
    pub m_pInterface: *mut SMikkTSpaceInterface,
    pub m_pUserData: *mut c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SMikkTSpaceInterface {
    pub m_getNumFaces: Option<unsafe extern "C" fn(*const SMikkTSpaceContext) -> c_int>,
    pub m_getNumVerticesOfFace:
        Option<unsafe extern "C" fn(*const SMikkTSpaceContext, c_int) -> c_int>,
    pub m_getPosition:
        Option<unsafe extern "C" fn(*const SMikkTSpaceContext, *mut c_float, c_int, c_int) -> ()>,
    pub m_getNormal:
        Option<unsafe extern "C" fn(*const SMikkTSpaceContext, *mut c_float, c_int, c_int) -> ()>,
    pub m_getTexCoord:
        Option<unsafe extern "C" fn(*const SMikkTSpaceContext, *mut c_float, c_int, c_int) -> ()>,
    pub m_setTSpaceBasic: Option<
        unsafe extern "C" fn(
            *const SMikkTSpaceContext,
            *const c_float,
            c_float,
            c_int,
            c_int,
        ) -> (),
    >,
    pub m_setTSpace: Option<
        unsafe extern "C" fn(
            *const SMikkTSpaceContext,
            *const c_float,
            *const c_float,
            c_float,
            c_float,
            bool,
            c_int,
            c_int,
        ) -> (),
    >,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct STSpace {
    pub vOs: SVec3,
    pub fMagS: c_float,
    pub vOt: SVec3,
    pub fMagT: c_float,
    pub iCounter: c_int,
    pub bOrient: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct STriInfo {
    pub FaceNeighbors: [c_int; 3],
    pub AssignedGroup: [*mut SGroup; 3],
    pub vOs: SVec3,
    pub vOt: SVec3,
    pub fMagS: c_float,
    pub fMagT: c_float,
    pub iOrgFaceNumber: c_int,
    pub iFlag: c_int,
    pub iTSpacesOffs: c_int,
    pub vert_num: [c_uchar; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SGroup {
    pub iNrFaces: c_int,
    pub pFaceIndices: *mut c_int,
    pub iVertexRepresentitive: c_int,
    pub bOrientPreservering: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SSubGroup {
    pub iNrFaces: c_int,
    pub pTriMembers: *mut c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union SEdge {
    pub c2rust_unnamed: C2RustUnnamed,
    pub array: [c_int; 3],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed {
    pub i0: c_int,
    pub i1: c_int,
    pub f: c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct STmpVert {
    pub vert: [c_float; 3],
    pub index: c_int,
}
pub const NULL: c_int = 0 as c_int;
pub const INTERNAL_RND_SORT_SEED: c_int = 39871946 as c_int;
pub const MARK_DEGENERATE: c_int = 1 as c_int;
pub const QUAD_ONE_DEGEN_TRI: c_int = 2 as c_int;
pub const GROUP_WITH_ANY: c_int = 4 as c_int;
pub const ORIENT_PRESERVING: c_int = 8 as c_int;
unsafe extern "C" fn MakeIndex(iFace: c_int, iVert: c_int) -> c_int {
    assert!(iVert >= 0 as c_int && iVert < 4 as c_int && iFace >= 0 as c_int);
    iFace << 2 as c_int | iVert & 0x3 as c_int
}
unsafe extern "C" fn IndexToData(mut piFace: *mut c_int, mut piVert: *mut c_int, iIndexIn: c_int) {
    *piVert.offset(0 as c_int as isize) = iIndexIn & 0x3 as c_int;
    *piFace.offset(0 as c_int as isize) = iIndexIn >> 2 as c_int;
}
unsafe extern "C" fn AvgTSpace(mut pTS0: *const STSpace, mut pTS1: *const STSpace) -> STSpace {
    let mut ts_res: STSpace = STSpace {
        vOs: SVec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        },
        fMagS: 0.,
        vOt: SVec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        },
        fMagT: 0.,
        iCounter: 0,
        bOrient: false,
    };
    if (*pTS0).fMagS == (*pTS1).fMagS
        && (*pTS0).fMagT == (*pTS1).fMagT
        && ((*pTS0).vOs == (*pTS1).vOs)
        && ((*pTS0).vOt == (*pTS1).vOt)
    {
        ts_res.fMagS = (*pTS0).fMagS;
        ts_res.fMagT = (*pTS0).fMagT;
        ts_res.vOs = (*pTS0).vOs;
        ts_res.vOt = (*pTS0).vOt;
    } else {
        ts_res.fMagS = 0.5f32 * ((*pTS0).fMagS + (*pTS1).fMagS);
        ts_res.fMagT = 0.5f32 * ((*pTS0).fMagT + (*pTS1).fMagT);
        ts_res.vOs = (*pTS0).vOs + (*pTS1).vOs;
        ts_res.vOt = (*pTS0).vOt + (*pTS1).vOt;
        ts_res.vOs.normalize_or_zero();
        ts_res.vOt.normalize_or_zero();
    }
    ts_res
}
pub unsafe extern "C" fn genTangSpaceDefault(mut pContext: *const SMikkTSpaceContext) -> bool {
    genTangSpace(pContext, 180.0f32)
}
pub unsafe extern "C" fn genTangSpace(
    mut pContext: *const SMikkTSpaceContext,
    fAngularThreshold: c_float,
) -> bool {
    let mut piTriListIn: *mut c_int = NULL as *mut c_int;
    let mut piGroupTrianglesBuffer: *mut c_int = NULL as *mut c_int;
    let mut pTriInfos: *mut STriInfo = NULL as *mut STriInfo;
    let mut pGroups: *mut SGroup = NULL as *mut SGroup;
    let mut psTspace: *mut STSpace = NULL as *mut STSpace;
    let mut iNrTrianglesIn: c_int = 0 as c_int;
    let mut f: c_int = 0 as c_int;
    let mut t: c_int = 0 as c_int;
    let mut i: c_int = 0 as c_int;
    let mut iNrTSPaces: c_int = 0 as c_int;
    let mut iTotTris: c_int = 0 as c_int;
    let mut iDegenTriangles: c_int = 0 as c_int;
    let mut iNrMaxGroups: c_int = 0 as c_int;
    let mut iNrActiveGroups: c_int = 0 as c_int;
    let mut index: c_int = 0 as c_int;
    let iNrFaces: c_int =
        ((*(*pContext).m_pInterface).m_getNumFaces).expect("non-null function pointer")(pContext);
    let mut bRes: bool = false;
    let fThresCos: c_float = cos(deg_to_rad(fAngularThreshold) as c_double) as c_float;
    if ((*(*pContext).m_pInterface).m_getNumFaces).is_none()
        || ((*(*pContext).m_pInterface).m_getNumVerticesOfFace).is_none()
        || ((*(*pContext).m_pInterface).m_getPosition).is_none()
        || ((*(*pContext).m_pInterface).m_getNormal).is_none()
        || ((*(*pContext).m_pInterface).m_getTexCoord).is_none()
    {
        return false;
    }
    f = 0 as c_int;
    while f < iNrFaces {
        let verts: c_int = ((*(*pContext).m_pInterface).m_getNumVerticesOfFace)
            .expect("non-null function pointer")(pContext, f);
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
    piTriListIn = malloc(
        (::core::mem::size_of::<c_int>() as c_ulong)
            .wrapping_mul(3 as c_int as c_ulong)
            .wrapping_mul(iNrTrianglesIn as c_ulong),
    ) as *mut c_int;
    pTriInfos = malloc(
        (::core::mem::size_of::<STriInfo>() as c_ulong).wrapping_mul(iNrTrianglesIn as c_ulong),
    ) as *mut STriInfo;
    if piTriListIn.is_null() || pTriInfos.is_null() {
        if !piTriListIn.is_null() {
            free(piTriListIn as *mut c_void);
        }
        if !pTriInfos.is_null() {
            free(pTriInfos as *mut c_void);
        }
        return false;
    }
    iNrTSPaces = GenerateInitialVerticesIndexList(pTriInfos, piTriListIn, pContext, iNrTrianglesIn);
    GenerateSharedVerticesIndexList(piTriListIn, pContext, iNrTrianglesIn);
    iTotTris = iNrTrianglesIn;
    iDegenTriangles = 0 as c_int;
    t = 0 as c_int;
    while t < iTotTris {
        let i0: c_int = *piTriListIn.offset((t * 3 as c_int + 0 as c_int) as isize);
        let i1: c_int = *piTriListIn.offset((t * 3 as c_int + 1 as c_int) as isize);
        let i2: c_int = *piTriListIn.offset((t * 3 as c_int + 2 as c_int) as isize);
        let p0: SVec3 = GetPosition(pContext, i0);
        let p1: SVec3 = GetPosition(pContext, i1);
        let p2: SVec3 = GetPosition(pContext, i2);
        if (p0 == p1) || (p0 == p2) || (p1 == p2) {
            (*pTriInfos.offset(t as isize)).iFlag |= MARK_DEGENERATE;
            iDegenTriangles += 1;
        }
        t += 1;
    }
    iNrTrianglesIn = iTotTris - iDegenTriangles;
    DegenPrologue(pTriInfos, piTriListIn, iNrTrianglesIn, iTotTris);
    InitTriInfo(
        pTriInfos,
        piTriListIn as *const c_int,
        pContext,
        iNrTrianglesIn,
    );
    iNrMaxGroups = iNrTrianglesIn * 3 as c_int;
    pGroups =
        malloc((::core::mem::size_of::<SGroup>() as c_ulong).wrapping_mul(iNrMaxGroups as c_ulong))
            as *mut SGroup;
    piGroupTrianglesBuffer = malloc(
        (::core::mem::size_of::<c_int>() as c_ulong)
            .wrapping_mul(iNrTrianglesIn as c_ulong)
            .wrapping_mul(3 as c_int as c_ulong),
    ) as *mut c_int;
    if pGroups.is_null() || piGroupTrianglesBuffer.is_null() {
        if !pGroups.is_null() {
            free(pGroups as *mut c_void);
        }
        if !piGroupTrianglesBuffer.is_null() {
            free(piGroupTrianglesBuffer as *mut c_void);
        }
        free(piTriListIn as *mut c_void);
        free(pTriInfos as *mut c_void);
        return false;
    }
    iNrActiveGroups = Build4RuleGroups(
        pTriInfos,
        pGroups,
        piGroupTrianglesBuffer,
        piTriListIn as *const c_int,
        iNrTrianglesIn,
    );
    psTspace =
        malloc((::core::mem::size_of::<STSpace>() as c_ulong).wrapping_mul(iNrTSPaces as c_ulong))
            as *mut STSpace;
    if psTspace.is_null() {
        free(piTriListIn as *mut c_void);
        free(pTriInfos as *mut c_void);
        free(pGroups as *mut c_void);
        free(piGroupTrianglesBuffer as *mut c_void);
        return false;
    }
    memset(
        psTspace as *mut c_void,
        0 as c_int,
        (::core::mem::size_of::<STSpace>() as c_ulong).wrapping_mul(iNrTSPaces as c_ulong),
    );
    t = 0 as c_int;
    while t < iNrTSPaces {
        (*psTspace.offset(t as isize)).vOs.x = 1.0f32;
        (*psTspace.offset(t as isize)).vOs.y = 0.0f32;
        (*psTspace.offset(t as isize)).vOs.z = 0.0f32;
        (*psTspace.offset(t as isize)).fMagS = 1.0f32;
        (*psTspace.offset(t as isize)).vOt.x = 0.0f32;
        (*psTspace.offset(t as isize)).vOt.y = 1.0f32;
        (*psTspace.offset(t as isize)).vOt.z = 0.0f32;
        (*psTspace.offset(t as isize)).fMagT = 1.0f32;
        t += 1;
    }
    bRes = GenerateTSpaces(
        psTspace,
        pTriInfos as *const STriInfo,
        pGroups as *const SGroup,
        iNrActiveGroups,
        piTriListIn as *const c_int,
        fThresCos,
        pContext,
    );
    free(pGroups as *mut c_void);
    free(piGroupTrianglesBuffer as *mut c_void);
    if !bRes {
        free(pTriInfos as *mut c_void);
        free(piTriListIn as *mut c_void);
        free(psTspace as *mut c_void);
        return false;
    }
    DegenEpilogue(
        psTspace,
        pTriInfos,
        piTriListIn,
        pContext,
        iNrTrianglesIn,
        iTotTris,
    );
    free(pTriInfos as *mut c_void);
    free(piTriListIn as *mut c_void);
    index = 0 as c_int;
    f = 0 as c_int;
    while f < iNrFaces {
        let verts_0: c_int = ((*(*pContext).m_pInterface).m_getNumVerticesOfFace)
            .expect("non-null function pointer")(pContext, f);
        if !(verts_0 != 3 as c_int && verts_0 != 4 as c_int) {
            i = 0 as c_int;
            while i < verts_0 {
                let mut pTSpace: *const STSpace =
                    &mut *psTspace.offset(index as isize) as *mut STSpace;
                let mut tang: [c_float; 3] = [(*pTSpace).vOs.x, (*pTSpace).vOs.y, (*pTSpace).vOs.z];
                let mut bitang: [c_float; 3] =
                    [(*pTSpace).vOt.x, (*pTSpace).vOt.y, (*pTSpace).vOt.z];
                if ((*(*pContext).m_pInterface).m_setTSpace).is_some() {
                    ((*(*pContext).m_pInterface).m_setTSpace).expect("non-null function pointer")(
                        pContext,
                        tang.as_mut_ptr() as *const c_float,
                        bitang.as_mut_ptr() as *const c_float,
                        (*pTSpace).fMagS,
                        (*pTSpace).fMagT,
                        (*pTSpace).bOrient,
                        f,
                        i,
                    );
                }
                if ((*(*pContext).m_pInterface).m_setTSpaceBasic).is_some() {
                    ((*(*pContext).m_pInterface).m_setTSpaceBasic)
                        .expect("non-null function pointer")(
                        pContext,
                        tang.as_mut_ptr() as *const c_float,
                        if (*pTSpace).bOrient {
                            1.0f32
                        } else {
                            -1.0f32
                        },
                        f,
                        i,
                    );
                }
                index += 1;
                i += 1;
            }
        }
        f += 1;
    }
    free(psTspace as *mut c_void);
    true
}
static mut g_iCells: c_int = 2048 as c_int;
#[inline(never)]
unsafe extern "C" fn FindGridCell(fMin: c_float, fMax: c_float, fVal: c_float) -> c_int {
    let fIndex: c_float = g_iCells as c_float * ((fVal - fMin) / (fMax - fMin));
    let iIndex: c_int = fIndex as c_int;
    if iIndex < g_iCells {
        if iIndex >= 0 as c_int {
            iIndex
        } else {
            0 as c_int
        }
    } else {
        g_iCells - 1 as c_int
    }
}
unsafe extern "C" fn GenerateSharedVerticesIndexList(
    mut piTriList_in_and_out: *mut c_int,
    mut pContext: *const SMikkTSpaceContext,
    iNrTrianglesIn: c_int,
) {
    let mut piHashTable: *mut c_int = NULL as *mut c_int;
    let mut piHashCount: *mut c_int = NULL as *mut c_int;
    let mut piHashOffsets: *mut c_int = NULL as *mut c_int;
    let mut piHashCount2: *mut c_int = NULL as *mut c_int;
    let mut pTmpVert: *mut STmpVert = NULL as *mut STmpVert;
    let mut i: c_int = 0 as c_int;
    let mut iChannel: c_int = 0 as c_int;
    let mut k: c_int = 0 as c_int;
    let mut e: c_int = 0 as c_int;
    let mut iMaxCount: c_int = 0 as c_int;
    let mut vMin: SVec3 = GetPosition(pContext, 0 as c_int);
    let mut vMax: SVec3 = vMin;
    let mut vDim: SVec3 = SVec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    let mut fMin: c_float = 0.;
    let mut fMax: c_float = 0.;
    i = 1 as c_int;
    while i < iNrTrianglesIn * 3 as c_int {
        let index: c_int = *piTriList_in_and_out.offset(i as isize);
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
    vDim = vMax - vMin;
    iChannel = 0 as c_int;
    fMin = vMin.x;
    fMax = vMax.x;
    if vDim.y > vDim.x && vDim.y > vDim.z {
        iChannel = 1 as c_int;
        fMin = vMin.y;
        fMax = vMax.y;
    } else if vDim.z > vDim.x {
        iChannel = 2 as c_int;
        fMin = vMin.z;
        fMax = vMax.z;
    }
    piHashTable = malloc(
        (::core::mem::size_of::<c_int>() as c_ulong)
            .wrapping_mul(iNrTrianglesIn as c_ulong)
            .wrapping_mul(3 as c_int as c_ulong),
    ) as *mut c_int;
    piHashCount =
        malloc((::core::mem::size_of::<c_int>() as c_ulong).wrapping_mul(g_iCells as c_ulong))
            as *mut c_int;
    piHashOffsets =
        malloc((::core::mem::size_of::<c_int>() as c_ulong).wrapping_mul(g_iCells as c_ulong))
            as *mut c_int;
    piHashCount2 =
        malloc((::core::mem::size_of::<c_int>() as c_ulong).wrapping_mul(g_iCells as c_ulong))
            as *mut c_int;
    if piHashTable.is_null()
        || piHashCount.is_null()
        || piHashOffsets.is_null()
        || piHashCount2.is_null()
    {
        if !piHashTable.is_null() {
            free(piHashTable as *mut c_void);
        }
        if !piHashCount.is_null() {
            free(piHashCount as *mut c_void);
        }
        if !piHashOffsets.is_null() {
            free(piHashOffsets as *mut c_void);
        }
        if !piHashCount2.is_null() {
            free(piHashCount2 as *mut c_void);
        }
        GenerateSharedVerticesIndexListSlow(piTriList_in_and_out, pContext, iNrTrianglesIn);
        return;
    }
    memset(
        piHashCount as *mut c_void,
        0 as c_int,
        (::core::mem::size_of::<c_int>() as c_ulong).wrapping_mul(g_iCells as c_ulong),
    );
    memset(
        piHashCount2 as *mut c_void,
        0 as c_int,
        (::core::mem::size_of::<c_int>() as c_ulong).wrapping_mul(g_iCells as c_ulong),
    );
    i = 0 as c_int;
    while i < iNrTrianglesIn * 3 as c_int {
        let index_0: c_int = *piTriList_in_and_out.offset(i as isize);
        let vP_0: SVec3 = GetPosition(pContext, index_0);
        let fVal: c_float = if iChannel == 0 as c_int {
            vP_0.x
        } else if iChannel == 1 as c_int {
            vP_0.y
        } else {
            vP_0.z
        };
        let iCell: c_int = FindGridCell(fMin, fMax, fVal);
        let fresh0 = &mut (*piHashCount.offset(iCell as isize));
        *fresh0 += 1;
        i += 1;
    }
    *piHashOffsets.offset(0 as c_int as isize) = 0 as c_int;
    k = 1 as c_int;
    while k < g_iCells {
        *piHashOffsets.offset(k as isize) = *piHashOffsets.offset((k - 1 as c_int) as isize)
            + *piHashCount.offset((k - 1 as c_int) as isize);
        k += 1;
    }
    i = 0 as c_int;
    while i < iNrTrianglesIn * 3 as c_int {
        let index_1: c_int = *piTriList_in_and_out.offset(i as isize);
        let vP_1: SVec3 = GetPosition(pContext, index_1);
        let fVal_0: c_float = if iChannel == 0 as c_int {
            vP_1.x
        } else if iChannel == 1 as c_int {
            vP_1.y
        } else {
            vP_1.z
        };
        let iCell_0: c_int = FindGridCell(fMin, fMax, fVal_0);
        let mut pTable: *mut c_int = NULL as *mut c_int;
        assert!(*piHashCount2.offset(iCell_0 as isize) < *piHashCount.offset(iCell_0 as isize));
        pTable = &mut *piHashTable.offset(*piHashOffsets.offset(iCell_0 as isize) as isize)
            as *mut c_int;
        *pTable.offset(*piHashCount2.offset(iCell_0 as isize) as isize) = i;
        let fresh1 = &mut (*piHashCount2.offset(iCell_0 as isize));
        *fresh1 += 1;
        i += 1;
    }
    k = 0 as c_int;
    while k < g_iCells {
        assert!(*piHashCount2.offset(k as isize) == *piHashCount.offset(k as isize));
        k += 1;
    }
    free(piHashCount2 as *mut c_void);
    iMaxCount = *piHashCount.offset(0 as c_int as isize);
    k = 1 as c_int;
    while k < g_iCells {
        if iMaxCount < *piHashCount.offset(k as isize) {
            iMaxCount = *piHashCount.offset(k as isize);
        }
        k += 1;
    }
    pTmpVert =
        malloc((::core::mem::size_of::<STmpVert>() as c_ulong).wrapping_mul(iMaxCount as c_ulong))
            as *mut STmpVert;
    k = 0 as c_int;
    while k < g_iCells {
        let mut pTable_0: *mut c_int =
            &mut *piHashTable.offset(*piHashOffsets.offset(k as isize) as isize) as *mut c_int;
        let iEntries: c_int = *piHashCount.offset(k as isize);
        if iEntries >= 2 as c_int {
            if !pTmpVert.is_null() {
                e = 0 as c_int;
                while e < iEntries {
                    let mut i_0: c_int = *pTable_0.offset(e as isize);
                    let vP_2: SVec3 =
                        GetPosition(pContext, *piTriList_in_and_out.offset(i_0 as isize));
                    (*pTmpVert.offset(e as isize)).vert[0 as c_int as usize] = vP_2.x;
                    (*pTmpVert.offset(e as isize)).vert[1 as c_int as usize] = vP_2.y;
                    (*pTmpVert.offset(e as isize)).vert[2 as c_int as usize] = vP_2.z;
                    (*pTmpVert.offset(e as isize)).index = i_0;
                    e += 1;
                }
                MergeVertsFast(
                    piTriList_in_and_out,
                    pTmpVert,
                    pContext,
                    0 as c_int,
                    iEntries - 1 as c_int,
                );
            } else {
                MergeVertsSlow(
                    piTriList_in_and_out,
                    pContext,
                    pTable_0 as *const c_int,
                    iEntries,
                );
            }
        }
        k += 1;
    }
    if !pTmpVert.is_null() {
        free(pTmpVert as *mut c_void);
    }
    free(piHashTable as *mut c_void);
    free(piHashCount as *mut c_void);
    free(piHashOffsets as *mut c_void);
}
unsafe extern "C" fn MergeVertsFast(
    mut piTriList_in_and_out: *mut c_int,
    mut pTmpVert: *mut STmpVert,
    mut pContext: *const SMikkTSpaceContext,
    iL_in: c_int,
    iR_in: c_int,
) {
    let mut c: c_int = 0 as c_int;
    let mut l: c_int = 0 as c_int;
    let mut channel: c_int = 0 as c_int;
    let mut fvMin: [c_float; 3] = [0.; 3];
    let mut fvMax: [c_float; 3] = [0.; 3];
    let mut dx: c_float = 0 as c_int as c_float;
    let mut dy: c_float = 0 as c_int as c_float;
    let mut dz: c_float = 0 as c_int as c_float;
    let mut fSep: c_float = 0 as c_int as c_float;
    c = 0 as c_int;
    while c < 3 as c_int {
        fvMin[c as usize] = (*pTmpVert.offset(iL_in as isize)).vert[c as usize];
        fvMax[c as usize] = fvMin[c as usize];
        c += 1;
    }
    l = iL_in + 1 as c_int;
    while l <= iR_in {
        c = 0 as c_int;
        while c < 3 as c_int {
            if fvMin[c as usize] > (*pTmpVert.offset(l as isize)).vert[c as usize] {
                fvMin[c as usize] = (*pTmpVert.offset(l as isize)).vert[c as usize];
            }
            if fvMax[c as usize] < (*pTmpVert.offset(l as isize)).vert[c as usize] {
                fvMax[c as usize] = (*pTmpVert.offset(l as isize)).vert[c as usize];
            }
            c += 1;
        }
        l += 1;
    }
    dx = fvMax[0 as c_int as usize] - fvMin[0 as c_int as usize];
    dy = fvMax[1 as c_int as usize] - fvMin[1 as c_int as usize];
    dz = fvMax[2 as c_int as usize] - fvMin[2 as c_int as usize];
    channel = 0 as c_int;
    if dy > dx && dy > dz {
        channel = 1 as c_int;
    } else if dz > dx {
        channel = 2 as c_int;
    }
    fSep = 0.5f32 * (fvMax[channel as usize] + fvMin[channel as usize]);
    if fSep.is_finite() as i32 == 0 {
        return;
    }
    if fSep >= fvMax[channel as usize] || fSep <= fvMin[channel as usize] {
        l = iL_in;
        while l <= iR_in {
            let mut i: c_int = (*pTmpVert.offset(l as isize)).index;
            let index: c_int = *piTriList_in_and_out.offset(i as isize);
            let vP: SVec3 = GetPosition(pContext, index);
            let vN: SVec3 = GetNormal(pContext, index);
            let vT: SVec3 = GetTexCoord(pContext, index);
            let mut bNotFound: bool = true;
            let mut l2: c_int = iL_in;
            let mut i2rec: c_int = -(1 as c_int);
            while l2 < l && bNotFound {
                let i2: c_int = (*pTmpVert.offset(l2 as isize)).index;
                let index2: c_int = *piTriList_in_and_out.offset(i2 as isize);
                let vP2: SVec3 = GetPosition(pContext, index2);
                let vN2: SVec3 = GetNormal(pContext, index2);
                let vT2: SVec3 = GetTexCoord(pContext, index2);
                i2rec = i2;
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
            if !bNotFound {
                *piTriList_in_and_out.offset(i as isize) =
                    *piTriList_in_and_out.offset(i2rec as isize);
            }
            l += 1;
        }
    } else {
        let mut iL: c_int = iL_in;
        let mut iR: c_int = iR_in;
        assert!(iR_in - iL_in > 0 as c_int);
        while iL < iR {
            let mut bReadyLeftSwap: bool = false;
            let mut bReadyRightSwap: bool = false;
            while !bReadyLeftSwap && iL < iR {
                assert!(iL >= iL_in && iL <= iR_in);
                #[expect(clippy::neg_cmp_op_on_partial_ord)]
                {
                    bReadyLeftSwap =
                        !((*pTmpVert.offset(iL as isize)).vert[channel as usize] < fSep);
                }
                if !bReadyLeftSwap {
                    iL += 1;
                }
            }
            while !bReadyRightSwap && iL < iR {
                assert!(iR >= iL_in && iR <= iR_in);
                bReadyRightSwap = (*pTmpVert.offset(iR as isize)).vert[channel as usize] < fSep;
                if !bReadyRightSwap {
                    iR -= 1;
                }
            }
            assert!(iL < iR || !(bReadyLeftSwap && bReadyRightSwap));
            if bReadyLeftSwap && bReadyRightSwap {
                let sTmp: STmpVert = *pTmpVert.offset(iL as isize);
                assert!(iL < iR);
                *pTmpVert.offset(iL as isize) = *pTmpVert.offset(iR as isize);
                *pTmpVert.offset(iR as isize) = sTmp;
                iL += 1;
                iR -= 1;
            }
        }
        assert!(iL == iR + 1 as c_int || iL == iR);
        if iL == iR {
            let bReadyRightSwap_0: bool =
                (*pTmpVert.offset(iR as isize)).vert[channel as usize] < fSep;
            if bReadyRightSwap_0 {
                iL += 1;
            } else {
                iR -= 1;
            }
        }
        if iL_in < iR {
            MergeVertsFast(piTriList_in_and_out, pTmpVert, pContext, iL_in, iR);
        }
        if iL < iR_in {
            MergeVertsFast(piTriList_in_and_out, pTmpVert, pContext, iL, iR_in);
        }
    };
}
unsafe extern "C" fn MergeVertsSlow(
    mut piTriList_in_and_out: *mut c_int,
    mut pContext: *const SMikkTSpaceContext,
    mut pTable: *const c_int,
    iEntries: c_int,
) {
    let mut e: c_int = 0 as c_int;
    e = 0 as c_int;
    while e < iEntries {
        let mut i: c_int = *pTable.offset(e as isize);
        let index: c_int = *piTriList_in_and_out.offset(i as isize);
        let vP: SVec3 = GetPosition(pContext, index);
        let vN: SVec3 = GetNormal(pContext, index);
        let vT: SVec3 = GetTexCoord(pContext, index);
        let mut bNotFound: bool = true;
        let mut e2: c_int = 0 as c_int;
        let mut i2rec: c_int = -(1 as c_int);
        while e2 < e && bNotFound {
            let i2: c_int = *pTable.offset(e2 as isize);
            let index2: c_int = *piTriList_in_and_out.offset(i2 as isize);
            let vP2: SVec3 = GetPosition(pContext, index2);
            let vN2: SVec3 = GetNormal(pContext, index2);
            let vT2: SVec3 = GetTexCoord(pContext, index2);
            i2rec = i2;
            if (vP == vP2) && (vN == vN2) && (vT == vT2) {
                bNotFound = false;
            } else {
                e2 += 1;
            }
        }
        if !bNotFound {
            *piTriList_in_and_out.offset(i as isize) = *piTriList_in_and_out.offset(i2rec as isize);
        }
        e += 1;
    }
}
unsafe extern "C" fn GenerateSharedVerticesIndexListSlow(
    mut piTriList_in_and_out: *mut c_int,
    mut pContext: *const SMikkTSpaceContext,
    iNrTrianglesIn: c_int,
) {
    let mut t: c_int = 0 as c_int;
    let mut i: c_int = 0 as c_int;
    t = 0 as c_int;
    while t < iNrTrianglesIn {
        i = 0 as c_int;
        while i < 3 as c_int {
            let offs: c_int = t * 3 as c_int + i;
            let index: c_int = *piTriList_in_and_out.offset(offs as isize);
            let vP: SVec3 = GetPosition(pContext, index);
            let vN: SVec3 = GetNormal(pContext, index);
            let vT: SVec3 = GetTexCoord(pContext, index);
            let mut bFound: bool = false;
            let mut t2: c_int = 0 as c_int;
            let mut index2rec: c_int = -(1 as c_int);
            while !bFound && t2 <= t {
                let mut j: c_int = 0 as c_int;
                while !bFound && j < 3 as c_int {
                    let index2: c_int =
                        *piTriList_in_and_out.offset((t2 * 3 as c_int + j) as isize);
                    let vP2: SVec3 = GetPosition(pContext, index2);
                    let vN2: SVec3 = GetNormal(pContext, index2);
                    let vT2: SVec3 = GetTexCoord(pContext, index2);
                    if (vP == vP2) && (vN == vN2) && (vT == vT2) {
                        bFound = true;
                    } else {
                        j += 1;
                    }
                }
                if !bFound {
                    t2 += 1;
                }
            }
            assert!(bFound);
            *piTriList_in_and_out.offset(offs as isize) = index2rec;
            i += 1;
        }
        t += 1;
    }
}
unsafe extern "C" fn GenerateInitialVerticesIndexList(
    mut pTriInfos: *mut STriInfo,
    mut piTriList_out: *mut c_int,
    mut pContext: *const SMikkTSpaceContext,
    iNrTrianglesIn: c_int,
) -> c_int {
    let mut iTSpacesOffs: c_int = 0 as c_int;
    let mut f: c_int = 0 as c_int;
    let mut t: c_int = 0 as c_int;
    let mut iDstTriIndex: c_int = 0 as c_int;
    f = 0 as c_int;
    while f
        < ((*(*pContext).m_pInterface).m_getNumFaces).expect("non-null function pointer")(pContext)
    {
        let verts: c_int = ((*(*pContext).m_pInterface).m_getNumVerticesOfFace)
            .expect("non-null function pointer")(pContext, f);
        if !(verts != 3 as c_int && verts != 4 as c_int) {
            (*pTriInfos.offset(iDstTriIndex as isize)).iOrgFaceNumber = f;
            (*pTriInfos.offset(iDstTriIndex as isize)).iTSpacesOffs = iTSpacesOffs;
            if verts == 3 as c_int {
                let mut pVerts: *mut c_uchar =
                    ((*pTriInfos.offset(iDstTriIndex as isize)).vert_num).as_mut_ptr();
                *pVerts.offset(0 as c_int as isize) = 0 as c_int as c_uchar;
                *pVerts.offset(1 as c_int as isize) = 1 as c_int as c_uchar;
                *pVerts.offset(2 as c_int as isize) = 2 as c_int as c_uchar;
                *piTriList_out.offset((iDstTriIndex * 3 as c_int + 0 as c_int) as isize) =
                    MakeIndex(f, 0 as c_int);
                *piTriList_out.offset((iDstTriIndex * 3 as c_int + 1 as c_int) as isize) =
                    MakeIndex(f, 1 as c_int);
                *piTriList_out.offset((iDstTriIndex * 3 as c_int + 2 as c_int) as isize) =
                    MakeIndex(f, 2 as c_int);
                iDstTriIndex += 1;
            } else {
                (*pTriInfos.offset((iDstTriIndex + 1 as c_int) as isize)).iOrgFaceNumber = f;
                (*pTriInfos.offset((iDstTriIndex + 1 as c_int) as isize)).iTSpacesOffs =
                    iTSpacesOffs;
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
                let mut bQuadDiagIs_02: bool = false;
                if distSQ_02 < distSQ_13 {
                    bQuadDiagIs_02 = true;
                } else if distSQ_13 < distSQ_02 {
                    bQuadDiagIs_02 = false;
                } else {
                    let P0: SVec3 = GetPosition(pContext, i0);
                    let P1: SVec3 = GetPosition(pContext, i1);
                    let P2: SVec3 = GetPosition(pContext, i2);
                    let P3: SVec3 = GetPosition(pContext, i3);
                    let distSQ_02_0: c_float = (P2 - P0).length_squared();
                    let distSQ_13_0: c_float = (P3 - P1).length_squared();
                    bQuadDiagIs_02 = distSQ_13_0 >= distSQ_02_0;
                }
                if bQuadDiagIs_02 {
                    let mut pVerts_A: *mut c_uchar =
                        ((*pTriInfos.offset(iDstTriIndex as isize)).vert_num).as_mut_ptr();
                    *pVerts_A.offset(0 as c_int as isize) = 0 as c_int as c_uchar;
                    *pVerts_A.offset(1 as c_int as isize) = 1 as c_int as c_uchar;
                    *pVerts_A.offset(2 as c_int as isize) = 2 as c_int as c_uchar;
                    *piTriList_out.offset((iDstTriIndex * 3 as c_int + 0 as c_int) as isize) = i0;
                    *piTriList_out.offset((iDstTriIndex * 3 as c_int + 1 as c_int) as isize) = i1;
                    *piTriList_out.offset((iDstTriIndex * 3 as c_int + 2 as c_int) as isize) = i2;
                    iDstTriIndex += 1;
                    let mut pVerts_B: *mut c_uchar =
                        ((*pTriInfos.offset(iDstTriIndex as isize)).vert_num).as_mut_ptr();
                    *pVerts_B.offset(0 as c_int as isize) = 0 as c_int as c_uchar;
                    *pVerts_B.offset(1 as c_int as isize) = 2 as c_int as c_uchar;
                    *pVerts_B.offset(2 as c_int as isize) = 3 as c_int as c_uchar;
                    *piTriList_out.offset((iDstTriIndex * 3 as c_int + 0 as c_int) as isize) = i0;
                    *piTriList_out.offset((iDstTriIndex * 3 as c_int + 1 as c_int) as isize) = i2;
                    *piTriList_out.offset((iDstTriIndex * 3 as c_int + 2 as c_int) as isize) = i3;
                    iDstTriIndex += 1;
                } else {
                    let mut pVerts_A_0: *mut c_uchar =
                        ((*pTriInfos.offset(iDstTriIndex as isize)).vert_num).as_mut_ptr();
                    *pVerts_A_0.offset(0 as c_int as isize) = 0 as c_int as c_uchar;
                    *pVerts_A_0.offset(1 as c_int as isize) = 1 as c_int as c_uchar;
                    *pVerts_A_0.offset(2 as c_int as isize) = 3 as c_int as c_uchar;
                    *piTriList_out.offset((iDstTriIndex * 3 as c_int + 0 as c_int) as isize) = i0;
                    *piTriList_out.offset((iDstTriIndex * 3 as c_int + 1 as c_int) as isize) = i1;
                    *piTriList_out.offset((iDstTriIndex * 3 as c_int + 2 as c_int) as isize) = i3;
                    iDstTriIndex += 1;
                    let mut pVerts_B_0: *mut c_uchar =
                        ((*pTriInfos.offset(iDstTriIndex as isize)).vert_num).as_mut_ptr();
                    *pVerts_B_0.offset(0 as c_int as isize) = 1 as c_int as c_uchar;
                    *pVerts_B_0.offset(1 as c_int as isize) = 2 as c_int as c_uchar;
                    *pVerts_B_0.offset(2 as c_int as isize) = 3 as c_int as c_uchar;
                    *piTriList_out.offset((iDstTriIndex * 3 as c_int + 0 as c_int) as isize) = i1;
                    *piTriList_out.offset((iDstTriIndex * 3 as c_int + 1 as c_int) as isize) = i2;
                    *piTriList_out.offset((iDstTriIndex * 3 as c_int + 2 as c_int) as isize) = i3;
                    iDstTriIndex += 1;
                }
            }
            iTSpacesOffs += verts;
            assert!(iDstTriIndex <= iNrTrianglesIn);
        }
        f += 1;
    }
    t = 0 as c_int;
    while t < iNrTrianglesIn {
        (*pTriInfos.offset(t as isize)).iFlag = 0 as c_int;
        t += 1;
    }
    iTSpacesOffs
}
unsafe extern "C" fn GetPosition(mut pContext: *const SMikkTSpaceContext, index: c_int) -> SVec3 {
    let mut iF: c_int = 0;
    let mut iI: c_int = 0;
    let mut res: SVec3 = SVec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    let mut pos: [c_float; 3] = [0.; 3];
    IndexToData(&mut iF, &mut iI, index);
    ((*(*pContext).m_pInterface).m_getPosition).expect("non-null function pointer")(
        pContext,
        pos.as_mut_ptr(),
        iF,
        iI,
    );
    res.x = pos[0 as c_int as usize];
    res.y = pos[1 as c_int as usize];
    res.z = pos[2 as c_int as usize];
    res
}
unsafe extern "C" fn GetNormal(mut pContext: *const SMikkTSpaceContext, index: c_int) -> SVec3 {
    let mut iF: c_int = 0;
    let mut iI: c_int = 0;
    let mut res: SVec3 = SVec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    let mut norm: [c_float; 3] = [0.; 3];
    IndexToData(&mut iF, &mut iI, index);
    ((*(*pContext).m_pInterface).m_getNormal).expect("non-null function pointer")(
        pContext,
        norm.as_mut_ptr(),
        iF,
        iI,
    );
    res.x = norm[0 as c_int as usize];
    res.y = norm[1 as c_int as usize];
    res.z = norm[2 as c_int as usize];
    res
}
unsafe extern "C" fn GetTexCoord(mut pContext: *const SMikkTSpaceContext, index: c_int) -> SVec3 {
    let mut iF: c_int = 0;
    let mut iI: c_int = 0;
    let mut res: SVec3 = SVec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    let mut texc: [c_float; 2] = [0.; 2];
    IndexToData(&mut iF, &mut iI, index);
    ((*(*pContext).m_pInterface).m_getTexCoord).expect("non-null function pointer")(
        pContext,
        texc.as_mut_ptr(),
        iF,
        iI,
    );
    res.x = texc[0 as c_int as usize];
    res.y = texc[1 as c_int as usize];
    res.z = 1.0f32;
    res
}
unsafe extern "C" fn CalcTexArea(
    mut pContext: *const SMikkTSpaceContext,
    mut indices: *const c_int,
) -> c_float {
    let t1: SVec3 = GetTexCoord(pContext, *indices.offset(0 as c_int as isize));
    let t2: SVec3 = GetTexCoord(pContext, *indices.offset(1 as c_int as isize));
    let t3: SVec3 = GetTexCoord(pContext, *indices.offset(2 as c_int as isize));
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
unsafe extern "C" fn InitTriInfo(
    mut pTriInfos: *mut STriInfo,
    mut piTriListIn: *const c_int,
    mut pContext: *const SMikkTSpaceContext,
    iNrTrianglesIn: c_int,
) {
    let mut f: c_int = 0 as c_int;
    let mut i: c_int = 0 as c_int;
    let mut t: c_int = 0 as c_int;
    f = 0 as c_int;
    while f < iNrTrianglesIn {
        i = 0 as c_int;
        while i < 3 as c_int {
            (*pTriInfos.offset(f as isize)).FaceNeighbors[i as usize] = -(1 as c_int);
            let fresh2 = &mut (*pTriInfos.offset(f as isize)).AssignedGroup[i as usize];
            *fresh2 = NULL as *mut SGroup;
            (*pTriInfos.offset(f as isize)).vOs.x = 0.0f32;
            (*pTriInfos.offset(f as isize)).vOs.y = 0.0f32;
            (*pTriInfos.offset(f as isize)).vOs.z = 0.0f32;
            (*pTriInfos.offset(f as isize)).vOt.x = 0.0f32;
            (*pTriInfos.offset(f as isize)).vOt.y = 0.0f32;
            (*pTriInfos.offset(f as isize)).vOt.z = 0.0f32;
            (*pTriInfos.offset(f as isize)).fMagS = 0 as c_int as c_float;
            (*pTriInfos.offset(f as isize)).fMagT = 0 as c_int as c_float;
            (*pTriInfos.offset(f as isize)).iFlag |= GROUP_WITH_ANY;
            i += 1;
        }
        f += 1;
    }
    f = 0 as c_int;
    while f < iNrTrianglesIn {
        let v1: SVec3 = GetPosition(
            pContext,
            *piTriListIn.offset((f * 3 as c_int + 0 as c_int) as isize),
        );
        let v2: SVec3 = GetPosition(
            pContext,
            *piTriListIn.offset((f * 3 as c_int + 1 as c_int) as isize),
        );
        let v3: SVec3 = GetPosition(
            pContext,
            *piTriListIn.offset((f * 3 as c_int + 2 as c_int) as isize),
        );
        let t1: SVec3 = GetTexCoord(
            pContext,
            *piTriListIn.offset((f * 3 as c_int + 0 as c_int) as isize),
        );
        let t2: SVec3 = GetTexCoord(
            pContext,
            *piTriListIn.offset((f * 3 as c_int + 1 as c_int) as isize),
        );
        let t3: SVec3 = GetTexCoord(
            pContext,
            *piTriListIn.offset((f * 3 as c_int + 2 as c_int) as isize),
        );
        let t21x: c_float = t2.x - t1.x;
        let t21y: c_float = t2.y - t1.y;
        let t31x: c_float = t3.x - t1.x;
        let t31y: c_float = t3.y - t1.y;
        let d1: SVec3 = v2 - v1;
        let d2: SVec3 = v3 - v1;
        let fSignedAreaSTx2: c_float = t21x * t31y - t21y * t31x;
        let mut vOs: SVec3 = (t31y * d1) - (t21y * d2);
        let mut vOt: SVec3 = (-t31x * d1) + (t21x * d2);
        (*pTriInfos.offset(f as isize)).iFlag |= if fSignedAreaSTx2 > 0 as c_int as c_float {
            ORIENT_PRESERVING
        } else {
            0 as c_int
        };
        if not_zero(fSignedAreaSTx2) {
            let fAbsArea: c_float = fabsf(fSignedAreaSTx2);
            let fLenOs: c_float = vOs.length();
            let fLenOt: c_float = vOt.length();
            let fS: c_float =
                if (*pTriInfos.offset(f as isize)).iFlag & ORIENT_PRESERVING == 0 as c_int {
                    -1.0f32
                } else {
                    1.0f32
                };
            if not_zero(fLenOs) {
                (*pTriInfos.offset(f as isize)).vOs = (fS / fLenOs) * vOs;
            }
            if not_zero(fLenOt) {
                (*pTriInfos.offset(f as isize)).vOt = (fS / fLenOt) * vOt;
            }
            (*pTriInfos.offset(f as isize)).fMagS = fLenOs / fAbsArea;
            (*pTriInfos.offset(f as isize)).fMagT = fLenOt / fAbsArea;
            if not_zero((*pTriInfos.offset(f as isize)).fMagS)
                && not_zero((*pTriInfos.offset(f as isize)).fMagT)
            {
                (*pTriInfos.offset(f as isize)).iFlag &= !GROUP_WITH_ANY;
            }
        }
        f += 1;
    }
    while t < iNrTrianglesIn - 1 as c_int {
        let iFO_a: c_int = (*pTriInfos.offset(t as isize)).iOrgFaceNumber;
        let iFO_b: c_int = (*pTriInfos.offset((t + 1 as c_int) as isize)).iOrgFaceNumber;
        if iFO_a == iFO_b {
            let bIsDeg_a: bool =
                (*pTriInfos.offset(t as isize)).iFlag & MARK_DEGENERATE != 0 as c_int;
            let bIsDeg_b: bool = (*pTriInfos.offset((t + 1 as c_int) as isize)).iFlag
                & MARK_DEGENERATE != 0 as c_int;
            if !(bIsDeg_a || bIsDeg_b) {
                let bOrientA: bool =
                    (*pTriInfos.offset(t as isize)).iFlag & ORIENT_PRESERVING != 0 as c_int;
                let bOrientB: bool = (*pTriInfos.offset((t + 1 as c_int) as isize)).iFlag
                    & ORIENT_PRESERVING != 0 as c_int;
                if bOrientA != bOrientB {
                    let mut bChooseOrientFirstTri: bool = false;
                    #[expect(clippy::if_same_then_else)]
                    if (*pTriInfos.offset((t + 1 as c_int) as isize)).iFlag & GROUP_WITH_ANY
                        != 0 as c_int
                    {
                        bChooseOrientFirstTri = true;
                    } else if CalcTexArea(
                        pContext,
                        &*piTriListIn.offset((t * 3 as c_int + 0 as c_int) as isize),
                    ) >= CalcTexArea(
                        pContext,
                        &*piTriListIn.offset(((t + 1 as c_int) * 3 as c_int + 0 as c_int) as isize),
                    ) {
                        bChooseOrientFirstTri = true;
                    }
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
                    (*pTriInfos.offset(t1_0 as isize)).iFlag &= !ORIENT_PRESERVING;
                    (*pTriInfos.offset(t1_0 as isize)).iFlag |=
                        (*pTriInfos.offset(t0 as isize)).iFlag & ORIENT_PRESERVING;
                }
            }
            t += 2 as c_int;
        } else {
            t += 1;
        }
    }
    let mut pEdges: *mut SEdge = malloc(
        (::core::mem::size_of::<SEdge>() as c_ulong)
            .wrapping_mul(iNrTrianglesIn as c_ulong)
            .wrapping_mul(3 as c_int as c_ulong),
    ) as *mut SEdge;
    if pEdges.is_null() {
        BuildNeighborsSlow(pTriInfos, piTriListIn, iNrTrianglesIn);
    } else {
        BuildNeighborsFast(pTriInfos, pEdges, piTriListIn, iNrTrianglesIn);
        free(pEdges as *mut c_void);
    };
}
unsafe extern "C" fn Build4RuleGroups(
    mut pTriInfos: *mut STriInfo,
    mut pGroups: *mut SGroup,
    mut piGroupTrianglesBuffer: *mut c_int,
    mut piTriListIn: *const c_int,
    iNrTrianglesIn: c_int,
) -> c_int {
    let iNrMaxGroups: c_int = iNrTrianglesIn * 3 as c_int;
    let mut iNrActiveGroups: c_int = 0 as c_int;
    let mut iOffset: c_int = 0 as c_int;
    let mut f: c_int = 0 as c_int;
    let mut i: c_int = 0 as c_int;
    f = 0 as c_int;
    while f < iNrTrianglesIn {
        i = 0 as c_int;
        while i < 3 as c_int {
            if (*pTriInfos.offset(f as isize)).iFlag & GROUP_WITH_ANY == 0 as c_int
                && ((*pTriInfos.offset(f as isize)).AssignedGroup[i as usize]).is_null()
            {
                let mut bOrPre: bool = false;
                let mut neigh_indexL: c_int = 0;
                let mut neigh_indexR: c_int = 0;
                let vert_index: c_int = *piTriListIn.offset((f * 3 as c_int + i) as isize);
                assert!(iNrActiveGroups < iNrMaxGroups);
                let fresh3 = &mut (*pTriInfos.offset(f as isize)).AssignedGroup[i as usize];
                *fresh3 = &mut *pGroups.offset(iNrActiveGroups as isize) as *mut SGroup;
                (*(*pTriInfos.offset(f as isize)).AssignedGroup[i as usize])
                    .iVertexRepresentitive = vert_index;
                (*(*pTriInfos.offset(f as isize)).AssignedGroup[i as usize]).bOrientPreservering =
                    (*pTriInfos.offset(f as isize)).iFlag & ORIENT_PRESERVING != 0 as c_int;
                (*(*pTriInfos.offset(f as isize)).AssignedGroup[i as usize]).iNrFaces = 0 as c_int;
                let fresh4 =
                    &mut (*(*pTriInfos.offset(f as isize)).AssignedGroup[i as usize]).pFaceIndices;
                *fresh4 = &mut *piGroupTrianglesBuffer.offset(iOffset as isize) as *mut c_int;
                iNrActiveGroups += 1;
                AddTriToGroup((*pTriInfos.offset(f as isize)).AssignedGroup[i as usize], f);
                bOrPre = (*pTriInfos.offset(f as isize)).iFlag & ORIENT_PRESERVING != 0 as c_int;
                neigh_indexL = (*pTriInfos.offset(f as isize)).FaceNeighbors[i as usize];
                neigh_indexR = (*pTriInfos.offset(f as isize)).FaceNeighbors[(if i > 0 as c_int {
                    i - 1 as c_int
                } else {
                    2 as c_int
                })
                    as usize];
                if neigh_indexL >= 0 as c_int {
                    let bAnswer: bool = AssignRecur(
                        piTriListIn,
                        pTriInfos,
                        neigh_indexL,
                        (*pTriInfos.offset(f as isize)).AssignedGroup[i as usize],
                    );
                    let bOrPre2: bool = (*pTriInfos.offset(neigh_indexL as isize)).iFlag
                        & ORIENT_PRESERVING != 0 as c_int;
                    let bDiff: bool = bOrPre != bOrPre2;
                    assert!(bAnswer || bDiff);
                }
                if neigh_indexR >= 0 as c_int {
                    let bAnswer_0: bool = AssignRecur(
                        piTriListIn,
                        pTriInfos,
                        neigh_indexR,
                        (*pTriInfos.offset(f as isize)).AssignedGroup[i as usize],
                    );
                    let bOrPre2_0: bool = (*pTriInfos.offset(neigh_indexR as isize)).iFlag
                        & ORIENT_PRESERVING != 0 as c_int;
                    let bDiff_0: bool = bOrPre != bOrPre2_0;
                    assert!(bAnswer_0 || bDiff_0);
                }
                iOffset += (*(*pTriInfos.offset(f as isize)).AssignedGroup[i as usize]).iNrFaces;
                assert!(iOffset <= iNrMaxGroups);
            }
            i += 1;
        }
        f += 1;
    }
    iNrActiveGroups
}
unsafe extern "C" fn AddTriToGroup(mut pGroup: *mut SGroup, iTriIndex: c_int) {
    *((*pGroup).pFaceIndices).offset((*pGroup).iNrFaces as isize) = iTriIndex;
    (*pGroup).iNrFaces += 1;
}
unsafe extern "C" fn AssignRecur(
    mut piTriListIn: *const c_int,
    mut psTriInfos: *mut STriInfo,
    iMyTriIndex: c_int,
    mut pGroup: *mut SGroup,
) -> bool {
    let mut pMyTriInfo: *mut STriInfo =
        &mut *psTriInfos.offset(iMyTriIndex as isize) as *mut STriInfo;
    let iVertRep: c_int = (*pGroup).iVertexRepresentitive;
    let mut pVerts: *const c_int =
        &*piTriListIn.offset((3 as c_int * iMyTriIndex + 0 as c_int) as isize) as *const c_int;
    let mut i: c_int = -(1 as c_int);
    if *pVerts.offset(0 as c_int as isize) == iVertRep {
        i = 0 as c_int;
    } else if *pVerts.offset(1 as c_int as isize) == iVertRep {
        i = 1 as c_int;
    } else if *pVerts.offset(2 as c_int as isize) == iVertRep {
        i = 2 as c_int;
    }
    assert!(i >= 0 as c_int && i < 3 as c_int);
    if (*pMyTriInfo).AssignedGroup[i as usize] == pGroup {
        return true;
    } else if !((*pMyTriInfo).AssignedGroup[i as usize]).is_null() {
        return false;
    }
    if (*pMyTriInfo).iFlag & GROUP_WITH_ANY != 0 as c_int
        && ((*pMyTriInfo).AssignedGroup[0 as c_int as usize]).is_null()
        && ((*pMyTriInfo).AssignedGroup[1 as c_int as usize]).is_null()
        && ((*pMyTriInfo).AssignedGroup[2 as c_int as usize]).is_null()
    {
        (*pMyTriInfo).iFlag &= !ORIENT_PRESERVING;
        (*pMyTriInfo).iFlag |= if (*pGroup).bOrientPreservering {
            ORIENT_PRESERVING
        } else {
            0 as c_int
        };
    }
    let bOrient: bool = (*pMyTriInfo).iFlag & ORIENT_PRESERVING != 0 as c_int;
    if bOrient != (*pGroup).bOrientPreservering {
        return false;
    }
    AddTriToGroup(pGroup, iMyTriIndex);
    (*pMyTriInfo).AssignedGroup[i as usize] = pGroup;
    let neigh_indexL: c_int = (*pMyTriInfo).FaceNeighbors[i as usize];
    let neigh_indexR: c_int = (*pMyTriInfo).FaceNeighbors[(if i > 0 as c_int {
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
unsafe extern "C" fn GenerateTSpaces(
    mut psTspace: *mut STSpace,
    mut pTriInfos: *const STriInfo,
    mut pGroups: *const SGroup,
    iNrActiveGroups: c_int,
    mut piTriListIn: *const c_int,
    fThresCos: c_float,
    mut pContext: *const SMikkTSpaceContext,
) -> bool {
    let mut pSubGroupTspace: *mut STSpace = NULL as *mut STSpace;
    let mut pUniSubGroups: *mut SSubGroup = NULL as *mut SSubGroup;
    let mut pTmpMembers: *mut c_int = NULL as *mut c_int;
    let mut iMaxNrFaces: c_int = 0 as c_int;
    let mut g: c_int = 0 as c_int;
    let mut i: c_int = 0 as c_int;
    g = 0 as c_int;
    while g < iNrActiveGroups {
        if iMaxNrFaces < (*pGroups.offset(g as isize)).iNrFaces {
            iMaxNrFaces = (*pGroups.offset(g as isize)).iNrFaces;
        }
        g += 1;
    }
    if iMaxNrFaces == 0 as c_int {
        return true;
    }
    pSubGroupTspace =
        malloc((::core::mem::size_of::<STSpace>() as c_ulong).wrapping_mul(iMaxNrFaces as c_ulong))
            as *mut STSpace;
    pUniSubGroups = malloc(
        (::core::mem::size_of::<SSubGroup>() as c_ulong).wrapping_mul(iMaxNrFaces as c_ulong),
    ) as *mut SSubGroup;
    pTmpMembers =
        malloc((::core::mem::size_of::<c_int>() as c_ulong).wrapping_mul(iMaxNrFaces as c_ulong))
            as *mut c_int;
    if pSubGroupTspace.is_null() || pUniSubGroups.is_null() || pTmpMembers.is_null() {
        if !pSubGroupTspace.is_null() {
            free(pSubGroupTspace as *mut c_void);
        }
        if !pUniSubGroups.is_null() {
            free(pUniSubGroups as *mut c_void);
        }
        if !pTmpMembers.is_null() {
            free(pTmpMembers as *mut c_void);
        }
        return false;
    }
    g = 0 as c_int;
    while g < iNrActiveGroups {
        let mut pGroup: *const SGroup = &*pGroups.offset(g as isize) as *const SGroup;
        let mut iUniqueSubGroups: c_int = 0 as c_int;
        let mut s: c_int = 0 as c_int;
        i = 0 as c_int;
        while i < (*pGroup).iNrFaces {
            let f: c_int = *((*pGroup).pFaceIndices).offset(i as isize);
            let mut index: c_int = -(1 as c_int);
            let mut iVertIndex: c_int = -(1 as c_int);
            let mut iOF_1: c_int = -(1 as c_int);
            let mut iMembers: c_int = 0 as c_int;
            let mut j: c_int = 0 as c_int;
            let mut l: c_int = 0 as c_int;
            let mut tmp_group: SSubGroup = SSubGroup {
                iNrFaces: 0,
                pTriMembers: core::ptr::null_mut::<c_int>(),
            };
            let mut bFound: bool = false;
            let mut n: SVec3 = SVec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            };
            let mut vOs: SVec3 = SVec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            };
            let mut vOt: SVec3 = SVec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            };
            if (*pTriInfos.offset(f as isize)).AssignedGroup[0 as c_int as usize]
                == pGroup as *mut SGroup
            {
                index = 0 as c_int;
            } else if (*pTriInfos.offset(f as isize)).AssignedGroup[1 as c_int as usize]
                == pGroup as *mut SGroup
            {
                index = 1 as c_int;
            } else if (*pTriInfos.offset(f as isize)).AssignedGroup[2 as c_int as usize]
                == pGroup as *mut SGroup
            {
                index = 2 as c_int;
            }
            assert!(index >= 0 as c_int && index < 3 as c_int);
            iVertIndex = *piTriListIn.offset((f * 3 as c_int + index) as isize);
            assert!(iVertIndex == (*pGroup).iVertexRepresentitive);
            n = GetNormal(pContext, iVertIndex);
            vOs = (*pTriInfos.offset(f as isize)).vOs
                - ((n.dot((*pTriInfos.offset(f as isize)).vOs)) * n);
            vOt = (*pTriInfos.offset(f as isize)).vOt
                - ((n.dot((*pTriInfos.offset(f as isize)).vOt)) * n);
            vOs.normalize_or_zero();
            vOt.normalize_or_zero();
            iOF_1 = (*pTriInfos.offset(f as isize)).iOrgFaceNumber;
            iMembers = 0 as c_int;
            j = 0 as c_int;
            while j < (*pGroup).iNrFaces {
                let t: c_int = *((*pGroup).pFaceIndices).offset(j as isize);
                let iOF_2: c_int = (*pTriInfos.offset(t as isize)).iOrgFaceNumber;
                let mut vOs2: SVec3 = (*pTriInfos.offset(t as isize)).vOs
                    - ((n.dot((*pTriInfos.offset(t as isize)).vOs)) * n);
                let mut vOt2: SVec3 = (*pTriInfos.offset(t as isize)).vOt
                    - ((n.dot((*pTriInfos.offset(t as isize)).vOt)) * n);
                vOs2.normalize_or_zero();
                vOt2.normalize_or_zero();
                let bAny: bool = ((*pTriInfos.offset(f as isize)).iFlag
                    | (*pTriInfos.offset(t as isize)).iFlag)
                    & GROUP_WITH_ANY != 0 as c_int;
                let bSameOrgFace: bool = iOF_1 == iOF_2;
                let fCosS: c_float = vOs.dot(vOs2);
                let fCosT: c_float = vOt.dot(vOt2);
                assert!(f != t || bSameOrgFace);
                if bAny || bSameOrgFace || fCosS > fThresCos && fCosT > fThresCos
                {
                    let fresh5 = iMembers;
                    iMembers += 1;
                    *pTmpMembers.offset(fresh5 as isize) = t;
                }
                j += 1;
            }
            tmp_group.iNrFaces = iMembers;
            tmp_group.pTriMembers = pTmpMembers;
            if iMembers > 1 as c_int {
                let mut uSeed: c_uint = INTERNAL_RND_SORT_SEED as c_uint;
                QuickSort(pTmpMembers, 0 as c_int, iMembers - 1 as c_int, uSeed);
            }
            bFound = false;
            l = 0 as c_int;
            while l < iUniqueSubGroups && !bFound {
                bFound = CompareSubGroups(&tmp_group, &*pUniSubGroups.offset(l as isize));
                if !bFound {
                    l += 1;
                }
            }
            assert!(bFound || l == iUniqueSubGroups);
            if !bFound {
                let mut pIndices: *mut c_int = malloc(
                    (::core::mem::size_of::<c_int>() as c_ulong).wrapping_mul(iMembers as c_ulong),
                ) as *mut c_int;
                if pIndices.is_null() {
                    let mut s_0: c_int = 0 as c_int;
                    s_0 = 0 as c_int;
                    while s_0 < iUniqueSubGroups {
                        free((*pUniSubGroups.offset(s_0 as isize)).pTriMembers as *mut c_void);
                        s_0 += 1;
                    }
                    free(pUniSubGroups as *mut c_void);
                    free(pTmpMembers as *mut c_void);
                    free(pSubGroupTspace as *mut c_void);
                    return false;
                }
                (*pUniSubGroups.offset(iUniqueSubGroups as isize)).iNrFaces = iMembers;
                let fresh6 = &mut (*pUniSubGroups.offset(iUniqueSubGroups as isize)).pTriMembers;
                *fresh6 = pIndices;
                memcpy(
                    pIndices as *mut c_void,
                    tmp_group.pTriMembers as *const c_void,
                    (iMembers as c_ulong).wrapping_mul(::core::mem::size_of::<c_int>() as c_ulong),
                );
                *pSubGroupTspace.offset(iUniqueSubGroups as isize) = EvalTspace(
                    tmp_group.pTriMembers,
                    iMembers,
                    piTriListIn,
                    pTriInfos,
                    pContext,
                    (*pGroup).iVertexRepresentitive,
                );
                iUniqueSubGroups += 1;
            }
            let iOffs: c_int = (*pTriInfos.offset(f as isize)).iTSpacesOffs;
            let iVert: c_int = (*pTriInfos.offset(f as isize)).vert_num[index as usize] as c_int;
            let mut pTS_out: *mut STSpace =
                &mut *psTspace.offset((iOffs + iVert) as isize) as *mut STSpace;
            assert!((*pTS_out).iCounter < 2 as c_int);
            assert!(
                ((*pTriInfos.offset(f as isize)).iFlag & 8 as c_int != 0 as c_int)
                    == (*pGroup).bOrientPreservering
            );
            if (*pTS_out).iCounter == 1 as c_int {
                *pTS_out = AvgTSpace(pTS_out, &*pSubGroupTspace.offset(l as isize));
                (*pTS_out).iCounter = 2 as c_int;
                (*pTS_out).bOrient = (*pGroup).bOrientPreservering;
            } else {
                assert!((*pTS_out).iCounter == 0 as c_int);
                *pTS_out = *pSubGroupTspace.offset(l as isize);
                (*pTS_out).iCounter = 1 as c_int;
                (*pTS_out).bOrient = (*pGroup).bOrientPreservering;
            }
            i += 1;
        }
        s = 0 as c_int;
        while s < iUniqueSubGroups {
            free((*pUniSubGroups.offset(s as isize)).pTriMembers as *mut c_void);
            s += 1;
        }
        g += 1;
    }
    free(pUniSubGroups as *mut c_void);
    free(pTmpMembers as *mut c_void);
    free(pSubGroupTspace as *mut c_void);
    true
}
unsafe extern "C" fn EvalTspace(
    mut face_indices: *mut c_int,
    iFaces: c_int,
    mut piTriListIn: *const c_int,
    mut pTriInfos: *const STriInfo,
    mut pContext: *const SMikkTSpaceContext,
    iVertexRepresentitive: c_int,
) -> STSpace {
    let mut res: STSpace = STSpace {
        vOs: SVec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        },
        fMagS: 0.,
        vOt: SVec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        },
        fMagT: 0.,
        iCounter: 0,
        bOrient: false,
    };
    let mut fAngleSum: c_float = 0 as c_int as c_float;
    let mut face: c_int = 0 as c_int;
    res.vOs.x = 0.0f32;
    res.vOs.y = 0.0f32;
    res.vOs.z = 0.0f32;
    res.vOt.x = 0.0f32;
    res.vOt.y = 0.0f32;
    res.vOt.z = 0.0f32;
    res.fMagS = 0 as c_int as c_float;
    res.fMagT = 0 as c_int as c_float;
    face = 0 as c_int;
    while face < iFaces {
        let f: c_int = *face_indices.offset(face as isize);
        if (*pTriInfos.offset(f as isize)).iFlag & GROUP_WITH_ANY == 0 as c_int {
            let mut n: SVec3 = SVec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            };
            let mut vOs: SVec3 = SVec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            };
            let mut vOt: SVec3 = SVec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            };
            let mut p0: SVec3 = SVec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            };
            let mut p1: SVec3 = SVec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            };
            let mut p2: SVec3 = SVec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            };
            let mut v1: SVec3 = SVec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            };
            let mut v2: SVec3 = SVec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            };
            let mut fCos: c_float = 0.;
            let mut fAngle: c_float = 0.;
            let mut fMagS: c_float = 0.;
            let mut fMagT: c_float = 0.;
            let mut i: c_int = -(1 as c_int);
            let mut index: c_int = -(1 as c_int);
            let mut i0: c_int = -(1 as c_int);
            let mut i1: c_int = -(1 as c_int);
            let mut i2: c_int = -(1 as c_int);
            if *piTriListIn.offset((3 as c_int * f + 0 as c_int) as isize) == iVertexRepresentitive
            {
                i = 0 as c_int;
            } else if *piTriListIn.offset((3 as c_int * f + 1 as c_int) as isize)
                == iVertexRepresentitive
            {
                i = 1 as c_int;
            } else if *piTriListIn.offset((3 as c_int * f + 2 as c_int) as isize)
                == iVertexRepresentitive
            {
                i = 2 as c_int;
            }
            assert!(i >= 0 as c_int && i < 3 as c_int);
            index = *piTriListIn.offset((3 as c_int * f + i) as isize);
            n = GetNormal(pContext, index);
            vOs = (*pTriInfos.offset(f as isize)).vOs
                - ((n.dot((*pTriInfos.offset(f as isize)).vOs)) * n);
            vOt = (*pTriInfos.offset(f as isize)).vOt
                - ((n.dot((*pTriInfos.offset(f as isize)).vOt)) * n);
            vOs.normalize_or_zero();
            vOt.normalize_or_zero();
            i2 = *piTriListIn.offset(
                (3 as c_int * f
                    + (if i < 2 as c_int {
                        i + 1 as c_int
                    } else {
                        0 as c_int
                    })) as isize,
            );
            i1 = *piTriListIn.offset((3 as c_int * f + i) as isize);
            i0 = *piTriListIn.offset(
                (3 as c_int * f
                    + (if i > 0 as c_int {
                        i - 1 as c_int
                    } else {
                        2 as c_int
                    })) as isize,
            );
            p0 = GetPosition(pContext, i0);
            p1 = GetPosition(pContext, i1);
            p2 = GetPosition(pContext, i2);
            v1 = p0 - p1;
            v2 = p2 - p1;
            v1 = v1 - ((n.dot(v1)) * n);
            v1.normalize_or_zero();
            v2 = v2 - ((n.dot(v2)) * n);
            v2.normalize_or_zero();
            fCos = v1.dot(v2);
            fCos = if fCos > 1 as c_int as c_float {
                1 as c_int as c_float
            } else if fCos < -(1 as c_int) as c_float {
                -(1 as c_int) as c_float
            } else {
                fCos
            };
            fAngle = acos(fCos as c_double) as c_float;
            fMagS = (*pTriInfos.offset(f as isize)).fMagS;
            fMagT = (*pTriInfos.offset(f as isize)).fMagT;
            res.vOs = res.vOs + (fAngle * vOs);
            res.vOt = res.vOt + (fAngle * vOt);
            res.fMagS += fAngle * fMagS;
            res.fMagT += fAngle * fMagT;
            fAngleSum += fAngle;
        }
        face += 1;
    }
    res.vOs.normalize_or_zero();
    res.vOt.normalize_or_zero();
    if fAngleSum > 0 as c_int as c_float {
        res.fMagS /= fAngleSum;
        res.fMagT /= fAngleSum;
    }
    res
}
unsafe extern "C" fn CompareSubGroups(
    mut pg1: *const SSubGroup,
    mut pg2: *const SSubGroup,
) -> bool {
    let mut bStillSame: bool = true;
    let mut i: c_int = 0 as c_int;
    if (*pg1).iNrFaces != (*pg2).iNrFaces {
        return false;
    }
    while i < (*pg1).iNrFaces && bStillSame {
        bStillSame = *((*pg1).pTriMembers).offset(i as isize) == *((*pg2).pTriMembers).offset(i as isize);
        if bStillSame {
            i += 1;
        }
    }
    bStillSame
}
unsafe extern "C" fn QuickSort(
    mut pSortBuffer: *mut c_int,
    mut iLeft: c_int,
    mut iRight: c_int,
    mut uSeed: c_uint,
) {
    let mut iL: c_int = 0;
    let mut iR: c_int = 0;
    let mut n: c_int = 0;
    let mut index: c_int = 0;
    let mut iMid: c_int = 0;
    let mut iTmp: c_int = 0;
    let mut t: c_uint = uSeed & 31 as c_int as c_uint;
    t = uSeed.wrapping_shl(t) | uSeed.wrapping_shr((32 as c_int as c_uint).wrapping_sub(t));
    uSeed = uSeed.wrapping_add(t).wrapping_add(3 as c_int as c_uint);
    iL = iLeft;
    iR = iRight;
    n = iR - iL + 1 as c_int;
    assert!(n >= 0 as c_int);
    index = uSeed.wrapping_rem(n as c_uint) as c_int;
    iMid = *pSortBuffer.offset((index + iL) as isize);
    loop {
        while *pSortBuffer.offset(iL as isize) < iMid {
            iL += 1;
        }
        while *pSortBuffer.offset(iR as isize) > iMid {
            iR -= 1;
        }
        if iL <= iR {
            iTmp = *pSortBuffer.offset(iL as isize);
            *pSortBuffer.offset(iL as isize) = *pSortBuffer.offset(iR as isize);
            *pSortBuffer.offset(iR as isize) = iTmp;
            iL += 1;
            iR -= 1;
        }
        if iL > iR {
            break;
        }
    }
    if iLeft < iR {
        QuickSort(pSortBuffer, iLeft, iR, uSeed);
    }
    if iL < iRight {
        QuickSort(pSortBuffer, iL, iRight, uSeed);
    }
}
unsafe extern "C" fn BuildNeighborsFast(
    mut pTriInfos: *mut STriInfo,
    mut pEdges: *mut SEdge,
    mut piTriListIn: *const c_int,
    iNrTrianglesIn: c_int,
) {
    let mut uSeed: c_uint = INTERNAL_RND_SORT_SEED as c_uint;
    let mut iEntries: c_int = 0 as c_int;
    let mut iCurStartIndex: c_int = -(1 as c_int);
    let mut f: c_int = 0 as c_int;
    let mut i: c_int = 0 as c_int;
    f = 0 as c_int;
    while f < iNrTrianglesIn {
        i = 0 as c_int;
        while i < 3 as c_int {
            let i0: c_int = *piTriListIn.offset((f * 3 as c_int + i) as isize);
            let i1: c_int = *piTriListIn.offset(
                (f * 3 as c_int
                    + (if i < 2 as c_int {
                        i + 1 as c_int
                    } else {
                        0 as c_int
                    })) as isize,
            );
            (*pEdges.offset((f * 3 as c_int + i) as isize))
                .c2rust_unnamed
                .i0 = if i0 < i1 { i0 } else { i1 };
            (*pEdges.offset((f * 3 as c_int + i) as isize))
                .c2rust_unnamed
                .i1 = if i0 >= i1 { i0 } else { i1 };
            (*pEdges.offset((f * 3 as c_int + i) as isize))
                .c2rust_unnamed
                .f = f;
            i += 1;
        }
        f += 1;
    }
    QuickSortEdges(
        pEdges,
        0 as c_int,
        iNrTrianglesIn * 3 as c_int - 1 as c_int,
        0 as c_int,
        uSeed,
    );
    iEntries = iNrTrianglesIn * 3 as c_int;
    iCurStartIndex = 0 as c_int;
    i = 1 as c_int;
    while i < iEntries {
        if (*pEdges.offset(iCurStartIndex as isize)).c2rust_unnamed.i0
            != (*pEdges.offset(i as isize)).c2rust_unnamed.i0
        {
            let iL: c_int = iCurStartIndex;
            let iR: c_int = i - 1 as c_int;
            iCurStartIndex = i;
            QuickSortEdges(pEdges, iL, iR, 1 as c_int, uSeed);
        }
        i += 1;
    }
    iCurStartIndex = 0 as c_int;
    i = 1 as c_int;
    while i < iEntries {
        if (*pEdges.offset(iCurStartIndex as isize)).c2rust_unnamed.i0
            != (*pEdges.offset(i as isize)).c2rust_unnamed.i0
            || (*pEdges.offset(iCurStartIndex as isize)).c2rust_unnamed.i1
                != (*pEdges.offset(i as isize)).c2rust_unnamed.i1
        {
            let iL_0: c_int = iCurStartIndex;
            let iR_0: c_int = i - 1 as c_int;
            iCurStartIndex = i;
            QuickSortEdges(pEdges, iL_0, iR_0, 2 as c_int, uSeed);
        }
        i += 1;
    }
    i = 0 as c_int;
    while i < iEntries {
        let i0_0: c_int = (*pEdges.offset(i as isize)).c2rust_unnamed.i0;
        let i1_0: c_int = (*pEdges.offset(i as isize)).c2rust_unnamed.i1;
        let f_0: c_int = (*pEdges.offset(i as isize)).c2rust_unnamed.f;
        let mut bUnassigned_A: bool = false;
        let mut i0_A: c_int = 0;
        let mut i1_A: c_int = 0;
        let mut edgenum_A: c_int = 0;
        let mut edgenum_B: c_int = 0 as c_int;
        GetEdge(
            &mut i0_A,
            &mut i1_A,
            &mut edgenum_A,
            &*piTriListIn.offset((f_0 * 3 as c_int) as isize),
            i0_0,
            i1_0,
        );
        bUnassigned_A = (*pTriInfos.offset(f_0 as isize)).FaceNeighbors[edgenum_A as usize] == -(1 as c_int);
        if bUnassigned_A {
            let mut j: c_int = i + 1 as c_int;
            let mut t: c_int = 0;
            let mut bNotFound: bool = true;
            while j < iEntries
                && i0_0 == (*pEdges.offset(j as isize)).c2rust_unnamed.i0
                && i1_0 == (*pEdges.offset(j as isize)).c2rust_unnamed.i1
                && bNotFound
            {
                let mut bUnassigned_B: bool = false;
                let mut i0_B: c_int = 0;
                let mut i1_B: c_int = 0;
                t = (*pEdges.offset(j as isize)).c2rust_unnamed.f;
                GetEdge(
                    &mut i1_B,
                    &mut i0_B,
                    &mut edgenum_B,
                    &*piTriListIn.offset((t * 3 as c_int) as isize),
                    (*pEdges.offset(j as isize)).c2rust_unnamed.i0,
                    (*pEdges.offset(j as isize)).c2rust_unnamed.i1,
                );
                bUnassigned_B = (*pTriInfos.offset(t as isize)).FaceNeighbors[edgenum_B as usize] == -(1 as c_int);
                if i0_A == i0_B && i1_A == i1_B && bUnassigned_B {
                    bNotFound = false;
                } else {
                    j += 1;
                }
            }
            if !bNotFound {
                let mut t_0: c_int = (*pEdges.offset(j as isize)).c2rust_unnamed.f;
                (*pTriInfos.offset(f_0 as isize)).FaceNeighbors[edgenum_A as usize] = t_0;
                (*pTriInfos.offset(t_0 as isize)).FaceNeighbors[edgenum_B as usize] = f_0;
            }
        }
        i += 1;
    }
}
unsafe extern "C" fn BuildNeighborsSlow(
    mut pTriInfos: *mut STriInfo,
    mut piTriListIn: *const c_int,
    iNrTrianglesIn: c_int,
) {
    let mut f: c_int = 0 as c_int;
    let mut i: c_int = 0 as c_int;
    f = 0 as c_int;
    while f < iNrTrianglesIn {
        i = 0 as c_int;
        while i < 3 as c_int {
            if (*pTriInfos.offset(f as isize)).FaceNeighbors[i as usize] == -(1 as c_int) {
                let i0_A: c_int = *piTriListIn.offset((f * 3 as c_int + i) as isize);
                let i1_A: c_int = *piTriListIn.offset(
                    (f * 3 as c_int
                        + (if i < 2 as c_int {
                            i + 1 as c_int
                        } else {
                            0 as c_int
                        })) as isize,
                );
                let mut bFound: bool = false;
                let mut t: c_int = 0 as c_int;
                let mut j: c_int = 0 as c_int;
                while !bFound && t < iNrTrianglesIn {
                    if t != f {
                        j = 0 as c_int;
                        while !bFound && j < 3 as c_int {
                            let i1_B: c_int = *piTriListIn.offset((t * 3 as c_int + j) as isize);
                            let i0_B: c_int = *piTriListIn.offset(
                                (t * 3 as c_int
                                    + (if j < 2 as c_int {
                                        j + 1 as c_int
                                    } else {
                                        0 as c_int
                                    })) as isize,
                            );
                            if i0_A == i0_B && i1_A == i1_B {
                                bFound = true;
                            } else {
                                j += 1;
                            }
                        }
                    }
                    if !bFound {
                        t += 1;
                    }
                }
                if bFound {
                    (*pTriInfos.offset(f as isize)).FaceNeighbors[i as usize] = t;
                    (*pTriInfos.offset(t as isize)).FaceNeighbors[j as usize] = f;
                }
            }
            i += 1;
        }
        f += 1;
    }
}
unsafe extern "C" fn QuickSortEdges(
    mut pSortBuffer: *mut SEdge,
    mut iLeft: c_int,
    mut iRight: c_int,
    channel: c_int,
    mut uSeed: c_uint,
) {
    let mut t: c_uint = 0;
    let mut iL: c_int = 0;
    let mut iR: c_int = 0;
    let mut n: c_int = 0;
    let mut index: c_int = 0;
    let mut iMid: c_int = 0;
    let mut sTmp: SEdge = SEdge {
        c2rust_unnamed: C2RustUnnamed { i0: 0, i1: 0, f: 0 },
    };
    let iElems: c_int = iRight - iLeft + 1 as c_int;
    #[expect(clippy::comparison_chain)]
    if iElems < 2 as c_int {
        return;
    } else if iElems == 2 as c_int {
        if (*pSortBuffer.offset(iLeft as isize)).array[channel as usize]
            > (*pSortBuffer.offset(iRight as isize)).array[channel as usize]
        {
            sTmp = *pSortBuffer.offset(iLeft as isize);
            *pSortBuffer.offset(iLeft as isize) = *pSortBuffer.offset(iRight as isize);
            *pSortBuffer.offset(iRight as isize) = sTmp;
        }
        return;
    }
    t = uSeed & 31 as c_int as c_uint;
    t = uSeed.wrapping_shl(t) | uSeed.wrapping_shr(((32 as c_int as c_uint).wrapping_sub(t)));
    uSeed = uSeed.wrapping_add(t).wrapping_add(3 as c_int as c_uint);
    iL = iLeft;
    iR = iRight;
    n = iR - iL + 1 as c_int;
    assert!(n >= 0 as c_int);
    index = uSeed.wrapping_rem(n as c_uint) as c_int;
    iMid = (*pSortBuffer.offset((index + iL) as isize)).array[channel as usize];
    loop {
        while (*pSortBuffer.offset(iL as isize)).array[channel as usize] < iMid {
            iL += 1;
        }
        while (*pSortBuffer.offset(iR as isize)).array[channel as usize] > iMid {
            iR -= 1;
        }
        if iL <= iR {
            sTmp = *pSortBuffer.offset(iL as isize);
            *pSortBuffer.offset(iL as isize) = *pSortBuffer.offset(iR as isize);
            *pSortBuffer.offset(iR as isize) = sTmp;
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
unsafe extern "C" fn GetEdge(
    mut i0_out: *mut c_int,
    mut i1_out: *mut c_int,
    mut edgenum_out: *mut c_int,
    mut indices: *const c_int,
    i0_in: c_int,
    i1_in: c_int,
) {
    *edgenum_out = -(1 as c_int);
    if *indices.offset(0 as c_int as isize) == i0_in
        || *indices.offset(0 as c_int as isize) == i1_in
    {
        if *indices.offset(1 as c_int as isize) == i0_in
            || *indices.offset(1 as c_int as isize) == i1_in
        {
            *edgenum_out.offset(0 as c_int as isize) = 0 as c_int;
            *i0_out.offset(0 as c_int as isize) = *indices.offset(0 as c_int as isize);
            *i1_out.offset(0 as c_int as isize) = *indices.offset(1 as c_int as isize);
        } else {
            *edgenum_out.offset(0 as c_int as isize) = 2 as c_int;
            *i0_out.offset(0 as c_int as isize) = *indices.offset(2 as c_int as isize);
            *i1_out.offset(0 as c_int as isize) = *indices.offset(0 as c_int as isize);
        }
    } else {
        *edgenum_out.offset(0 as c_int as isize) = 1 as c_int;
        *i0_out.offset(0 as c_int as isize) = *indices.offset(1 as c_int as isize);
        *i1_out.offset(0 as c_int as isize) = *indices.offset(2 as c_int as isize);
    };
}
unsafe extern "C" fn DegenPrologue(
    mut pTriInfos: *mut STriInfo,
    mut piTriList_out: *mut c_int,
    iNrTrianglesIn: c_int,
    iTotTris: c_int,
) {
    let mut iNextGoodTriangleSearchIndex: c_int = -(1 as c_int);
    let mut bStillFindingGoodOnes: bool = false;
    let mut t: c_int = 0 as c_int;
    while t < iTotTris - 1 as c_int {
        let iFO_a: c_int = (*pTriInfos.offset(t as isize)).iOrgFaceNumber;
        let iFO_b: c_int = (*pTriInfos.offset((t + 1 as c_int) as isize)).iOrgFaceNumber;
        if iFO_a == iFO_b {
            let bIsDeg_a: bool =
                (*pTriInfos.offset(t as isize)).iFlag & MARK_DEGENERATE != 0 as c_int;
            let bIsDeg_b: bool = (*pTriInfos.offset((t + 1 as c_int) as isize)).iFlag
                & MARK_DEGENERATE != 0 as c_int;
            if bIsDeg_a ^ bIsDeg_b {
                (*pTriInfos.offset(t as isize)).iFlag |= QUAD_ONE_DEGEN_TRI;
                (*pTriInfos.offset((t + 1 as c_int) as isize)).iFlag |= QUAD_ONE_DEGEN_TRI;
            }
            t += 2 as c_int;
        } else {
            t += 1;
        }
    }
    iNextGoodTriangleSearchIndex = 1 as c_int;
    t = 0 as c_int;
    bStillFindingGoodOnes = true;
    while t < iNrTrianglesIn && bStillFindingGoodOnes {
        let bIsGood: bool = (*pTriInfos.offset(t as isize)).iFlag & MARK_DEGENERATE == 0 as c_int;
        if bIsGood {
            if iNextGoodTriangleSearchIndex < t + 2 as c_int {
                iNextGoodTriangleSearchIndex = t + 2 as c_int;
            }
        } else {
            let mut t0: c_int = 0;
            let mut t1: c_int = 0;
            let mut bJustADegenerate: bool = true;
            while bJustADegenerate && iNextGoodTriangleSearchIndex < iTotTris {
                let bIsGood_0: bool = (*pTriInfos.offset(iNextGoodTriangleSearchIndex as isize))
                    .iFlag
                    & MARK_DEGENERATE == 0 as c_int;
                if bIsGood_0 {
                    bJustADegenerate = false;
                } else {
                    iNextGoodTriangleSearchIndex += 1;
                }
            }
            t0 = t;
            t1 = iNextGoodTriangleSearchIndex;
            iNextGoodTriangleSearchIndex += 1;
            assert!(iNextGoodTriangleSearchIndex > t + 1 as c_int);
            if !bJustADegenerate {
                let mut i: c_int = 0 as c_int;
                i = 0 as c_int;
                while i < 3 as c_int {
                    let index: c_int = *piTriList_out.offset((t0 * 3 as c_int + i) as isize);
                    *piTriList_out.offset((t0 * 3 as c_int + i) as isize) =
                        *piTriList_out.offset((t1 * 3 as c_int + i) as isize);
                    *piTriList_out.offset((t1 * 3 as c_int + i) as isize) = index;
                    i += 1;
                }
                let tri_info: STriInfo = *pTriInfos.offset(t0 as isize);
                *pTriInfos.offset(t0 as isize) = *pTriInfos.offset(t1 as isize);
                *pTriInfos.offset(t1 as isize) = tri_info;
            } else {
                bStillFindingGoodOnes = false;
            }
        }
        if bStillFindingGoodOnes {
            t += 1;
        }
    }
    assert!(bStillFindingGoodOnes);
    assert!(iNrTrianglesIn == t);
}
unsafe extern "C" fn DegenEpilogue(
    mut psTspace: *mut STSpace,
    mut pTriInfos: *mut STriInfo,
    mut piTriListIn: *mut c_int,
    mut pContext: *const SMikkTSpaceContext,
    iNrTrianglesIn: c_int,
    iTotTris: c_int,
) {
    let mut t: c_int = 0 as c_int;
    let mut i: c_int = 0 as c_int;
    t = iNrTrianglesIn;
    while t < iTotTris {
        let bSkip: bool =
            (*pTriInfos.offset(t as isize)).iFlag & QUAD_ONE_DEGEN_TRI != 0 as c_int;
        if !bSkip {
            i = 0 as c_int;
            while i < 3 as c_int {
                let index1: c_int = *piTriListIn.offset((t * 3 as c_int + i) as isize);
                let mut bNotFound: bool = true;
                let mut j: c_int = 0 as c_int;
                while bNotFound && j < 3 as c_int * iNrTrianglesIn {
                    let index2: c_int = *piTriListIn.offset(j as isize);
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
                        (*pTriInfos.offset(iTri as isize)).vert_num[iVert as usize] as c_int;
                    let iSrcOffs: c_int = (*pTriInfos.offset(iTri as isize)).iTSpacesOffs;
                    let iDstVert: c_int =
                        (*pTriInfos.offset(t as isize)).vert_num[i as usize] as c_int;
                    let iDstOffs: c_int = (*pTriInfos.offset(t as isize)).iTSpacesOffs;
                    *psTspace.offset((iDstOffs + iDstVert) as isize) =
                        *psTspace.offset((iSrcOffs + iSrcVert) as isize);
                }
                i += 1;
            }
        }
        t += 1;
    }
    t = 0 as c_int;
    while t < iNrTrianglesIn {
        if (*pTriInfos.offset(t as isize)).iFlag & QUAD_ONE_DEGEN_TRI != 0 as c_int {
            let mut vDstP: SVec3 = SVec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            };
            let mut iOrgF: c_int = -(1 as c_int);
            let mut i_0: c_int = 0 as c_int;
            let mut bNotFound_0: bool = false;
            let mut pV: *mut c_uchar = ((*pTriInfos.offset(t as isize)).vert_num).as_mut_ptr();
            let mut iFlag: c_int = (1 as c_int) << *pV.offset(0 as c_int as isize) as c_int
                | (1 as c_int) << *pV.offset(1 as c_int as isize) as c_int
                | (1 as c_int) << *pV.offset(2 as c_int as isize) as c_int;
            let mut iMissingIndex: c_int = 0 as c_int;
            if iFlag & 2 as c_int == 0 as c_int {
                iMissingIndex = 1 as c_int;
            } else if iFlag & 4 as c_int == 0 as c_int {
                iMissingIndex = 2 as c_int;
            } else if iFlag & 8 as c_int == 0 as c_int {
                iMissingIndex = 3 as c_int;
            }
            iOrgF = (*pTriInfos.offset(t as isize)).iOrgFaceNumber;
            vDstP = GetPosition(pContext, MakeIndex(iOrgF, iMissingIndex));
            bNotFound_0 = true;
            i_0 = 0 as c_int;
            while bNotFound_0 && i_0 < 3 as c_int {
                let iVert_0: c_int = *pV.offset(i_0 as isize) as c_int;
                let vSrcP: SVec3 = GetPosition(pContext, MakeIndex(iOrgF, iVert_0));
                if vSrcP == vDstP {
                    let iOffs: c_int = (*pTriInfos.offset(t as isize)).iTSpacesOffs;
                    *psTspace.offset((iOffs + iMissingIndex) as isize) =
                        *psTspace.offset((iOffs + iVert_0) as isize);
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
