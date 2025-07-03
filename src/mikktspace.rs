#![expect(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]

use super::{libc, math::*};

extern "C" {
    fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
    fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong) -> *mut libc::c_void;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
}
pub type tbool = libc::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SMikkTSpaceContext {
    pub m_pInterface: *mut SMikkTSpaceInterface,
    pub m_pUserData: *mut libc::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SMikkTSpaceInterface {
    pub m_getNumFaces: Option<unsafe extern "C" fn(*const SMikkTSpaceContext) -> libc::c_int>,
    pub m_getNumVerticesOfFace:
        Option<unsafe extern "C" fn(*const SMikkTSpaceContext, libc::c_int) -> libc::c_int>,
    pub m_getPosition: Option<
        unsafe extern "C" fn(
            *const SMikkTSpaceContext,
            *mut libc::c_float,
            libc::c_int,
            libc::c_int,
        ) -> (),
    >,
    pub m_getNormal: Option<
        unsafe extern "C" fn(
            *const SMikkTSpaceContext,
            *mut libc::c_float,
            libc::c_int,
            libc::c_int,
        ) -> (),
    >,
    pub m_getTexCoord: Option<
        unsafe extern "C" fn(
            *const SMikkTSpaceContext,
            *mut libc::c_float,
            libc::c_int,
            libc::c_int,
        ) -> (),
    >,
    pub m_setTSpaceBasic: Option<
        unsafe extern "C" fn(
            *const SMikkTSpaceContext,
            *const libc::c_float,
            libc::c_float,
            libc::c_int,
            libc::c_int,
        ) -> (),
    >,
    pub m_setTSpace: Option<
        unsafe extern "C" fn(
            *const SMikkTSpaceContext,
            *const libc::c_float,
            *const libc::c_float,
            libc::c_float,
            libc::c_float,
            tbool,
            libc::c_int,
            libc::c_int,
        ) -> (),
    >,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct STSpace {
    pub vOs: SVec3,
    pub fMagS: libc::c_float,
    pub vOt: SVec3,
    pub fMagT: libc::c_float,
    pub iCounter: libc::c_int,
    pub bOrient: tbool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct STriInfo {
    pub FaceNeighbors: [libc::c_int; 3],
    pub AssignedGroup: [*mut SGroup; 3],
    pub vOs: SVec3,
    pub vOt: SVec3,
    pub fMagS: libc::c_float,
    pub fMagT: libc::c_float,
    pub iOrgFaceNumber: libc::c_int,
    pub iFlag: libc::c_int,
    pub iTSpacesOffs: libc::c_int,
    pub vert_num: [libc::c_uchar; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SGroup {
    pub iNrFaces: libc::c_int,
    pub pFaceIndices: *mut libc::c_int,
    pub iVertexRepresentitive: libc::c_int,
    pub bOrientPreservering: tbool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SSubGroup {
    pub iNrFaces: libc::c_int,
    pub pTriMembers: *mut libc::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union SEdge {
    pub c2rust_unnamed: C2RustUnnamed,
    pub array: [libc::c_int; 3],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed {
    pub i0: libc::c_int,
    pub i1: libc::c_int,
    pub f: libc::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct STmpVert {
    pub vert: [libc::c_float; 3],
    pub index: libc::c_int,
}
pub const NULL: libc::c_int = 0 as libc::c_int;
pub const TFALSE: libc::c_int = 0 as libc::c_int;
pub const TTRUE: libc::c_int = 1 as libc::c_int;
pub const INTERNAL_RND_SORT_SEED: libc::c_int = 39871946 as libc::c_int;
pub const MARK_DEGENERATE: libc::c_int = 1 as libc::c_int;
pub const QUAD_ONE_DEGEN_TRI: libc::c_int = 2 as libc::c_int;
pub const GROUP_WITH_ANY: libc::c_int = 4 as libc::c_int;
pub const ORIENT_PRESERVING: libc::c_int = 8 as libc::c_int;
unsafe extern "C" fn MakeIndex(iFace: libc::c_int, iVert: libc::c_int) -> libc::c_int {
    assert!(iVert >= 0 as libc::c_int && iVert < 4 as libc::c_int && iFace >= 0 as libc::c_int);
    iFace << 2 as libc::c_int | iVert & 0x3 as libc::c_int
}
unsafe extern "C" fn IndexToData(
    mut piFace: *mut libc::c_int,
    mut piVert: *mut libc::c_int,
    iIndexIn: libc::c_int,
) {
    *piVert.offset(0 as libc::c_int as isize) = iIndexIn & 0x3 as libc::c_int;
    *piFace.offset(0 as libc::c_int as isize) = iIndexIn >> 2 as libc::c_int;
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
        bOrient: 0,
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
pub unsafe extern "C" fn genTangSpaceDefault(mut pContext: *const SMikkTSpaceContext) -> tbool {
    genTangSpace(pContext, 180.0f32)
}
pub unsafe extern "C" fn genTangSpace(
    mut pContext: *const SMikkTSpaceContext,
    fAngularThreshold: libc::c_float,
) -> tbool {
    let mut piTriListIn: *mut libc::c_int = NULL as *mut libc::c_int;
    let mut piGroupTrianglesBuffer: *mut libc::c_int = NULL as *mut libc::c_int;
    let mut pTriInfos: *mut STriInfo = NULL as *mut STriInfo;
    let mut pGroups: *mut SGroup = NULL as *mut SGroup;
    let mut psTspace: *mut STSpace = NULL as *mut STSpace;
    let mut iNrTrianglesIn: libc::c_int = 0 as libc::c_int;
    let mut f: libc::c_int = 0 as libc::c_int;
    let mut t: libc::c_int = 0 as libc::c_int;
    let mut i: libc::c_int = 0 as libc::c_int;
    let mut iNrTSPaces: libc::c_int = 0 as libc::c_int;
    let mut iTotTris: libc::c_int = 0 as libc::c_int;
    let mut iDegenTriangles: libc::c_int = 0 as libc::c_int;
    let mut iNrMaxGroups: libc::c_int = 0 as libc::c_int;
    let mut iNrActiveGroups: libc::c_int = 0 as libc::c_int;
    let mut index: libc::c_int = 0 as libc::c_int;
    let iNrFaces: libc::c_int =
        ((*(*pContext).m_pInterface).m_getNumFaces).expect("non-null function pointer")(pContext);
    let mut bRes: tbool = TFALSE;
    let fThresCos: libc::c_float =
        cos(deg_to_rad(fAngularThreshold) as libc::c_double) as libc::c_float;
    if ((*(*pContext).m_pInterface).m_getNumFaces).is_none()
        || ((*(*pContext).m_pInterface).m_getNumVerticesOfFace).is_none()
        || ((*(*pContext).m_pInterface).m_getPosition).is_none()
        || ((*(*pContext).m_pInterface).m_getNormal).is_none()
        || ((*(*pContext).m_pInterface).m_getTexCoord).is_none()
    {
        return TFALSE;
    }
    f = 0 as libc::c_int;
    while f < iNrFaces {
        let verts: libc::c_int = ((*(*pContext).m_pInterface).m_getNumVerticesOfFace)
            .expect("non-null function pointer")(pContext, f);
        if verts == 3 as libc::c_int {
            iNrTrianglesIn += 1;
        } else if verts == 4 as libc::c_int {
            iNrTrianglesIn += 2 as libc::c_int;
        }
        f += 1;
    }
    if iNrTrianglesIn <= 0 as libc::c_int {
        return TFALSE;
    }
    piTriListIn = malloc(
        (::core::mem::size_of::<libc::c_int>() as libc::c_ulong)
            .wrapping_mul(3 as libc::c_int as libc::c_ulong)
            .wrapping_mul(iNrTrianglesIn as libc::c_ulong),
    ) as *mut libc::c_int;
    pTriInfos = malloc(
        (::core::mem::size_of::<STriInfo>() as libc::c_ulong)
            .wrapping_mul(iNrTrianglesIn as libc::c_ulong),
    ) as *mut STriInfo;
    if piTriListIn.is_null() || pTriInfos.is_null() {
        if !piTriListIn.is_null() {
            free(piTriListIn as *mut libc::c_void);
        }
        if !pTriInfos.is_null() {
            free(pTriInfos as *mut libc::c_void);
        }
        return TFALSE;
    }
    iNrTSPaces = GenerateInitialVerticesIndexList(pTriInfos, piTriListIn, pContext, iNrTrianglesIn);
    GenerateSharedVerticesIndexList(piTriListIn, pContext, iNrTrianglesIn);
    iTotTris = iNrTrianglesIn;
    iDegenTriangles = 0 as libc::c_int;
    t = 0 as libc::c_int;
    while t < iTotTris {
        let i0: libc::c_int =
            *piTriListIn.offset((t * 3 as libc::c_int + 0 as libc::c_int) as isize);
        let i1: libc::c_int =
            *piTriListIn.offset((t * 3 as libc::c_int + 1 as libc::c_int) as isize);
        let i2: libc::c_int =
            *piTriListIn.offset((t * 3 as libc::c_int + 2 as libc::c_int) as isize);
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
        piTriListIn as *const libc::c_int,
        pContext,
        iNrTrianglesIn,
    );
    iNrMaxGroups = iNrTrianglesIn * 3 as libc::c_int;
    pGroups = malloc(
        (::core::mem::size_of::<SGroup>() as libc::c_ulong)
            .wrapping_mul(iNrMaxGroups as libc::c_ulong),
    ) as *mut SGroup;
    piGroupTrianglesBuffer = malloc(
        (::core::mem::size_of::<libc::c_int>() as libc::c_ulong)
            .wrapping_mul(iNrTrianglesIn as libc::c_ulong)
            .wrapping_mul(3 as libc::c_int as libc::c_ulong),
    ) as *mut libc::c_int;
    if pGroups.is_null() || piGroupTrianglesBuffer.is_null() {
        if !pGroups.is_null() {
            free(pGroups as *mut libc::c_void);
        }
        if !piGroupTrianglesBuffer.is_null() {
            free(piGroupTrianglesBuffer as *mut libc::c_void);
        }
        free(piTriListIn as *mut libc::c_void);
        free(pTriInfos as *mut libc::c_void);
        return TFALSE;
    }
    iNrActiveGroups = Build4RuleGroups(
        pTriInfos,
        pGroups,
        piGroupTrianglesBuffer,
        piTriListIn as *const libc::c_int,
        iNrTrianglesIn,
    );
    psTspace = malloc(
        (::core::mem::size_of::<STSpace>() as libc::c_ulong)
            .wrapping_mul(iNrTSPaces as libc::c_ulong),
    ) as *mut STSpace;
    if psTspace.is_null() {
        free(piTriListIn as *mut libc::c_void);
        free(pTriInfos as *mut libc::c_void);
        free(pGroups as *mut libc::c_void);
        free(piGroupTrianglesBuffer as *mut libc::c_void);
        return TFALSE;
    }
    memset(
        psTspace as *mut libc::c_void,
        0 as libc::c_int,
        (::core::mem::size_of::<STSpace>() as libc::c_ulong)
            .wrapping_mul(iNrTSPaces as libc::c_ulong),
    );
    t = 0 as libc::c_int;
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
        piTriListIn as *const libc::c_int,
        fThresCos,
        pContext,
    );
    free(pGroups as *mut libc::c_void);
    free(piGroupTrianglesBuffer as *mut libc::c_void);
    if bRes == 0 {
        free(pTriInfos as *mut libc::c_void);
        free(piTriListIn as *mut libc::c_void);
        free(psTspace as *mut libc::c_void);
        return TFALSE;
    }
    DegenEpilogue(
        psTspace,
        pTriInfos,
        piTriListIn,
        pContext,
        iNrTrianglesIn,
        iTotTris,
    );
    free(pTriInfos as *mut libc::c_void);
    free(piTriListIn as *mut libc::c_void);
    index = 0 as libc::c_int;
    f = 0 as libc::c_int;
    while f < iNrFaces {
        let verts_0: libc::c_int = ((*(*pContext).m_pInterface).m_getNumVerticesOfFace)
            .expect("non-null function pointer")(pContext, f);
        if !(verts_0 != 3 as libc::c_int && verts_0 != 4 as libc::c_int) {
            i = 0 as libc::c_int;
            while i < verts_0 {
                let mut pTSpace: *const STSpace =
                    &mut *psTspace.offset(index as isize) as *mut STSpace;
                let mut tang: [libc::c_float; 3] =
                    [(*pTSpace).vOs.x, (*pTSpace).vOs.y, (*pTSpace).vOs.z];
                let mut bitang: [libc::c_float; 3] =
                    [(*pTSpace).vOt.x, (*pTSpace).vOt.y, (*pTSpace).vOt.z];
                if ((*(*pContext).m_pInterface).m_setTSpace).is_some() {
                    ((*(*pContext).m_pInterface).m_setTSpace).expect("non-null function pointer")(
                        pContext,
                        tang.as_mut_ptr() as *const libc::c_float,
                        bitang.as_mut_ptr() as *const libc::c_float,
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
                        tang.as_mut_ptr() as *const libc::c_float,
                        if (*pTSpace).bOrient == TTRUE {
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
    free(psTspace as *mut libc::c_void);
    TTRUE
}
static mut g_iCells: libc::c_int = 2048 as libc::c_int;
#[inline(never)]
unsafe extern "C" fn FindGridCell(
    fMin: libc::c_float,
    fMax: libc::c_float,
    fVal: libc::c_float,
) -> libc::c_int {
    let fIndex: libc::c_float = g_iCells as libc::c_float * ((fVal - fMin) / (fMax - fMin));
    let iIndex: libc::c_int = fIndex as libc::c_int;
    if iIndex < g_iCells {
        if iIndex >= 0 as libc::c_int {
            iIndex
        } else {
            0 as libc::c_int
        }
    } else {
        g_iCells - 1 as libc::c_int
    }
}
unsafe extern "C" fn GenerateSharedVerticesIndexList(
    mut piTriList_in_and_out: *mut libc::c_int,
    mut pContext: *const SMikkTSpaceContext,
    iNrTrianglesIn: libc::c_int,
) {
    let mut piHashTable: *mut libc::c_int = NULL as *mut libc::c_int;
    let mut piHashCount: *mut libc::c_int = NULL as *mut libc::c_int;
    let mut piHashOffsets: *mut libc::c_int = NULL as *mut libc::c_int;
    let mut piHashCount2: *mut libc::c_int = NULL as *mut libc::c_int;
    let mut pTmpVert: *mut STmpVert = NULL as *mut STmpVert;
    let mut i: libc::c_int = 0 as libc::c_int;
    let mut iChannel: libc::c_int = 0 as libc::c_int;
    let mut k: libc::c_int = 0 as libc::c_int;
    let mut e: libc::c_int = 0 as libc::c_int;
    let mut iMaxCount: libc::c_int = 0 as libc::c_int;
    let mut vMin: SVec3 = GetPosition(pContext, 0 as libc::c_int);
    let mut vMax: SVec3 = vMin;
    let mut vDim: SVec3 = SVec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    let mut fMin: libc::c_float = 0.;
    let mut fMax: libc::c_float = 0.;
    i = 1 as libc::c_int;
    while i < iNrTrianglesIn * 3 as libc::c_int {
        let index: libc::c_int = *piTriList_in_and_out.offset(i as isize);
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
    iChannel = 0 as libc::c_int;
    fMin = vMin.x;
    fMax = vMax.x;
    if vDim.y > vDim.x && vDim.y > vDim.z {
        iChannel = 1 as libc::c_int;
        fMin = vMin.y;
        fMax = vMax.y;
    } else if vDim.z > vDim.x {
        iChannel = 2 as libc::c_int;
        fMin = vMin.z;
        fMax = vMax.z;
    }
    piHashTable = malloc(
        (::core::mem::size_of::<libc::c_int>() as libc::c_ulong)
            .wrapping_mul(iNrTrianglesIn as libc::c_ulong)
            .wrapping_mul(3 as libc::c_int as libc::c_ulong),
    ) as *mut libc::c_int;
    piHashCount = malloc(
        (::core::mem::size_of::<libc::c_int>() as libc::c_ulong)
            .wrapping_mul(g_iCells as libc::c_ulong),
    ) as *mut libc::c_int;
    piHashOffsets = malloc(
        (::core::mem::size_of::<libc::c_int>() as libc::c_ulong)
            .wrapping_mul(g_iCells as libc::c_ulong),
    ) as *mut libc::c_int;
    piHashCount2 = malloc(
        (::core::mem::size_of::<libc::c_int>() as libc::c_ulong)
            .wrapping_mul(g_iCells as libc::c_ulong),
    ) as *mut libc::c_int;
    if piHashTable.is_null()
        || piHashCount.is_null()
        || piHashOffsets.is_null()
        || piHashCount2.is_null()
    {
        if !piHashTable.is_null() {
            free(piHashTable as *mut libc::c_void);
        }
        if !piHashCount.is_null() {
            free(piHashCount as *mut libc::c_void);
        }
        if !piHashOffsets.is_null() {
            free(piHashOffsets as *mut libc::c_void);
        }
        if !piHashCount2.is_null() {
            free(piHashCount2 as *mut libc::c_void);
        }
        GenerateSharedVerticesIndexListSlow(piTriList_in_and_out, pContext, iNrTrianglesIn);
        return;
    }
    memset(
        piHashCount as *mut libc::c_void,
        0 as libc::c_int,
        (::core::mem::size_of::<libc::c_int>() as libc::c_ulong)
            .wrapping_mul(g_iCells as libc::c_ulong),
    );
    memset(
        piHashCount2 as *mut libc::c_void,
        0 as libc::c_int,
        (::core::mem::size_of::<libc::c_int>() as libc::c_ulong)
            .wrapping_mul(g_iCells as libc::c_ulong),
    );
    i = 0 as libc::c_int;
    while i < iNrTrianglesIn * 3 as libc::c_int {
        let index_0: libc::c_int = *piTriList_in_and_out.offset(i as isize);
        let vP_0: SVec3 = GetPosition(pContext, index_0);
        let fVal: libc::c_float = if iChannel == 0 as libc::c_int {
            vP_0.x
        } else if iChannel == 1 as libc::c_int {
            vP_0.y
        } else {
            vP_0.z
        };
        let iCell: libc::c_int = FindGridCell(fMin, fMax, fVal);
        let fresh0 = &mut (*piHashCount.offset(iCell as isize));
        *fresh0 += 1;
        i += 1;
    }
    *piHashOffsets.offset(0 as libc::c_int as isize) = 0 as libc::c_int;
    k = 1 as libc::c_int;
    while k < g_iCells {
        *piHashOffsets.offset(k as isize) = *piHashOffsets.offset((k - 1 as libc::c_int) as isize)
            + *piHashCount.offset((k - 1 as libc::c_int) as isize);
        k += 1;
    }
    i = 0 as libc::c_int;
    while i < iNrTrianglesIn * 3 as libc::c_int {
        let index_1: libc::c_int = *piTriList_in_and_out.offset(i as isize);
        let vP_1: SVec3 = GetPosition(pContext, index_1);
        let fVal_0: libc::c_float = if iChannel == 0 as libc::c_int {
            vP_1.x
        } else if iChannel == 1 as libc::c_int {
            vP_1.y
        } else {
            vP_1.z
        };
        let iCell_0: libc::c_int = FindGridCell(fMin, fMax, fVal_0);
        let mut pTable: *mut libc::c_int = NULL as *mut libc::c_int;
        assert!(*piHashCount2.offset(iCell_0 as isize) < *piHashCount.offset(iCell_0 as isize));
        pTable = &mut *piHashTable.offset(*piHashOffsets.offset(iCell_0 as isize) as isize)
            as *mut libc::c_int;
        *pTable.offset(*piHashCount2.offset(iCell_0 as isize) as isize) = i;
        let fresh1 = &mut (*piHashCount2.offset(iCell_0 as isize));
        *fresh1 += 1;
        i += 1;
    }
    k = 0 as libc::c_int;
    while k < g_iCells {
        assert!(*piHashCount2.offset(k as isize) == *piHashCount.offset(k as isize));
        k += 1;
    }
    free(piHashCount2 as *mut libc::c_void);
    iMaxCount = *piHashCount.offset(0 as libc::c_int as isize);
    k = 1 as libc::c_int;
    while k < g_iCells {
        if iMaxCount < *piHashCount.offset(k as isize) {
            iMaxCount = *piHashCount.offset(k as isize);
        }
        k += 1;
    }
    pTmpVert = malloc(
        (::core::mem::size_of::<STmpVert>() as libc::c_ulong)
            .wrapping_mul(iMaxCount as libc::c_ulong),
    ) as *mut STmpVert;
    k = 0 as libc::c_int;
    while k < g_iCells {
        let mut pTable_0: *mut libc::c_int = &mut *piHashTable
            .offset(*piHashOffsets.offset(k as isize) as isize)
            as *mut libc::c_int;
        let iEntries: libc::c_int = *piHashCount.offset(k as isize);
        if iEntries >= 2 as libc::c_int {
            if !pTmpVert.is_null() {
                e = 0 as libc::c_int;
                while e < iEntries {
                    let mut i_0: libc::c_int = *pTable_0.offset(e as isize);
                    let vP_2: SVec3 =
                        GetPosition(pContext, *piTriList_in_and_out.offset(i_0 as isize));
                    (*pTmpVert.offset(e as isize)).vert[0 as libc::c_int as usize] = vP_2.x;
                    (*pTmpVert.offset(e as isize)).vert[1 as libc::c_int as usize] = vP_2.y;
                    (*pTmpVert.offset(e as isize)).vert[2 as libc::c_int as usize] = vP_2.z;
                    (*pTmpVert.offset(e as isize)).index = i_0;
                    e += 1;
                }
                MergeVertsFast(
                    piTriList_in_and_out,
                    pTmpVert,
                    pContext,
                    0 as libc::c_int,
                    iEntries - 1 as libc::c_int,
                );
            } else {
                MergeVertsSlow(
                    piTriList_in_and_out,
                    pContext,
                    pTable_0 as *const libc::c_int,
                    iEntries,
                );
            }
        }
        k += 1;
    }
    if !pTmpVert.is_null() {
        free(pTmpVert as *mut libc::c_void);
    }
    free(piHashTable as *mut libc::c_void);
    free(piHashCount as *mut libc::c_void);
    free(piHashOffsets as *mut libc::c_void);
}
unsafe extern "C" fn MergeVertsFast(
    mut piTriList_in_and_out: *mut libc::c_int,
    mut pTmpVert: *mut STmpVert,
    mut pContext: *const SMikkTSpaceContext,
    iL_in: libc::c_int,
    iR_in: libc::c_int,
) {
    let mut c: libc::c_int = 0 as libc::c_int;
    let mut l: libc::c_int = 0 as libc::c_int;
    let mut channel: libc::c_int = 0 as libc::c_int;
    let mut fvMin: [libc::c_float; 3] = [0.; 3];
    let mut fvMax: [libc::c_float; 3] = [0.; 3];
    let mut dx: libc::c_float = 0 as libc::c_int as libc::c_float;
    let mut dy: libc::c_float = 0 as libc::c_int as libc::c_float;
    let mut dz: libc::c_float = 0 as libc::c_int as libc::c_float;
    let mut fSep: libc::c_float = 0 as libc::c_int as libc::c_float;
    c = 0 as libc::c_int;
    while c < 3 as libc::c_int {
        fvMin[c as usize] = (*pTmpVert.offset(iL_in as isize)).vert[c as usize];
        fvMax[c as usize] = fvMin[c as usize];
        c += 1;
    }
    l = iL_in + 1 as libc::c_int;
    while l <= iR_in {
        c = 0 as libc::c_int;
        while c < 3 as libc::c_int {
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
    dx = fvMax[0 as libc::c_int as usize] - fvMin[0 as libc::c_int as usize];
    dy = fvMax[1 as libc::c_int as usize] - fvMin[1 as libc::c_int as usize];
    dz = fvMax[2 as libc::c_int as usize] - fvMin[2 as libc::c_int as usize];
    channel = 0 as libc::c_int;
    if dy > dx && dy > dz {
        channel = 1 as libc::c_int;
    } else if dz > dx {
        channel = 2 as libc::c_int;
    }
    fSep = 0.5f32 * (fvMax[channel as usize] + fvMin[channel as usize]);
    if fSep.is_finite() as i32 == 0 {
        return;
    }
    if fSep >= fvMax[channel as usize] || fSep <= fvMin[channel as usize] {
        l = iL_in;
        while l <= iR_in {
            let mut i: libc::c_int = (*pTmpVert.offset(l as isize)).index;
            let index: libc::c_int = *piTriList_in_and_out.offset(i as isize);
            let vP: SVec3 = GetPosition(pContext, index);
            let vN: SVec3 = GetNormal(pContext, index);
            let vT: SVec3 = GetTexCoord(pContext, index);
            let mut bNotFound: tbool = TTRUE;
            let mut l2: libc::c_int = iL_in;
            let mut i2rec: libc::c_int = -(1 as libc::c_int);
            while l2 < l && bNotFound != 0 {
                let i2: libc::c_int = (*pTmpVert.offset(l2 as isize)).index;
                let index2: libc::c_int = *piTriList_in_and_out.offset(i2 as isize);
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
                    bNotFound = TFALSE;
                } else {
                    l2 += 1;
                }
            }
            if bNotFound == 0 {
                *piTriList_in_and_out.offset(i as isize) =
                    *piTriList_in_and_out.offset(i2rec as isize);
            }
            l += 1;
        }
    } else {
        let mut iL: libc::c_int = iL_in;
        let mut iR: libc::c_int = iR_in;
        assert!(iR_in - iL_in > 0 as libc::c_int);
        while iL < iR {
            let mut bReadyLeftSwap: tbool = TFALSE;
            let mut bReadyRightSwap: tbool = TFALSE;
            while bReadyLeftSwap == 0 && iL < iR {
                assert!(iL >= iL_in && iL <= iR_in);
                bReadyLeftSwap =
                    !((*pTmpVert.offset(iL as isize)).vert[channel as usize] < fSep) as libc::c_int;
                if bReadyLeftSwap == 0 {
                    iL += 1;
                }
            }
            while bReadyRightSwap == 0 && iL < iR {
                assert!(iR >= iL_in && iR <= iR_in);
                bReadyRightSwap =
                    ((*pTmpVert.offset(iR as isize)).vert[channel as usize] < fSep) as libc::c_int;
                if bReadyRightSwap == 0 {
                    iR -= 1;
                }
            }
            assert!(iL < iR || !(bReadyLeftSwap != 0 && bReadyRightSwap != 0));
            if bReadyLeftSwap != 0 && bReadyRightSwap != 0 {
                let sTmp: STmpVert = *pTmpVert.offset(iL as isize);
                assert!(iL < iR);
                *pTmpVert.offset(iL as isize) = *pTmpVert.offset(iR as isize);
                *pTmpVert.offset(iR as isize) = sTmp;
                iL += 1;
                iR -= 1;
            }
        }
        assert!(iL == iR + 1 as libc::c_int || iL == iR);
        if iL == iR {
            let bReadyRightSwap_0: tbool =
                ((*pTmpVert.offset(iR as isize)).vert[channel as usize] < fSep) as libc::c_int;
            if bReadyRightSwap_0 != 0 {
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
    mut piTriList_in_and_out: *mut libc::c_int,
    mut pContext: *const SMikkTSpaceContext,
    mut pTable: *const libc::c_int,
    iEntries: libc::c_int,
) {
    let mut e: libc::c_int = 0 as libc::c_int;
    e = 0 as libc::c_int;
    while e < iEntries {
        let mut i: libc::c_int = *pTable.offset(e as isize);
        let index: libc::c_int = *piTriList_in_and_out.offset(i as isize);
        let vP: SVec3 = GetPosition(pContext, index);
        let vN: SVec3 = GetNormal(pContext, index);
        let vT: SVec3 = GetTexCoord(pContext, index);
        let mut bNotFound: tbool = TTRUE;
        let mut e2: libc::c_int = 0 as libc::c_int;
        let mut i2rec: libc::c_int = -(1 as libc::c_int);
        while e2 < e && bNotFound != 0 {
            let i2: libc::c_int = *pTable.offset(e2 as isize);
            let index2: libc::c_int = *piTriList_in_and_out.offset(i2 as isize);
            let vP2: SVec3 = GetPosition(pContext, index2);
            let vN2: SVec3 = GetNormal(pContext, index2);
            let vT2: SVec3 = GetTexCoord(pContext, index2);
            i2rec = i2;
            if (vP == vP2) && (vN == vN2) && (vT == vT2) {
                bNotFound = TFALSE;
            } else {
                e2 += 1;
            }
        }
        if bNotFound == 0 {
            *piTriList_in_and_out.offset(i as isize) = *piTriList_in_and_out.offset(i2rec as isize);
        }
        e += 1;
    }
}
unsafe extern "C" fn GenerateSharedVerticesIndexListSlow(
    mut piTriList_in_and_out: *mut libc::c_int,
    mut pContext: *const SMikkTSpaceContext,
    iNrTrianglesIn: libc::c_int,
) {
    let mut t: libc::c_int = 0 as libc::c_int;
    let mut i: libc::c_int = 0 as libc::c_int;
    t = 0 as libc::c_int;
    while t < iNrTrianglesIn {
        i = 0 as libc::c_int;
        while i < 3 as libc::c_int {
            let offs: libc::c_int = t * 3 as libc::c_int + i;
            let index: libc::c_int = *piTriList_in_and_out.offset(offs as isize);
            let vP: SVec3 = GetPosition(pContext, index);
            let vN: SVec3 = GetNormal(pContext, index);
            let vT: SVec3 = GetTexCoord(pContext, index);
            let mut bFound: tbool = TFALSE;
            let mut t2: libc::c_int = 0 as libc::c_int;
            let mut index2rec: libc::c_int = -(1 as libc::c_int);
            while bFound == 0 && t2 <= t {
                let mut j: libc::c_int = 0 as libc::c_int;
                while bFound == 0 && j < 3 as libc::c_int {
                    let index2: libc::c_int =
                        *piTriList_in_and_out.offset((t2 * 3 as libc::c_int + j) as isize);
                    let vP2: SVec3 = GetPosition(pContext, index2);
                    let vN2: SVec3 = GetNormal(pContext, index2);
                    let vT2: SVec3 = GetTexCoord(pContext, index2);
                    if (vP == vP2) && (vN == vN2) && (vT == vT2) {
                        bFound = TTRUE;
                    } else {
                        j += 1;
                    }
                }
                if bFound == 0 {
                    t2 += 1;
                }
            }
            assert!(bFound != 0);
            *piTriList_in_and_out.offset(offs as isize) = index2rec;
            i += 1;
        }
        t += 1;
    }
}
unsafe extern "C" fn GenerateInitialVerticesIndexList(
    mut pTriInfos: *mut STriInfo,
    mut piTriList_out: *mut libc::c_int,
    mut pContext: *const SMikkTSpaceContext,
    iNrTrianglesIn: libc::c_int,
) -> libc::c_int {
    let mut iTSpacesOffs: libc::c_int = 0 as libc::c_int;
    let mut f: libc::c_int = 0 as libc::c_int;
    let mut t: libc::c_int = 0 as libc::c_int;
    let mut iDstTriIndex: libc::c_int = 0 as libc::c_int;
    f = 0 as libc::c_int;
    while f
        < ((*(*pContext).m_pInterface).m_getNumFaces).expect("non-null function pointer")(pContext)
    {
        let verts: libc::c_int = ((*(*pContext).m_pInterface).m_getNumVerticesOfFace)
            .expect("non-null function pointer")(pContext, f);
        if !(verts != 3 as libc::c_int && verts != 4 as libc::c_int) {
            (*pTriInfos.offset(iDstTriIndex as isize)).iOrgFaceNumber = f;
            (*pTriInfos.offset(iDstTriIndex as isize)).iTSpacesOffs = iTSpacesOffs;
            if verts == 3 as libc::c_int {
                let mut pVerts: *mut libc::c_uchar =
                    ((*pTriInfos.offset(iDstTriIndex as isize)).vert_num).as_mut_ptr();
                *pVerts.offset(0 as libc::c_int as isize) = 0 as libc::c_int as libc::c_uchar;
                *pVerts.offset(1 as libc::c_int as isize) = 1 as libc::c_int as libc::c_uchar;
                *pVerts.offset(2 as libc::c_int as isize) = 2 as libc::c_int as libc::c_uchar;
                *piTriList_out
                    .offset((iDstTriIndex * 3 as libc::c_int + 0 as libc::c_int) as isize) =
                    MakeIndex(f, 0 as libc::c_int);
                *piTriList_out
                    .offset((iDstTriIndex * 3 as libc::c_int + 1 as libc::c_int) as isize) =
                    MakeIndex(f, 1 as libc::c_int);
                *piTriList_out
                    .offset((iDstTriIndex * 3 as libc::c_int + 2 as libc::c_int) as isize) =
                    MakeIndex(f, 2 as libc::c_int);
                iDstTriIndex += 1;
            } else {
                (*pTriInfos.offset((iDstTriIndex + 1 as libc::c_int) as isize)).iOrgFaceNumber = f;
                (*pTriInfos.offset((iDstTriIndex + 1 as libc::c_int) as isize)).iTSpacesOffs =
                    iTSpacesOffs;
                let i0: libc::c_int = MakeIndex(f, 0 as libc::c_int);
                let i1: libc::c_int = MakeIndex(f, 1 as libc::c_int);
                let i2: libc::c_int = MakeIndex(f, 2 as libc::c_int);
                let i3: libc::c_int = MakeIndex(f, 3 as libc::c_int);
                let T0: SVec3 = GetTexCoord(pContext, i0);
                let T1: SVec3 = GetTexCoord(pContext, i1);
                let T2: SVec3 = GetTexCoord(pContext, i2);
                let T3: SVec3 = GetTexCoord(pContext, i3);
                let distSQ_02: libc::c_float = (T2 - T0).length_squared();
                let distSQ_13: libc::c_float = (T3 - T1).length_squared();
                let mut bQuadDiagIs_02: tbool = 0;
                if distSQ_02 < distSQ_13 {
                    bQuadDiagIs_02 = TTRUE;
                } else if distSQ_13 < distSQ_02 {
                    bQuadDiagIs_02 = TFALSE;
                } else {
                    let P0: SVec3 = GetPosition(pContext, i0);
                    let P1: SVec3 = GetPosition(pContext, i1);
                    let P2: SVec3 = GetPosition(pContext, i2);
                    let P3: SVec3 = GetPosition(pContext, i3);
                    let distSQ_02_0: libc::c_float = (P2 - P0).length_squared();
                    let distSQ_13_0: libc::c_float = (P3 - P1).length_squared();
                    bQuadDiagIs_02 = if distSQ_13_0 < distSQ_02_0 {
                        TFALSE
                    } else {
                        TTRUE
                    };
                }
                if bQuadDiagIs_02 != 0 {
                    let mut pVerts_A: *mut libc::c_uchar =
                        ((*pTriInfos.offset(iDstTriIndex as isize)).vert_num).as_mut_ptr();
                    *pVerts_A.offset(0 as libc::c_int as isize) = 0 as libc::c_int as libc::c_uchar;
                    *pVerts_A.offset(1 as libc::c_int as isize) = 1 as libc::c_int as libc::c_uchar;
                    *pVerts_A.offset(2 as libc::c_int as isize) = 2 as libc::c_int as libc::c_uchar;
                    *piTriList_out
                        .offset((iDstTriIndex * 3 as libc::c_int + 0 as libc::c_int) as isize) = i0;
                    *piTriList_out
                        .offset((iDstTriIndex * 3 as libc::c_int + 1 as libc::c_int) as isize) = i1;
                    *piTriList_out
                        .offset((iDstTriIndex * 3 as libc::c_int + 2 as libc::c_int) as isize) = i2;
                    iDstTriIndex += 1;
                    let mut pVerts_B: *mut libc::c_uchar =
                        ((*pTriInfos.offset(iDstTriIndex as isize)).vert_num).as_mut_ptr();
                    *pVerts_B.offset(0 as libc::c_int as isize) = 0 as libc::c_int as libc::c_uchar;
                    *pVerts_B.offset(1 as libc::c_int as isize) = 2 as libc::c_int as libc::c_uchar;
                    *pVerts_B.offset(2 as libc::c_int as isize) = 3 as libc::c_int as libc::c_uchar;
                    *piTriList_out
                        .offset((iDstTriIndex * 3 as libc::c_int + 0 as libc::c_int) as isize) = i0;
                    *piTriList_out
                        .offset((iDstTriIndex * 3 as libc::c_int + 1 as libc::c_int) as isize) = i2;
                    *piTriList_out
                        .offset((iDstTriIndex * 3 as libc::c_int + 2 as libc::c_int) as isize) = i3;
                    iDstTriIndex += 1;
                } else {
                    let mut pVerts_A_0: *mut libc::c_uchar =
                        ((*pTriInfos.offset(iDstTriIndex as isize)).vert_num).as_mut_ptr();
                    *pVerts_A_0.offset(0 as libc::c_int as isize) =
                        0 as libc::c_int as libc::c_uchar;
                    *pVerts_A_0.offset(1 as libc::c_int as isize) =
                        1 as libc::c_int as libc::c_uchar;
                    *pVerts_A_0.offset(2 as libc::c_int as isize) =
                        3 as libc::c_int as libc::c_uchar;
                    *piTriList_out
                        .offset((iDstTriIndex * 3 as libc::c_int + 0 as libc::c_int) as isize) = i0;
                    *piTriList_out
                        .offset((iDstTriIndex * 3 as libc::c_int + 1 as libc::c_int) as isize) = i1;
                    *piTriList_out
                        .offset((iDstTriIndex * 3 as libc::c_int + 2 as libc::c_int) as isize) = i3;
                    iDstTriIndex += 1;
                    let mut pVerts_B_0: *mut libc::c_uchar =
                        ((*pTriInfos.offset(iDstTriIndex as isize)).vert_num).as_mut_ptr();
                    *pVerts_B_0.offset(0 as libc::c_int as isize) =
                        1 as libc::c_int as libc::c_uchar;
                    *pVerts_B_0.offset(1 as libc::c_int as isize) =
                        2 as libc::c_int as libc::c_uchar;
                    *pVerts_B_0.offset(2 as libc::c_int as isize) =
                        3 as libc::c_int as libc::c_uchar;
                    *piTriList_out
                        .offset((iDstTriIndex * 3 as libc::c_int + 0 as libc::c_int) as isize) = i1;
                    *piTriList_out
                        .offset((iDstTriIndex * 3 as libc::c_int + 1 as libc::c_int) as isize) = i2;
                    *piTriList_out
                        .offset((iDstTriIndex * 3 as libc::c_int + 2 as libc::c_int) as isize) = i3;
                    iDstTriIndex += 1;
                }
            }
            iTSpacesOffs += verts;
            assert!(iDstTriIndex <= iNrTrianglesIn);
        }
        f += 1;
    }
    t = 0 as libc::c_int;
    while t < iNrTrianglesIn {
        (*pTriInfos.offset(t as isize)).iFlag = 0 as libc::c_int;
        t += 1;
    }
    iTSpacesOffs
}
unsafe extern "C" fn GetPosition(
    mut pContext: *const SMikkTSpaceContext,
    index: libc::c_int,
) -> SVec3 {
    let mut iF: libc::c_int = 0;
    let mut iI: libc::c_int = 0;
    let mut res: SVec3 = SVec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    let mut pos: [libc::c_float; 3] = [0.; 3];
    IndexToData(&mut iF, &mut iI, index);
    ((*(*pContext).m_pInterface).m_getPosition).expect("non-null function pointer")(
        pContext,
        pos.as_mut_ptr(),
        iF,
        iI,
    );
    res.x = pos[0 as libc::c_int as usize];
    res.y = pos[1 as libc::c_int as usize];
    res.z = pos[2 as libc::c_int as usize];
    res
}
unsafe extern "C" fn GetNormal(
    mut pContext: *const SMikkTSpaceContext,
    index: libc::c_int,
) -> SVec3 {
    let mut iF: libc::c_int = 0;
    let mut iI: libc::c_int = 0;
    let mut res: SVec3 = SVec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    let mut norm: [libc::c_float; 3] = [0.; 3];
    IndexToData(&mut iF, &mut iI, index);
    ((*(*pContext).m_pInterface).m_getNormal).expect("non-null function pointer")(
        pContext,
        norm.as_mut_ptr(),
        iF,
        iI,
    );
    res.x = norm[0 as libc::c_int as usize];
    res.y = norm[1 as libc::c_int as usize];
    res.z = norm[2 as libc::c_int as usize];
    res
}
unsafe extern "C" fn GetTexCoord(
    mut pContext: *const SMikkTSpaceContext,
    index: libc::c_int,
) -> SVec3 {
    let mut iF: libc::c_int = 0;
    let mut iI: libc::c_int = 0;
    let mut res: SVec3 = SVec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    let mut texc: [libc::c_float; 2] = [0.; 2];
    IndexToData(&mut iF, &mut iI, index);
    ((*(*pContext).m_pInterface).m_getTexCoord).expect("non-null function pointer")(
        pContext,
        texc.as_mut_ptr(),
        iF,
        iI,
    );
    res.x = texc[0 as libc::c_int as usize];
    res.y = texc[1 as libc::c_int as usize];
    res.z = 1.0f32;
    res
}
unsafe extern "C" fn CalcTexArea(
    mut pContext: *const SMikkTSpaceContext,
    mut indices: *const libc::c_int,
) -> libc::c_float {
    let t1: SVec3 = GetTexCoord(pContext, *indices.offset(0 as libc::c_int as isize));
    let t2: SVec3 = GetTexCoord(pContext, *indices.offset(1 as libc::c_int as isize));
    let t3: SVec3 = GetTexCoord(pContext, *indices.offset(2 as libc::c_int as isize));
    let t21x: libc::c_float = t2.x - t1.x;
    let t21y: libc::c_float = t2.y - t1.y;
    let t31x: libc::c_float = t3.x - t1.x;
    let t31y: libc::c_float = t3.y - t1.y;
    let fSignedAreaSTx2: libc::c_float = t21x * t31y - t21y * t31x;
    if fSignedAreaSTx2 < 0 as libc::c_int as libc::c_float {
        -fSignedAreaSTx2
    } else {
        fSignedAreaSTx2
    }
}
unsafe extern "C" fn InitTriInfo(
    mut pTriInfos: *mut STriInfo,
    mut piTriListIn: *const libc::c_int,
    mut pContext: *const SMikkTSpaceContext,
    iNrTrianglesIn: libc::c_int,
) {
    let mut f: libc::c_int = 0 as libc::c_int;
    let mut i: libc::c_int = 0 as libc::c_int;
    let mut t: libc::c_int = 0 as libc::c_int;
    f = 0 as libc::c_int;
    while f < iNrTrianglesIn {
        i = 0 as libc::c_int;
        while i < 3 as libc::c_int {
            (*pTriInfos.offset(f as isize)).FaceNeighbors[i as usize] = -(1 as libc::c_int);
            let fresh2 = &mut (*pTriInfos.offset(f as isize)).AssignedGroup[i as usize];
            *fresh2 = NULL as *mut SGroup;
            (*pTriInfos.offset(f as isize)).vOs.x = 0.0f32;
            (*pTriInfos.offset(f as isize)).vOs.y = 0.0f32;
            (*pTriInfos.offset(f as isize)).vOs.z = 0.0f32;
            (*pTriInfos.offset(f as isize)).vOt.x = 0.0f32;
            (*pTriInfos.offset(f as isize)).vOt.y = 0.0f32;
            (*pTriInfos.offset(f as isize)).vOt.z = 0.0f32;
            (*pTriInfos.offset(f as isize)).fMagS = 0 as libc::c_int as libc::c_float;
            (*pTriInfos.offset(f as isize)).fMagT = 0 as libc::c_int as libc::c_float;
            (*pTriInfos.offset(f as isize)).iFlag |= GROUP_WITH_ANY;
            i += 1;
        }
        f += 1;
    }
    f = 0 as libc::c_int;
    while f < iNrTrianglesIn {
        let v1: SVec3 = GetPosition(
            pContext,
            *piTriListIn.offset((f * 3 as libc::c_int + 0 as libc::c_int) as isize),
        );
        let v2: SVec3 = GetPosition(
            pContext,
            *piTriListIn.offset((f * 3 as libc::c_int + 1 as libc::c_int) as isize),
        );
        let v3: SVec3 = GetPosition(
            pContext,
            *piTriListIn.offset((f * 3 as libc::c_int + 2 as libc::c_int) as isize),
        );
        let t1: SVec3 = GetTexCoord(
            pContext,
            *piTriListIn.offset((f * 3 as libc::c_int + 0 as libc::c_int) as isize),
        );
        let t2: SVec3 = GetTexCoord(
            pContext,
            *piTriListIn.offset((f * 3 as libc::c_int + 1 as libc::c_int) as isize),
        );
        let t3: SVec3 = GetTexCoord(
            pContext,
            *piTriListIn.offset((f * 3 as libc::c_int + 2 as libc::c_int) as isize),
        );
        let t21x: libc::c_float = t2.x - t1.x;
        let t21y: libc::c_float = t2.y - t1.y;
        let t31x: libc::c_float = t3.x - t1.x;
        let t31y: libc::c_float = t3.y - t1.y;
        let d1: SVec3 = v2 - v1;
        let d2: SVec3 = v3 - v1;
        let fSignedAreaSTx2: libc::c_float = t21x * t31y - t21y * t31x;
        let mut vOs: SVec3 = (t31y * d1) - (t21y * d2);
        let mut vOt: SVec3 = (-t31x * d1) + (t21x * d2);
        (*pTriInfos.offset(f as isize)).iFlag |=
            if fSignedAreaSTx2 > 0 as libc::c_int as libc::c_float {
                ORIENT_PRESERVING
            } else {
                0 as libc::c_int
            };
        if not_zero(fSignedAreaSTx2) {
            let fAbsArea: libc::c_float = fabsf(fSignedAreaSTx2);
            let fLenOs: libc::c_float = vOs.length();
            let fLenOt: libc::c_float = vOt.length();
            let fS: libc::c_float =
                if (*pTriInfos.offset(f as isize)).iFlag & ORIENT_PRESERVING == 0 as libc::c_int {
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
    while t < iNrTrianglesIn - 1 as libc::c_int {
        let iFO_a: libc::c_int = (*pTriInfos.offset(t as isize)).iOrgFaceNumber;
        let iFO_b: libc::c_int =
            (*pTriInfos.offset((t + 1 as libc::c_int) as isize)).iOrgFaceNumber;
        if iFO_a == iFO_b {
            let bIsDeg_a: tbool =
                if (*pTriInfos.offset(t as isize)).iFlag & MARK_DEGENERATE != 0 as libc::c_int {
                    TTRUE
                } else {
                    TFALSE
                };
            let bIsDeg_b: tbool = if (*pTriInfos.offset((t + 1 as libc::c_int) as isize)).iFlag
                & MARK_DEGENERATE
                != 0 as libc::c_int
            {
                TTRUE
            } else {
                TFALSE
            };
            if (bIsDeg_a != 0 || bIsDeg_b != 0) as libc::c_int == TFALSE {
                let bOrientA: tbool = if (*pTriInfos.offset(t as isize)).iFlag & ORIENT_PRESERVING
                    != 0 as libc::c_int
                {
                    TTRUE
                } else {
                    TFALSE
                };
                let bOrientB: tbool = if (*pTriInfos.offset((t + 1 as libc::c_int) as isize)).iFlag
                    & ORIENT_PRESERVING
                    != 0 as libc::c_int
                {
                    TTRUE
                } else {
                    TFALSE
                };
                if bOrientA != bOrientB {
                    let mut bChooseOrientFirstTri: tbool = TFALSE;
                    if (*pTriInfos.offset((t + 1 as libc::c_int) as isize)).iFlag & GROUP_WITH_ANY
                        != 0 as libc::c_int
                    {
                        bChooseOrientFirstTri = TTRUE;
                    } else if CalcTexArea(
                        pContext,
                        &*piTriListIn.offset((t * 3 as libc::c_int + 0 as libc::c_int) as isize),
                    ) >= CalcTexArea(
                        pContext,
                        &*piTriListIn.offset(
                            ((t + 1 as libc::c_int) * 3 as libc::c_int + 0 as libc::c_int) as isize,
                        ),
                    ) {
                        bChooseOrientFirstTri = TTRUE;
                    }
                    let t0: libc::c_int = if bChooseOrientFirstTri != 0 {
                        t
                    } else {
                        t + 1 as libc::c_int
                    };
                    let t1_0: libc::c_int = if bChooseOrientFirstTri != 0 {
                        t + 1 as libc::c_int
                    } else {
                        t
                    };
                    (*pTriInfos.offset(t1_0 as isize)).iFlag &= !ORIENT_PRESERVING;
                    (*pTriInfos.offset(t1_0 as isize)).iFlag |=
                        (*pTriInfos.offset(t0 as isize)).iFlag & ORIENT_PRESERVING;
                }
            }
            t += 2 as libc::c_int;
        } else {
            t += 1;
        }
    }
    let mut pEdges: *mut SEdge = malloc(
        (::core::mem::size_of::<SEdge>() as libc::c_ulong)
            .wrapping_mul(iNrTrianglesIn as libc::c_ulong)
            .wrapping_mul(3 as libc::c_int as libc::c_ulong),
    ) as *mut SEdge;
    if pEdges.is_null() {
        BuildNeighborsSlow(pTriInfos, piTriListIn, iNrTrianglesIn);
    } else {
        BuildNeighborsFast(pTriInfos, pEdges, piTriListIn, iNrTrianglesIn);
        free(pEdges as *mut libc::c_void);
    };
}
unsafe extern "C" fn Build4RuleGroups(
    mut pTriInfos: *mut STriInfo,
    mut pGroups: *mut SGroup,
    mut piGroupTrianglesBuffer: *mut libc::c_int,
    mut piTriListIn: *const libc::c_int,
    iNrTrianglesIn: libc::c_int,
) -> libc::c_int {
    let iNrMaxGroups: libc::c_int = iNrTrianglesIn * 3 as libc::c_int;
    let mut iNrActiveGroups: libc::c_int = 0 as libc::c_int;
    let mut iOffset: libc::c_int = 0 as libc::c_int;
    let mut f: libc::c_int = 0 as libc::c_int;
    let mut i: libc::c_int = 0 as libc::c_int;
    f = 0 as libc::c_int;
    while f < iNrTrianglesIn {
        i = 0 as libc::c_int;
        while i < 3 as libc::c_int {
            if (*pTriInfos.offset(f as isize)).iFlag & GROUP_WITH_ANY == 0 as libc::c_int
                && ((*pTriInfos.offset(f as isize)).AssignedGroup[i as usize]).is_null()
            {
                let mut bOrPre: tbool = 0;
                let mut neigh_indexL: libc::c_int = 0;
                let mut neigh_indexR: libc::c_int = 0;
                let vert_index: libc::c_int =
                    *piTriListIn.offset((f * 3 as libc::c_int + i) as isize);
                assert!(iNrActiveGroups < iNrMaxGroups);
                let fresh3 = &mut (*pTriInfos.offset(f as isize)).AssignedGroup[i as usize];
                *fresh3 = &mut *pGroups.offset(iNrActiveGroups as isize) as *mut SGroup;
                (*(*pTriInfos.offset(f as isize)).AssignedGroup[i as usize])
                    .iVertexRepresentitive = vert_index;
                (*(*pTriInfos.offset(f as isize)).AssignedGroup[i as usize]).bOrientPreservering =
                    ((*pTriInfos.offset(f as isize)).iFlag & ORIENT_PRESERVING != 0 as libc::c_int)
                        as libc::c_int;
                (*(*pTriInfos.offset(f as isize)).AssignedGroup[i as usize]).iNrFaces =
                    0 as libc::c_int;
                let fresh4 =
                    &mut (*(*pTriInfos.offset(f as isize)).AssignedGroup[i as usize]).pFaceIndices;
                *fresh4 = &mut *piGroupTrianglesBuffer.offset(iOffset as isize) as *mut libc::c_int;
                iNrActiveGroups += 1;
                AddTriToGroup((*pTriInfos.offset(f as isize)).AssignedGroup[i as usize], f);
                bOrPre = if (*pTriInfos.offset(f as isize)).iFlag & ORIENT_PRESERVING
                    != 0 as libc::c_int
                {
                    TTRUE
                } else {
                    TFALSE
                };
                neigh_indexL = (*pTriInfos.offset(f as isize)).FaceNeighbors[i as usize];
                neigh_indexR =
                    (*pTriInfos.offset(f as isize)).FaceNeighbors[(if i > 0 as libc::c_int {
                        i - 1 as libc::c_int
                    } else {
                        2 as libc::c_int
                    }) as usize];
                if neigh_indexL >= 0 as libc::c_int {
                    let bAnswer: tbool = AssignRecur(
                        piTriListIn,
                        pTriInfos,
                        neigh_indexL,
                        (*pTriInfos.offset(f as isize)).AssignedGroup[i as usize],
                    );
                    let bOrPre2: tbool = if (*pTriInfos.offset(neigh_indexL as isize)).iFlag
                        & ORIENT_PRESERVING
                        != 0 as libc::c_int
                    {
                        TTRUE
                    } else {
                        TFALSE
                    };
                    let bDiff: tbool = if bOrPre != bOrPre2 { TTRUE } else { TFALSE };
                    assert!(bAnswer != 0 || bDiff != 0);
                }
                if neigh_indexR >= 0 as libc::c_int {
                    let bAnswer_0: tbool = AssignRecur(
                        piTriListIn,
                        pTriInfos,
                        neigh_indexR,
                        (*pTriInfos.offset(f as isize)).AssignedGroup[i as usize],
                    );
                    let bOrPre2_0: tbool = if (*pTriInfos.offset(neigh_indexR as isize)).iFlag
                        & ORIENT_PRESERVING
                        != 0 as libc::c_int
                    {
                        TTRUE
                    } else {
                        TFALSE
                    };
                    let bDiff_0: tbool = if bOrPre != bOrPre2_0 { TTRUE } else { TFALSE };
                    assert!(bAnswer_0 != 0 || bDiff_0 != 0);
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
unsafe extern "C" fn AddTriToGroup(mut pGroup: *mut SGroup, iTriIndex: libc::c_int) {
    *((*pGroup).pFaceIndices).offset((*pGroup).iNrFaces as isize) = iTriIndex;
    (*pGroup).iNrFaces += 1;
    (*pGroup).iNrFaces;
}
unsafe extern "C" fn AssignRecur(
    mut piTriListIn: *const libc::c_int,
    mut psTriInfos: *mut STriInfo,
    iMyTriIndex: libc::c_int,
    mut pGroup: *mut SGroup,
) -> tbool {
    let mut pMyTriInfo: *mut STriInfo =
        &mut *psTriInfos.offset(iMyTriIndex as isize) as *mut STriInfo;
    let iVertRep: libc::c_int = (*pGroup).iVertexRepresentitive;
    let mut pVerts: *const libc::c_int = &*piTriListIn
        .offset((3 as libc::c_int * iMyTriIndex + 0 as libc::c_int) as isize)
        as *const libc::c_int;
    let mut i: libc::c_int = -(1 as libc::c_int);
    if *pVerts.offset(0 as libc::c_int as isize) == iVertRep {
        i = 0 as libc::c_int;
    } else if *pVerts.offset(1 as libc::c_int as isize) == iVertRep {
        i = 1 as libc::c_int;
    } else if *pVerts.offset(2 as libc::c_int as isize) == iVertRep {
        i = 2 as libc::c_int;
    }
    assert!(i >= 0 as libc::c_int && i < 3 as libc::c_int);
    if (*pMyTriInfo).AssignedGroup[i as usize] == pGroup {
        return TTRUE;
    } else if !((*pMyTriInfo).AssignedGroup[i as usize]).is_null() {
        return TFALSE;
    }
    if (*pMyTriInfo).iFlag & GROUP_WITH_ANY != 0 as libc::c_int
        && ((*pMyTriInfo).AssignedGroup[0 as libc::c_int as usize]).is_null()
        && ((*pMyTriInfo).AssignedGroup[1 as libc::c_int as usize]).is_null()
        && ((*pMyTriInfo).AssignedGroup[2 as libc::c_int as usize]).is_null()
    {
        (*pMyTriInfo).iFlag &= !ORIENT_PRESERVING;
        (*pMyTriInfo).iFlag |= if (*pGroup).bOrientPreservering != 0 {
            ORIENT_PRESERVING
        } else {
            0 as libc::c_int
        };
    }
    let bOrient: tbool = if (*pMyTriInfo).iFlag & ORIENT_PRESERVING != 0 as libc::c_int {
        TTRUE
    } else {
        TFALSE
    };
    if bOrient != (*pGroup).bOrientPreservering {
        return TFALSE;
    }
    AddTriToGroup(pGroup, iMyTriIndex);
    (*pMyTriInfo).AssignedGroup[i as usize] = pGroup;
    let neigh_indexL: libc::c_int = (*pMyTriInfo).FaceNeighbors[i as usize];
    let neigh_indexR: libc::c_int = (*pMyTriInfo).FaceNeighbors[(if i > 0 as libc::c_int {
        i - 1 as libc::c_int
    } else {
        2 as libc::c_int
    }) as usize];
    if neigh_indexL >= 0 as libc::c_int {
        AssignRecur(piTriListIn, psTriInfos, neigh_indexL, pGroup);
    }
    if neigh_indexR >= 0 as libc::c_int {
        AssignRecur(piTriListIn, psTriInfos, neigh_indexR, pGroup);
    }
    TTRUE
}
unsafe extern "C" fn GenerateTSpaces(
    mut psTspace: *mut STSpace,
    mut pTriInfos: *const STriInfo,
    mut pGroups: *const SGroup,
    iNrActiveGroups: libc::c_int,
    mut piTriListIn: *const libc::c_int,
    fThresCos: libc::c_float,
    mut pContext: *const SMikkTSpaceContext,
) -> tbool {
    let mut pSubGroupTspace: *mut STSpace = NULL as *mut STSpace;
    let mut pUniSubGroups: *mut SSubGroup = NULL as *mut SSubGroup;
    let mut pTmpMembers: *mut libc::c_int = NULL as *mut libc::c_int;
    let mut iMaxNrFaces: libc::c_int = 0 as libc::c_int;
    let mut g: libc::c_int = 0 as libc::c_int;
    let mut i: libc::c_int = 0 as libc::c_int;
    g = 0 as libc::c_int;
    while g < iNrActiveGroups {
        if iMaxNrFaces < (*pGroups.offset(g as isize)).iNrFaces {
            iMaxNrFaces = (*pGroups.offset(g as isize)).iNrFaces;
        }
        g += 1;
    }
    if iMaxNrFaces == 0 as libc::c_int {
        return TTRUE;
    }
    pSubGroupTspace = malloc(
        (::core::mem::size_of::<STSpace>() as libc::c_ulong)
            .wrapping_mul(iMaxNrFaces as libc::c_ulong),
    ) as *mut STSpace;
    pUniSubGroups = malloc(
        (::core::mem::size_of::<SSubGroup>() as libc::c_ulong)
            .wrapping_mul(iMaxNrFaces as libc::c_ulong),
    ) as *mut SSubGroup;
    pTmpMembers = malloc(
        (::core::mem::size_of::<libc::c_int>() as libc::c_ulong)
            .wrapping_mul(iMaxNrFaces as libc::c_ulong),
    ) as *mut libc::c_int;
    if pSubGroupTspace.is_null() || pUniSubGroups.is_null() || pTmpMembers.is_null() {
        if !pSubGroupTspace.is_null() {
            free(pSubGroupTspace as *mut libc::c_void);
        }
        if !pUniSubGroups.is_null() {
            free(pUniSubGroups as *mut libc::c_void);
        }
        if !pTmpMembers.is_null() {
            free(pTmpMembers as *mut libc::c_void);
        }
        return TFALSE;
    }
    g = 0 as libc::c_int;
    while g < iNrActiveGroups {
        let mut pGroup: *const SGroup = &*pGroups.offset(g as isize) as *const SGroup;
        let mut iUniqueSubGroups: libc::c_int = 0 as libc::c_int;
        let mut s: libc::c_int = 0 as libc::c_int;
        i = 0 as libc::c_int;
        while i < (*pGroup).iNrFaces {
            let f: libc::c_int = *((*pGroup).pFaceIndices).offset(i as isize);
            let mut index: libc::c_int = -(1 as libc::c_int);
            let mut iVertIndex: libc::c_int = -(1 as libc::c_int);
            let mut iOF_1: libc::c_int = -(1 as libc::c_int);
            let mut iMembers: libc::c_int = 0 as libc::c_int;
            let mut j: libc::c_int = 0 as libc::c_int;
            let mut l: libc::c_int = 0 as libc::c_int;
            let mut tmp_group: SSubGroup = SSubGroup {
                iNrFaces: 0,
                pTriMembers: core::ptr::null_mut::<libc::c_int>(),
            };
            let mut bFound: tbool = 0;
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
            if (*pTriInfos.offset(f as isize)).AssignedGroup[0 as libc::c_int as usize]
                == pGroup as *mut SGroup
            {
                index = 0 as libc::c_int;
            } else if (*pTriInfos.offset(f as isize)).AssignedGroup[1 as libc::c_int as usize]
                == pGroup as *mut SGroup
            {
                index = 1 as libc::c_int;
            } else if (*pTriInfos.offset(f as isize)).AssignedGroup[2 as libc::c_int as usize]
                == pGroup as *mut SGroup
            {
                index = 2 as libc::c_int;
            }
            assert!(index >= 0 as libc::c_int && index < 3 as libc::c_int);
            iVertIndex = *piTriListIn.offset((f * 3 as libc::c_int + index) as isize);
            assert!(iVertIndex == (*pGroup).iVertexRepresentitive);
            n = GetNormal(pContext, iVertIndex);
            vOs = (*pTriInfos.offset(f as isize)).vOs
                - ((n.dot((*pTriInfos.offset(f as isize)).vOs)) * n);
            vOt = (*pTriInfos.offset(f as isize)).vOt
                - ((n.dot((*pTriInfos.offset(f as isize)).vOt)) * n);
            vOs.normalize_or_zero();
            vOt.normalize_or_zero();
            iOF_1 = (*pTriInfos.offset(f as isize)).iOrgFaceNumber;
            iMembers = 0 as libc::c_int;
            j = 0 as libc::c_int;
            while j < (*pGroup).iNrFaces {
                let t: libc::c_int = *((*pGroup).pFaceIndices).offset(j as isize);
                let iOF_2: libc::c_int = (*pTriInfos.offset(t as isize)).iOrgFaceNumber;
                let mut vOs2: SVec3 = (*pTriInfos.offset(t as isize)).vOs
                    - ((n.dot((*pTriInfos.offset(t as isize)).vOs)) * n);
                let mut vOt2: SVec3 = (*pTriInfos.offset(t as isize)).vOt
                    - ((n.dot((*pTriInfos.offset(t as isize)).vOt)) * n);
                vOs2.normalize_or_zero();
                vOt2.normalize_or_zero();
                let bAny: tbool = if ((*pTriInfos.offset(f as isize)).iFlag
                    | (*pTriInfos.offset(t as isize)).iFlag)
                    & GROUP_WITH_ANY
                    != 0 as libc::c_int
                {
                    TTRUE
                } else {
                    TFALSE
                };
                let bSameOrgFace: tbool = if iOF_1 == iOF_2 { TTRUE } else { TFALSE };
                let fCosS: libc::c_float = vOs.dot(vOs2);
                let fCosT: libc::c_float = vOt.dot(vOt2);
                assert!(f != t || bSameOrgFace != 0);
                if bAny != 0 || bSameOrgFace != 0 || fCosS > fThresCos && fCosT > fThresCos {
                    let fresh5 = iMembers;
                    iMembers += 1;
                    *pTmpMembers.offset(fresh5 as isize) = t;
                }
                j += 1;
            }
            tmp_group.iNrFaces = iMembers;
            tmp_group.pTriMembers = pTmpMembers;
            if iMembers > 1 as libc::c_int {
                let mut uSeed: libc::c_uint = INTERNAL_RND_SORT_SEED as libc::c_uint;
                QuickSort(
                    pTmpMembers,
                    0 as libc::c_int,
                    iMembers - 1 as libc::c_int,
                    uSeed,
                );
            }
            bFound = TFALSE;
            l = 0 as libc::c_int;
            while l < iUniqueSubGroups && bFound == 0 {
                bFound = CompareSubGroups(&mut tmp_group, &mut *pUniSubGroups.offset(l as isize));
                if bFound == 0 {
                    l += 1;
                }
            }
            assert!(bFound != 0 || l == iUniqueSubGroups);
            if bFound == 0 {
                let mut pIndices: *mut libc::c_int = malloc(
                    (::core::mem::size_of::<libc::c_int>() as libc::c_ulong)
                        .wrapping_mul(iMembers as libc::c_ulong),
                ) as *mut libc::c_int;
                if pIndices.is_null() {
                    let mut s_0: libc::c_int = 0 as libc::c_int;
                    s_0 = 0 as libc::c_int;
                    while s_0 < iUniqueSubGroups {
                        free(
                            (*pUniSubGroups.offset(s_0 as isize)).pTriMembers as *mut libc::c_void,
                        );
                        s_0 += 1;
                    }
                    free(pUniSubGroups as *mut libc::c_void);
                    free(pTmpMembers as *mut libc::c_void);
                    free(pSubGroupTspace as *mut libc::c_void);
                    return TFALSE;
                }
                (*pUniSubGroups.offset(iUniqueSubGroups as isize)).iNrFaces = iMembers;
                let fresh6 = &mut (*pUniSubGroups.offset(iUniqueSubGroups as isize)).pTriMembers;
                *fresh6 = pIndices;
                memcpy(
                    pIndices as *mut libc::c_void,
                    tmp_group.pTriMembers as *const libc::c_void,
                    (iMembers as libc::c_ulong)
                        .wrapping_mul(::core::mem::size_of::<libc::c_int>() as libc::c_ulong),
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
            let iOffs: libc::c_int = (*pTriInfos.offset(f as isize)).iTSpacesOffs;
            let iVert: libc::c_int =
                (*pTriInfos.offset(f as isize)).vert_num[index as usize] as libc::c_int;
            let mut pTS_out: *mut STSpace =
                &mut *psTspace.offset((iOffs + iVert) as isize) as *mut STSpace;
            assert!((*pTS_out).iCounter < 2 as libc::c_int);
            assert!(
                ((*pTriInfos.offset(f as isize)).iFlag & 8 as libc::c_int != 0 as libc::c_int)
                    as libc::c_int
                    == (*pGroup).bOrientPreservering
            );
            if (*pTS_out).iCounter == 1 as libc::c_int {
                *pTS_out = AvgTSpace(pTS_out, &mut *pSubGroupTspace.offset(l as isize));
                (*pTS_out).iCounter = 2 as libc::c_int;
                (*pTS_out).bOrient = (*pGroup).bOrientPreservering;
            } else {
                assert!((*pTS_out).iCounter == 0 as libc::c_int);
                *pTS_out = *pSubGroupTspace.offset(l as isize);
                (*pTS_out).iCounter = 1 as libc::c_int;
                (*pTS_out).bOrient = (*pGroup).bOrientPreservering;
            }
            i += 1;
        }
        s = 0 as libc::c_int;
        while s < iUniqueSubGroups {
            free((*pUniSubGroups.offset(s as isize)).pTriMembers as *mut libc::c_void);
            s += 1;
        }
        g += 1;
    }
    free(pUniSubGroups as *mut libc::c_void);
    free(pTmpMembers as *mut libc::c_void);
    free(pSubGroupTspace as *mut libc::c_void);
    TTRUE
}
unsafe extern "C" fn EvalTspace(
    mut face_indices: *mut libc::c_int,
    iFaces: libc::c_int,
    mut piTriListIn: *const libc::c_int,
    mut pTriInfos: *const STriInfo,
    mut pContext: *const SMikkTSpaceContext,
    iVertexRepresentitive: libc::c_int,
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
        bOrient: 0,
    };
    let mut fAngleSum: libc::c_float = 0 as libc::c_int as libc::c_float;
    let mut face: libc::c_int = 0 as libc::c_int;
    res.vOs.x = 0.0f32;
    res.vOs.y = 0.0f32;
    res.vOs.z = 0.0f32;
    res.vOt.x = 0.0f32;
    res.vOt.y = 0.0f32;
    res.vOt.z = 0.0f32;
    res.fMagS = 0 as libc::c_int as libc::c_float;
    res.fMagT = 0 as libc::c_int as libc::c_float;
    face = 0 as libc::c_int;
    while face < iFaces {
        let f: libc::c_int = *face_indices.offset(face as isize);
        if (*pTriInfos.offset(f as isize)).iFlag & GROUP_WITH_ANY == 0 as libc::c_int {
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
            let mut fCos: libc::c_float = 0.;
            let mut fAngle: libc::c_float = 0.;
            let mut fMagS: libc::c_float = 0.;
            let mut fMagT: libc::c_float = 0.;
            let mut i: libc::c_int = -(1 as libc::c_int);
            let mut index: libc::c_int = -(1 as libc::c_int);
            let mut i0: libc::c_int = -(1 as libc::c_int);
            let mut i1: libc::c_int = -(1 as libc::c_int);
            let mut i2: libc::c_int = -(1 as libc::c_int);
            if *piTriListIn.offset((3 as libc::c_int * f + 0 as libc::c_int) as isize)
                == iVertexRepresentitive
            {
                i = 0 as libc::c_int;
            } else if *piTriListIn.offset((3 as libc::c_int * f + 1 as libc::c_int) as isize)
                == iVertexRepresentitive
            {
                i = 1 as libc::c_int;
            } else if *piTriListIn.offset((3 as libc::c_int * f + 2 as libc::c_int) as isize)
                == iVertexRepresentitive
            {
                i = 2 as libc::c_int;
            }
            assert!(i >= 0 as libc::c_int && i < 3 as libc::c_int);
            index = *piTriListIn.offset((3 as libc::c_int * f + i) as isize);
            n = GetNormal(pContext, index);
            vOs = (*pTriInfos.offset(f as isize)).vOs
                - ((n.dot((*pTriInfos.offset(f as isize)).vOs)) * n);
            vOt = (*pTriInfos.offset(f as isize)).vOt
                - ((n.dot((*pTriInfos.offset(f as isize)).vOt)) * n);
            vOs.normalize_or_zero();
            vOt.normalize_or_zero();
            i2 = *piTriListIn.offset(
                (3 as libc::c_int * f
                    + (if i < 2 as libc::c_int {
                        i + 1 as libc::c_int
                    } else {
                        0 as libc::c_int
                    })) as isize,
            );
            i1 = *piTriListIn.offset((3 as libc::c_int * f + i) as isize);
            i0 = *piTriListIn.offset(
                (3 as libc::c_int * f
                    + (if i > 0 as libc::c_int {
                        i - 1 as libc::c_int
                    } else {
                        2 as libc::c_int
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
            fCos = if fCos > 1 as libc::c_int as libc::c_float {
                1 as libc::c_int as libc::c_float
            } else if fCos < -(1 as libc::c_int) as libc::c_float {
                -(1 as libc::c_int) as libc::c_float
            } else {
                fCos
            };
            fAngle = acos(fCos as libc::c_double) as libc::c_float;
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
    if fAngleSum > 0 as libc::c_int as libc::c_float {
        res.fMagS /= fAngleSum;
        res.fMagT /= fAngleSum;
    }
    res
}
unsafe extern "C" fn CompareSubGroups(
    mut pg1: *const SSubGroup,
    mut pg2: *const SSubGroup,
) -> tbool {
    let mut bStillSame: tbool = TTRUE;
    let mut i: libc::c_int = 0 as libc::c_int;
    if (*pg1).iNrFaces != (*pg2).iNrFaces {
        return TFALSE;
    }
    while i < (*pg1).iNrFaces && bStillSame != 0 {
        bStillSame = if *((*pg1).pTriMembers).offset(i as isize)
            == *((*pg2).pTriMembers).offset(i as isize)
        {
            TTRUE
        } else {
            TFALSE
        };
        if bStillSame != 0 {
            i += 1;
        }
    }
    bStillSame
}
unsafe extern "C" fn QuickSort(
    mut pSortBuffer: *mut libc::c_int,
    mut iLeft: libc::c_int,
    mut iRight: libc::c_int,
    mut uSeed: libc::c_uint,
) {
    let mut iL: libc::c_int = 0;
    let mut iR: libc::c_int = 0;
    let mut n: libc::c_int = 0;
    let mut index: libc::c_int = 0;
    let mut iMid: libc::c_int = 0;
    let mut iTmp: libc::c_int = 0;
    let mut t: libc::c_uint = uSeed & 31 as libc::c_int as libc::c_uint;
    t = uSeed.wrapping_shl(t) | uSeed.wrapping_shr((32 as libc::c_int as libc::c_uint).wrapping_sub(t));
    uSeed = uSeed
        .wrapping_add(t)
        .wrapping_add(3 as libc::c_int as libc::c_uint);
    iL = iLeft;
    iR = iRight;
    n = iR - iL + 1 as libc::c_int;
    assert!(n >= 0 as libc::c_int);
    index = uSeed.wrapping_rem(n as libc::c_uint) as libc::c_int;
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
    mut piTriListIn: *const libc::c_int,
    iNrTrianglesIn: libc::c_int,
) {
    let mut uSeed: libc::c_uint = INTERNAL_RND_SORT_SEED as libc::c_uint;
    let mut iEntries: libc::c_int = 0 as libc::c_int;
    let mut iCurStartIndex: libc::c_int = -(1 as libc::c_int);
    let mut f: libc::c_int = 0 as libc::c_int;
    let mut i: libc::c_int = 0 as libc::c_int;
    f = 0 as libc::c_int;
    while f < iNrTrianglesIn {
        i = 0 as libc::c_int;
        while i < 3 as libc::c_int {
            let i0: libc::c_int = *piTriListIn.offset((f * 3 as libc::c_int + i) as isize);
            let i1: libc::c_int = *piTriListIn.offset(
                (f * 3 as libc::c_int
                    + (if i < 2 as libc::c_int {
                        i + 1 as libc::c_int
                    } else {
                        0 as libc::c_int
                    })) as isize,
            );
            (*pEdges.offset((f * 3 as libc::c_int + i) as isize))
                .c2rust_unnamed
                .i0 = if i0 < i1 { i0 } else { i1 };
            (*pEdges.offset((f * 3 as libc::c_int + i) as isize))
                .c2rust_unnamed
                .i1 = if i0 >= i1 { i0 } else { i1 };
            (*pEdges.offset((f * 3 as libc::c_int + i) as isize))
                .c2rust_unnamed
                .f = f;
            i += 1;
        }
        f += 1;
    }
    QuickSortEdges(
        pEdges,
        0 as libc::c_int,
        iNrTrianglesIn * 3 as libc::c_int - 1 as libc::c_int,
        0 as libc::c_int,
        uSeed,
    );
    iEntries = iNrTrianglesIn * 3 as libc::c_int;
    iCurStartIndex = 0 as libc::c_int;
    i = 1 as libc::c_int;
    while i < iEntries {
        if (*pEdges.offset(iCurStartIndex as isize)).c2rust_unnamed.i0
            != (*pEdges.offset(i as isize)).c2rust_unnamed.i0
        {
            let iL: libc::c_int = iCurStartIndex;
            let iR: libc::c_int = i - 1 as libc::c_int;
            iCurStartIndex = i;
            QuickSortEdges(pEdges, iL, iR, 1 as libc::c_int, uSeed);
        }
        i += 1;
    }
    iCurStartIndex = 0 as libc::c_int;
    i = 1 as libc::c_int;
    while i < iEntries {
        if (*pEdges.offset(iCurStartIndex as isize)).c2rust_unnamed.i0
            != (*pEdges.offset(i as isize)).c2rust_unnamed.i0
            || (*pEdges.offset(iCurStartIndex as isize)).c2rust_unnamed.i1
                != (*pEdges.offset(i as isize)).c2rust_unnamed.i1
        {
            let iL_0: libc::c_int = iCurStartIndex;
            let iR_0: libc::c_int = i - 1 as libc::c_int;
            iCurStartIndex = i;
            QuickSortEdges(pEdges, iL_0, iR_0, 2 as libc::c_int, uSeed);
        }
        i += 1;
    }
    i = 0 as libc::c_int;
    while i < iEntries {
        let i0_0: libc::c_int = (*pEdges.offset(i as isize)).c2rust_unnamed.i0;
        let i1_0: libc::c_int = (*pEdges.offset(i as isize)).c2rust_unnamed.i1;
        let f_0: libc::c_int = (*pEdges.offset(i as isize)).c2rust_unnamed.f;
        let mut bUnassigned_A: tbool = 0;
        let mut i0_A: libc::c_int = 0;
        let mut i1_A: libc::c_int = 0;
        let mut edgenum_A: libc::c_int = 0;
        let mut edgenum_B: libc::c_int = 0 as libc::c_int;
        GetEdge(
            &mut i0_A,
            &mut i1_A,
            &mut edgenum_A,
            &*piTriListIn.offset((f_0 * 3 as libc::c_int) as isize),
            i0_0,
            i1_0,
        );
        bUnassigned_A = if (*pTriInfos.offset(f_0 as isize)).FaceNeighbors[edgenum_A as usize]
            == -(1 as libc::c_int)
        {
            TTRUE
        } else {
            TFALSE
        };
        if bUnassigned_A != 0 {
            let mut j: libc::c_int = i + 1 as libc::c_int;
            let mut t: libc::c_int = 0;
            let mut bNotFound: tbool = TTRUE;
            while j < iEntries
                && i0_0 == (*pEdges.offset(j as isize)).c2rust_unnamed.i0
                && i1_0 == (*pEdges.offset(j as isize)).c2rust_unnamed.i1
                && bNotFound != 0
            {
                let mut bUnassigned_B: tbool = 0;
                let mut i0_B: libc::c_int = 0;
                let mut i1_B: libc::c_int = 0;
                t = (*pEdges.offset(j as isize)).c2rust_unnamed.f;
                GetEdge(
                    &mut i1_B,
                    &mut i0_B,
                    &mut edgenum_B,
                    &*piTriListIn.offset((t * 3 as libc::c_int) as isize),
                    (*pEdges.offset(j as isize)).c2rust_unnamed.i0,
                    (*pEdges.offset(j as isize)).c2rust_unnamed.i1,
                );
                bUnassigned_B = if (*pTriInfos.offset(t as isize)).FaceNeighbors[edgenum_B as usize]
                    == -(1 as libc::c_int)
                {
                    TTRUE
                } else {
                    TFALSE
                };
                if i0_A == i0_B && i1_A == i1_B && bUnassigned_B != 0 {
                    bNotFound = TFALSE;
                } else {
                    j += 1;
                }
            }
            if bNotFound == 0 {
                let mut t_0: libc::c_int = (*pEdges.offset(j as isize)).c2rust_unnamed.f;
                (*pTriInfos.offset(f_0 as isize)).FaceNeighbors[edgenum_A as usize] = t_0;
                (*pTriInfos.offset(t_0 as isize)).FaceNeighbors[edgenum_B as usize] = f_0;
            }
        }
        i += 1;
    }
}
unsafe extern "C" fn BuildNeighborsSlow(
    mut pTriInfos: *mut STriInfo,
    mut piTriListIn: *const libc::c_int,
    iNrTrianglesIn: libc::c_int,
) {
    let mut f: libc::c_int = 0 as libc::c_int;
    let mut i: libc::c_int = 0 as libc::c_int;
    f = 0 as libc::c_int;
    while f < iNrTrianglesIn {
        i = 0 as libc::c_int;
        while i < 3 as libc::c_int {
            if (*pTriInfos.offset(f as isize)).FaceNeighbors[i as usize] == -(1 as libc::c_int) {
                let i0_A: libc::c_int = *piTriListIn.offset((f * 3 as libc::c_int + i) as isize);
                let i1_A: libc::c_int = *piTriListIn.offset(
                    (f * 3 as libc::c_int
                        + (if i < 2 as libc::c_int {
                            i + 1 as libc::c_int
                        } else {
                            0 as libc::c_int
                        })) as isize,
                );
                let mut bFound: tbool = TFALSE;
                let mut t: libc::c_int = 0 as libc::c_int;
                let mut j: libc::c_int = 0 as libc::c_int;
                while bFound == 0 && t < iNrTrianglesIn {
                    if t != f {
                        j = 0 as libc::c_int;
                        while bFound == 0 && j < 3 as libc::c_int {
                            let i1_B: libc::c_int =
                                *piTriListIn.offset((t * 3 as libc::c_int + j) as isize);
                            let i0_B: libc::c_int = *piTriListIn.offset(
                                (t * 3 as libc::c_int
                                    + (if j < 2 as libc::c_int {
                                        j + 1 as libc::c_int
                                    } else {
                                        0 as libc::c_int
                                    })) as isize,
                            );
                            if i0_A == i0_B && i1_A == i1_B {
                                bFound = TTRUE;
                            } else {
                                j += 1;
                            }
                        }
                    }
                    if bFound == 0 {
                        t += 1;
                    }
                }
                if bFound != 0 {
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
    mut iLeft: libc::c_int,
    mut iRight: libc::c_int,
    channel: libc::c_int,
    mut uSeed: libc::c_uint,
) {
    let mut t: libc::c_uint = 0;
    let mut iL: libc::c_int = 0;
    let mut iR: libc::c_int = 0;
    let mut n: libc::c_int = 0;
    let mut index: libc::c_int = 0;
    let mut iMid: libc::c_int = 0;
    let mut sTmp: SEdge = SEdge {
        c2rust_unnamed: C2RustUnnamed { i0: 0, i1: 0, f: 0 },
    };
    let iElems: libc::c_int = iRight - iLeft + 1 as libc::c_int;
    if iElems < 2 as libc::c_int {
        return;
    } else if iElems == 2 as libc::c_int {
        if (*pSortBuffer.offset(iLeft as isize)).array[channel as usize]
            > (*pSortBuffer.offset(iRight as isize)).array[channel as usize]
        {
            sTmp = *pSortBuffer.offset(iLeft as isize);
            *pSortBuffer.offset(iLeft as isize) = *pSortBuffer.offset(iRight as isize);
            *pSortBuffer.offset(iRight as isize) = sTmp;
        }
        return;
    }
    t = uSeed & 31 as libc::c_int as libc::c_uint;
    t = uSeed.wrapping_shl(t) | uSeed.wrapping_shr(((32 as libc::c_int as libc::c_uint).wrapping_sub(t)));
    uSeed = uSeed
        .wrapping_add(t)
        .wrapping_add(3 as libc::c_int as libc::c_uint);
    iL = iLeft;
    iR = iRight;
    n = iR - iL + 1 as libc::c_int;
    assert!(n >= 0 as libc::c_int);
    index = uSeed.wrapping_rem(n as libc::c_uint) as libc::c_int;
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
    mut i0_out: *mut libc::c_int,
    mut i1_out: *mut libc::c_int,
    mut edgenum_out: *mut libc::c_int,
    mut indices: *const libc::c_int,
    i0_in: libc::c_int,
    i1_in: libc::c_int,
) {
    *edgenum_out = -(1 as libc::c_int);
    if *indices.offset(0 as libc::c_int as isize) == i0_in
        || *indices.offset(0 as libc::c_int as isize) == i1_in
    {
        if *indices.offset(1 as libc::c_int as isize) == i0_in
            || *indices.offset(1 as libc::c_int as isize) == i1_in
        {
            *edgenum_out.offset(0 as libc::c_int as isize) = 0 as libc::c_int;
            *i0_out.offset(0 as libc::c_int as isize) = *indices.offset(0 as libc::c_int as isize);
            *i1_out.offset(0 as libc::c_int as isize) = *indices.offset(1 as libc::c_int as isize);
        } else {
            *edgenum_out.offset(0 as libc::c_int as isize) = 2 as libc::c_int;
            *i0_out.offset(0 as libc::c_int as isize) = *indices.offset(2 as libc::c_int as isize);
            *i1_out.offset(0 as libc::c_int as isize) = *indices.offset(0 as libc::c_int as isize);
        }
    } else {
        *edgenum_out.offset(0 as libc::c_int as isize) = 1 as libc::c_int;
        *i0_out.offset(0 as libc::c_int as isize) = *indices.offset(1 as libc::c_int as isize);
        *i1_out.offset(0 as libc::c_int as isize) = *indices.offset(2 as libc::c_int as isize);
    };
}
unsafe extern "C" fn DegenPrologue(
    mut pTriInfos: *mut STriInfo,
    mut piTriList_out: *mut libc::c_int,
    iNrTrianglesIn: libc::c_int,
    iTotTris: libc::c_int,
) {
    let mut iNextGoodTriangleSearchIndex: libc::c_int = -(1 as libc::c_int);
    let mut bStillFindingGoodOnes: tbool = 0;
    let mut t: libc::c_int = 0 as libc::c_int;
    while t < iTotTris - 1 as libc::c_int {
        let iFO_a: libc::c_int = (*pTriInfos.offset(t as isize)).iOrgFaceNumber;
        let iFO_b: libc::c_int =
            (*pTriInfos.offset((t + 1 as libc::c_int) as isize)).iOrgFaceNumber;
        if iFO_a == iFO_b {
            let bIsDeg_a: tbool =
                if (*pTriInfos.offset(t as isize)).iFlag & MARK_DEGENERATE != 0 as libc::c_int {
                    TTRUE
                } else {
                    TFALSE
                };
            let bIsDeg_b: tbool = if (*pTriInfos.offset((t + 1 as libc::c_int) as isize)).iFlag
                & MARK_DEGENERATE
                != 0 as libc::c_int
            {
                TTRUE
            } else {
                TFALSE
            };
            if bIsDeg_a ^ bIsDeg_b != 0 as libc::c_int {
                (*pTriInfos.offset(t as isize)).iFlag |= QUAD_ONE_DEGEN_TRI;
                (*pTriInfos.offset((t + 1 as libc::c_int) as isize)).iFlag |= QUAD_ONE_DEGEN_TRI;
            }
            t += 2 as libc::c_int;
        } else {
            t += 1;
        }
    }
    iNextGoodTriangleSearchIndex = 1 as libc::c_int;
    t = 0 as libc::c_int;
    bStillFindingGoodOnes = TTRUE;
    while t < iNrTrianglesIn && bStillFindingGoodOnes != 0 {
        let bIsGood: tbool =
            if (*pTriInfos.offset(t as isize)).iFlag & MARK_DEGENERATE == 0 as libc::c_int {
                TTRUE
            } else {
                TFALSE
            };
        if bIsGood != 0 {
            if iNextGoodTriangleSearchIndex < t + 2 as libc::c_int {
                iNextGoodTriangleSearchIndex = t + 2 as libc::c_int;
            }
        } else {
            let mut t0: libc::c_int = 0;
            let mut t1: libc::c_int = 0;
            let mut bJustADegenerate: tbool = TTRUE;
            while bJustADegenerate != 0 && iNextGoodTriangleSearchIndex < iTotTris {
                let bIsGood_0: tbool = if (*pTriInfos.offset(iNextGoodTriangleSearchIndex as isize))
                    .iFlag
                    & MARK_DEGENERATE
                    == 0 as libc::c_int
                {
                    TTRUE
                } else {
                    TFALSE
                };
                if bIsGood_0 != 0 {
                    bJustADegenerate = TFALSE;
                } else {
                    iNextGoodTriangleSearchIndex += 1;
                }
            }
            t0 = t;
            t1 = iNextGoodTriangleSearchIndex;
            iNextGoodTriangleSearchIndex += 1;
            assert!(iNextGoodTriangleSearchIndex > t + 1 as libc::c_int);
            if bJustADegenerate == 0 {
                let mut i: libc::c_int = 0 as libc::c_int;
                i = 0 as libc::c_int;
                while i < 3 as libc::c_int {
                    let index: libc::c_int =
                        *piTriList_out.offset((t0 * 3 as libc::c_int + i) as isize);
                    *piTriList_out.offset((t0 * 3 as libc::c_int + i) as isize) =
                        *piTriList_out.offset((t1 * 3 as libc::c_int + i) as isize);
                    *piTriList_out.offset((t1 * 3 as libc::c_int + i) as isize) = index;
                    i += 1;
                }
                let tri_info: STriInfo = *pTriInfos.offset(t0 as isize);
                *pTriInfos.offset(t0 as isize) = *pTriInfos.offset(t1 as isize);
                *pTriInfos.offset(t1 as isize) = tri_info;
            } else {
                bStillFindingGoodOnes = TFALSE;
            }
        }
        if bStillFindingGoodOnes != 0 {
            t += 1;
        }
    }
    assert!(bStillFindingGoodOnes != 0);
    assert!(iNrTrianglesIn == t);
}
unsafe extern "C" fn DegenEpilogue(
    mut psTspace: *mut STSpace,
    mut pTriInfos: *mut STriInfo,
    mut piTriListIn: *mut libc::c_int,
    mut pContext: *const SMikkTSpaceContext,
    iNrTrianglesIn: libc::c_int,
    iTotTris: libc::c_int,
) {
    let mut t: libc::c_int = 0 as libc::c_int;
    let mut i: libc::c_int = 0 as libc::c_int;
    t = iNrTrianglesIn;
    while t < iTotTris {
        let bSkip: tbool =
            if (*pTriInfos.offset(t as isize)).iFlag & QUAD_ONE_DEGEN_TRI != 0 as libc::c_int {
                TTRUE
            } else {
                TFALSE
            };
        if bSkip == 0 {
            i = 0 as libc::c_int;
            while i < 3 as libc::c_int {
                let index1: libc::c_int = *piTriListIn.offset((t * 3 as libc::c_int + i) as isize);
                let mut bNotFound: tbool = TTRUE;
                let mut j: libc::c_int = 0 as libc::c_int;
                while bNotFound != 0 && j < 3 as libc::c_int * iNrTrianglesIn {
                    let index2: libc::c_int = *piTriListIn.offset(j as isize);
                    if index1 == index2 {
                        bNotFound = TFALSE;
                    } else {
                        j += 1;
                    }
                }
                if bNotFound == 0 {
                    let iTri: libc::c_int = j / 3 as libc::c_int;
                    let iVert: libc::c_int = j % 3 as libc::c_int;
                    let iSrcVert: libc::c_int =
                        (*pTriInfos.offset(iTri as isize)).vert_num[iVert as usize] as libc::c_int;
                    let iSrcOffs: libc::c_int = (*pTriInfos.offset(iTri as isize)).iTSpacesOffs;
                    let iDstVert: libc::c_int =
                        (*pTriInfos.offset(t as isize)).vert_num[i as usize] as libc::c_int;
                    let iDstOffs: libc::c_int = (*pTriInfos.offset(t as isize)).iTSpacesOffs;
                    *psTspace.offset((iDstOffs + iDstVert) as isize) =
                        *psTspace.offset((iSrcOffs + iSrcVert) as isize);
                }
                i += 1;
            }
        }
        t += 1;
    }
    t = 0 as libc::c_int;
    while t < iNrTrianglesIn {
        if (*pTriInfos.offset(t as isize)).iFlag & QUAD_ONE_DEGEN_TRI != 0 as libc::c_int {
            let mut vDstP: SVec3 = SVec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            };
            let mut iOrgF: libc::c_int = -(1 as libc::c_int);
            let mut i_0: libc::c_int = 0 as libc::c_int;
            let mut bNotFound_0: tbool = 0;
            let mut pV: *mut libc::c_uchar =
                ((*pTriInfos.offset(t as isize)).vert_num).as_mut_ptr();
            let mut iFlag: libc::c_int = (1 as libc::c_int)
                << *pV.offset(0 as libc::c_int as isize) as libc::c_int
                | (1 as libc::c_int) << *pV.offset(1 as libc::c_int as isize) as libc::c_int
                | (1 as libc::c_int) << *pV.offset(2 as libc::c_int as isize) as libc::c_int;
            let mut iMissingIndex: libc::c_int = 0 as libc::c_int;
            if iFlag & 2 as libc::c_int == 0 as libc::c_int {
                iMissingIndex = 1 as libc::c_int;
            } else if iFlag & 4 as libc::c_int == 0 as libc::c_int {
                iMissingIndex = 2 as libc::c_int;
            } else if iFlag & 8 as libc::c_int == 0 as libc::c_int {
                iMissingIndex = 3 as libc::c_int;
            }
            iOrgF = (*pTriInfos.offset(t as isize)).iOrgFaceNumber;
            vDstP = GetPosition(pContext, MakeIndex(iOrgF, iMissingIndex));
            bNotFound_0 = TTRUE;
            i_0 = 0 as libc::c_int;
            while bNotFound_0 != 0 && i_0 < 3 as libc::c_int {
                let iVert_0: libc::c_int = *pV.offset(i_0 as isize) as libc::c_int;
                let vSrcP: SVec3 = GetPosition(pContext, MakeIndex(iOrgF, iVert_0));
                if vSrcP == vDstP {
                    let iOffs: libc::c_int = (*pTriInfos.offset(t as isize)).iTSpacesOffs;
                    *psTspace.offset((iOffs + iMissingIndex) as isize) =
                        *psTspace.offset((iOffs + iVert_0) as isize);
                    bNotFound_0 = TFALSE;
                } else {
                    i_0 += 1;
                }
            }
            assert!(bNotFound_0 == 0);
        }
        t += 1;
    }
}
