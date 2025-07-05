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

use alloc::{vec, vec::Vec};
use core::{
    ffi::{c_int, c_uchar, c_uint, c_ulong},
    marker::PhantomData,
    ops::Index,
};

use crate::{math::*, MikkTSpaceInterface};

#[repr(C)]
pub struct TangentSpace<O: Ops> {
    pub s: Vec3<O>,
    pub s_magnitude: f32,
    pub t: Vec3<O>,
    pub t_magnitude: f32,
    /// this is to average back into quads.
    pub counter: c_int,
    pub orientation_preserving: bool,
}

impl<O: Ops> Copy for TangentSpace<O> {}

impl<O: Ops> Clone for TangentSpace<O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, O: Ops> From<&'a TangentSpace<O>> for crate::TangentSpace {
    fn from(value: &'a TangentSpace<O>) -> Self {
        let tangent = [value.s.x as f32, value.s.y as f32, value.s.z as f32];
        let bi_tangent = [value.t.x as f32, value.t.y as f32, value.t.z as f32];
        let tangent_magnitude = value.s_magnitude;
        let bi_tangent_magnitude = value.t_magnitude;

        crate::TangentSpace {
            tangent,
            bi_tangent,
            mag_s: tangent_magnitude,
            mag_t: bi_tangent_magnitude,
            is_orientation_preserving: value.orientation_preserving,
        }
    }
}

impl<O: Ops> From<TangentSpace<O>> for crate::TangentSpace {
    fn from(value: TangentSpace<O>) -> Self {
        crate::TangentSpace::from(&value)
    }
}

impl<O: Ops> TangentSpace<O> {
    pub const ZERO: TangentSpace<O> = TangentSpace {
        s: Vec3::ZERO,
        s_magnitude: 0.,
        t: Vec3::ZERO,
        t_magnitude: 0.,
        counter: 0,
        orientation_preserving: false,
    };
}

#[repr(C)]
pub struct TriangleInfo<O: Ops> {
    pub face_neighbors: [c_int; 3],
    pub assigned_group: [Option<usize>; 3],

    /// normalized first order face derivative
    pub s: Vec3<O>,
    /// normalized first order face derivative
    pub t: Vec3<O>,

    /// original magnitude of vOs
    pub s_magnitude: f32,
    /// original magnitude of vOs
    pub t_magnitude: f32,

    /// determines if the current and the next triangle are a quad.
    pub original_face_index: c_int,

    pub flags: c_int,
    pub tangent_spaces_offset: c_int,
    pub vertex_indices: [c_uchar; 4],
}

impl<O: Ops> Copy for TriangleInfo<O> {}

impl<O: Ops> Clone for TriangleInfo<O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<O: Ops> TriangleInfo<O> {
    pub const ZERO: TriangleInfo<O> = TriangleInfo {
        face_neighbors: [0; 3],
        assigned_group: [None; 3],
        s: Vec3::ZERO,
        t: Vec3::ZERO,
        s_magnitude: 0.,
        t_magnitude: 0.,
        original_face_index: 0,
        flags: 0,
        tangent_spaces_offset: 0,
        vertex_indices: [0; 4],
    };
}

#[derive(Clone)]
#[repr(C)]
pub struct Group {
    pub id: usize,
    pub face_indices: Vec<c_int>,
    pub vertex_representative: c_int,
    pub orientation_preserving: bool,
}

impl Group {
    pub const ZERO: Group = Group {
        id: 0,
        face_indices: Vec::new(),
        vertex_representative: 0,
        orientation_preserving: false,
    };
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Edge {
    pub i0: c_int,
    pub i1: c_int,
    pub f: c_int,
}

impl Edge {
    pub const ZERO: Edge = Edge { i0: 0, i1: 0, f: 0 };
}

impl Index<usize> for Edge {
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

#[repr(C)]
pub struct TemporaryVertex<O: Ops> {
    pub vert: Vec3<O>,
    pub index: c_int,
}

impl<O: Ops> Copy for TemporaryVertex<O> {}

impl<O: Ops> Clone for TemporaryVertex<O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<O: Ops> TemporaryVertex<O> {
    pub const ZERO: TemporaryVertex<O> = TemporaryVertex {
        vert: Vec3::ZERO,
        index: 0,
    };
}

pub const INTERNAL_RND_SORT_SEED: c_int = 39871946;
pub const MARK_DEGENERATE: c_int = 1;
pub const QUAD_ONE_DEGEN_TRI: c_int = 2;
pub const GROUP_WITH_ANY: c_int = 4;
pub const ORIENT_PRESERVING: c_int = 8;

fn as_index(face: c_int, vertex: c_int) -> c_int {
    assert!(vertex >= 0 && vertex < 4 && face >= 0);
    face << 2 | vertex & 0x3
}

fn from_index(index: c_int) -> (c_int, c_int) {
    (index >> 2, index & 0x3)
}

fn mean_tangent_space<O: Ops>(lhs: TangentSpace<O>, rhs: TangentSpace<O>) -> TangentSpace<O> {
    let mut ts_res: TangentSpace<O> = TangentSpace {
        s: Vec3::ZERO,
        s_magnitude: 0.,
        t: Vec3::ZERO,
        t_magnitude: 0.,
        counter: 0,
        orientation_preserving: false,
    };

    // this if is important. Due to floating point precision
    // averaging when ts0==ts1 will cause a slight difference
    // which results in tangent space splits later on
    if lhs.s_magnitude == rhs.s_magnitude
        && lhs.t_magnitude == rhs.t_magnitude
        && (lhs.s == rhs.s)
        && (lhs.t == rhs.t)
    {
        ts_res.s_magnitude = lhs.s_magnitude;
        ts_res.t_magnitude = lhs.t_magnitude;
        ts_res.s = lhs.s;
        ts_res.t = lhs.t;
    } else {
        ts_res.s_magnitude = 0.5f32 * (lhs.s_magnitude + rhs.s_magnitude);
        ts_res.t_magnitude = 0.5f32 * (lhs.t_magnitude + rhs.t_magnitude);
        ts_res.s = lhs.s + rhs.s;
        ts_res.t = lhs.t + rhs.t;
        ts_res.s.normalize_or_zero();
        ts_res.t.normalize_or_zero();
    }
    ts_res
}

pub fn generate_tangent_space_default<I: MikkTSpaceInterface<O>, O: Ops>(context: &mut I) -> bool {
    generate_tangent_space(context, 180.0f32)
}

pub fn generate_tangent_space<I: MikkTSpaceInterface<O>, O: Ops>(
    context: &mut I,
    angular_threshold: f32,
) -> bool {
    let mut triangles_count: c_int = 0;
    let threshold_cos: f32 = O::cos(deg_to_rad(angular_threshold) as f64) as f32;

    // count triangles on supported faces
    let faces_total = context.get_num_faces();
    let mut f = 0;
    while f < faces_total {
        let verts = context.get_num_vertices_of_face(f as usize);
        if verts == 3 {
            triangles_count += 1;
        } else if verts == 4 {
            triangles_count += 2;
        }
        f += 1;
    }
    if triangles_count <= 0 {
        return false;
    }

    // allocate memory for an index list
    let mut triangle_vertex_list: Vec<c_int> =
        vec![0; (triangles_count as c_ulong).wrapping_mul(3) as usize];
    let mut triangle_info_list: Vec<TriangleInfo<O>> =
        vec![TriangleInfo::ZERO; triangles_count as usize];

    // make an initial triangle --> face index list
    let tangent_spaces_total = generate_initial_vertices_index_list(
        &mut triangle_info_list,
        &mut triangle_vertex_list,
        context,
        triangles_count,
    );

    // make a welded index list of identical positions and attributes (pos, norm, texc)
    generate_shared_vertices_index_list(&mut triangle_vertex_list, context, triangles_count);

    // Mark all degenerate triangles
    let triangles_total_count = triangles_count;
    let mut triangles_degenerate_total = 0;
    let mut t = 0;
    while t < triangles_count {
        let i0: c_int = triangle_vertex_list[(t * 3 + 0) as usize];
        let i1: c_int = triangle_vertex_list[(t * 3 + 1) as usize];
        let i2: c_int = triangle_vertex_list[(t * 3 + 2) as usize];
        let p0: Vec3<O> = get_position_from_index(context, i0);
        let p1: Vec3<O> = get_position_from_index(context, i1);
        let p2: Vec3<O> = get_position_from_index(context, i2);
        if (p0 == p1) || (p0 == p2) || (p1 == p2) {
            // degenerate
            triangle_info_list[t as usize].flags |= MARK_DEGENERATE;
            triangles_degenerate_total += 1;
        }
        t += 1;
    }
    triangles_count = triangles_total_count - triangles_degenerate_total;

    // mark all triangle pairs that belong to a quad with only one
    // good triangle. These need special treatment in DegenEpilogue().
    // Additionally, move all good triangles to the start of
    // triangle_info_list[] and triangle_vertex_list[] without changing order and
    // put the degenerate triangles last.
    degen_prologue(
        &mut triangle_info_list,
        &mut triangle_vertex_list,
        triangles_count,
        triangles_total_count,
    );

    // evaluate triangle level attributes and neighbor list
    initialize_triangle_info(
        &mut triangle_info_list,
        &triangle_vertex_list,
        context,
        triangles_count,
    );

    // based on the 4 rules, identify groups based on connectivity
    let groups_max_count = triangles_count * 3;
    let mut groups: Vec<Group> = vec![Group::ZERO; groups_max_count as usize];
    let groups_active = build_4_rule_groups(
        &mut triangle_info_list,
        &mut groups,
        &triangle_vertex_list,
        triangles_count,
    );

    let mut tangent_spaces: Vec<TangentSpace<O>> =
        vec![TangentSpace::ZERO; tangent_spaces_total as usize];
    t = 0;
    while t < tangent_spaces_total {
        tangent_spaces[t as usize] = TangentSpace {
            s: Vec3::<O> {
                x: 1.0,
                y: 0.0,
                z: 0.0,
                _phantom: PhantomData,
            },
            s_magnitude: 1.0,
            t: Vec3::<O> {
                x: 0.0,
                y: 1.0,
                z: 0.0,
                _phantom: PhantomData,
            },
            t_magnitude: 1.0,
            ..TangentSpace::ZERO
        };
        t += 1;
    }

    // make tspaces, each group is split up into subgroups if necessary
    // based on fAngularThreshold. Finally a tangent space is made for
    // every resulting subgroup
    let result = generate_tangent_spaces(
        &mut tangent_spaces,
        &triangle_info_list,
        &groups,
        groups_active,
        &triangle_vertex_list,
        threshold_cos,
        context,
    );
    // if an allocation in GenerateTSpaces() failed
    if !result {
        return false;
    }

    // degenerate quads with one good triangle will be fixed by copying a space from
    // the good triangle to the coinciding vertex.
    // all other degenerate triangles will just copy a space from any good triangle
    // with the same welded index in triangle_vertex_list[].
    degen_epilogue(
        &mut tangent_spaces,
        &triangle_info_list,
        &triangle_vertex_list,
        context,
        triangles_count,
        triangles_total_count,
    );
    let mut index = 0;
    f = 0;
    while f < faces_total {
        let verts_0: c_int = context.get_num_vertices_of_face(f as usize) as c_int;
        if !(verts_0 != 3 && verts_0 != 4) {
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
            let mut i = 0;
            while i < verts_0 {
                let tangent_space = &tangent_spaces[index as usize];

                context.set_tangent_space(tangent_space.into(), f as usize, i as usize);
                index += 1;
                i += 1;
            }
        }
        f += 1;
    }
    true
}

const CELLS: c_int = 2048;

// it is IMPORTANT that this function is called to evaluate the hash since
// inlining could potentially reorder instructions and generate different
// results for the same effective input value fVal.
#[inline(never)]
fn find_grid_cell(min: f32, max: f32, val: f32) -> c_int {
    let face: f32 = CELLS as f32 * ((val - min) / (max - min));
    let vertex: c_int = face as c_int;
    if vertex < CELLS {
        if vertex >= 0 {
            vertex
        } else {
            0
        }
    } else {
        CELLS - 1
    }
}

fn generate_shared_vertices_index_list<I: MikkTSpaceInterface<O>, O: Ops>(
    triangle_vertices: &mut [c_int],
    context: &I,
    triangle_count: c_int,
) {
    // Generate bounding box
    let mut min: Vec3<O> = get_position_from_index(context, 0);
    let mut max: Vec3<O> = min;
    let mut i = 1;
    while i < triangle_count * 3 {
        let index: c_int = triangle_vertices[i as usize];
        let position: Vec3<O> = get_position_from_index(context, index);
        if min.x > position.x {
            min.x = position.x;
        } else if max.x < position.x {
            max.x = position.x;
        }
        if min.y > position.y {
            min.y = position.y;
        } else if max.y < position.y {
            max.y = position.y;
        }
        if min.z > position.z {
            min.z = position.z;
        } else if max.z < position.z {
            max.z = position.z;
        }
        i += 1;
    }
    let delta = max - min;
    let mut channel = 0;
    let mut min_channel = min.x;
    let mut max_channel = max.x;
    if delta.y > delta.x && delta.y > delta.z {
        channel = 1;
        min_channel = min.y;
        max_channel = max.y;
    } else if delta.z > delta.x {
        channel = 2;
        min_channel = min.z;
        max_channel = max.z;
    }

    // if /* can't allocate? */ {
    //     GenerateSharedVerticesIndexListSlow(piTriList_in_and_out, context, iNrTrianglesIn);
    //     return;
    // }

    // make allocations
    let mut hash_table: Vec<c_int> = vec![0; (triangle_count as c_ulong).wrapping_mul(3) as usize];
    let mut hash_count: Vec<c_int> = vec![0; CELLS as usize];
    let mut hash_offsets: Vec<c_int> = vec![0; CELLS as usize];
    let mut hash_count_2: Vec<c_int> = vec![0; CELLS as usize];

    // count amount of elements in each cell unit
    i = 0;
    while i < triangle_count * 3 {
        let index_0: c_int = triangle_vertices[i as usize];
        let position: Vec3<O> = get_position_from_index(context, index_0);
        let val: f32 = if channel == 0 {
            position.x
        } else if channel == 1 {
            position.y
        } else {
            position.z
        };
        let cell: c_int = find_grid_cell(min_channel, max_channel, val);
        let fresh0 = &mut hash_count[cell as usize];
        *fresh0 += 1;
        i += 1;
    }

    // evaluate start index of each cell.
    hash_offsets[0 as usize] = 0;
    let mut k = 1;
    while k < CELLS {
        hash_offsets[k as usize] = hash_offsets[(k - 1) as usize] + hash_count[(k - 1) as usize];
        k += 1;
    }

    // insert vertices
    i = 0;
    while i < triangle_count * 3 {
        let index_1: c_int = triangle_vertices[i as usize];
        let position: Vec3<O> = get_position_from_index(context, index_1);
        let val: f32 = if channel == 0 {
            position.x
        } else if channel == 1 {
            position.y
        } else {
            position.z
        };
        let cell: c_int = find_grid_cell(min_channel, max_channel, val);
        assert!(hash_count_2[cell as usize] < hash_count[cell as usize]);
        let entry = &mut hash_table
            [hash_offsets[cell as usize] as usize + hash_count_2[cell as usize] as usize];
        *entry = i; // vertex i has been inserted.
        let fresh1 = &mut hash_count_2[cell as usize];
        *fresh1 += 1;
        i += 1;
    }

    // verify the count
    k = 0;
    while k < CELLS {
        assert!(hash_count_2[k as usize] == hash_count[k as usize]);
        k += 1;
    }

    // find maximum amount of entries in any hash entry
    let mut max_count = hash_count[0 as usize];
    k = 1;
    while k < CELLS {
        if max_count < hash_count[k as usize] {
            max_count = hash_count[k as usize];
        }
        k += 1;
    }

    // complete the merge
    let mut temporary_vertices: Vec<TemporaryVertex<O>> =
        vec![TemporaryVertex::<O>::ZERO; max_count as usize];
    k = 0;
    while k < CELLS {
        let entries: c_int = hash_count[k as usize];
        if entries >= 2 {
            // if /* couldn't allocate pTmpVert? */ {
            //     MergeVertsSlow(
            //         piTriList_in_and_out,
            //         context,
            //         pTable_0 as *const c_int,
            //         iEntries,
            //     );
            // }
            let mut e = 0;
            while e < entries {
                let i_0: c_int = hash_table[hash_offsets[k as usize] as usize + e as usize];
                let position: Vec3<O> =
                    get_position_from_index(context, triangle_vertices[i_0 as usize]);
                temporary_vertices[e as usize].vert = position;
                temporary_vertices[e as usize].index = i_0;
                e += 1;
            }
            merge_verts_fast(
                triangle_vertices,
                &mut temporary_vertices,
                context,
                0,
                entries - 1,
            );
        }
        k += 1;
    }
}

fn merge_verts_fast<I: MikkTSpaceInterface<O>, O: Ops>(
    triangle_verticies: &mut [c_int],
    temporary_verticies: &mut [TemporaryVertex<O>],
    context: &I,
    i_left_in: c_int,
    i_right_in: c_int,
) {
    // make bbox
    let mut min: [f32; 3] = [0.; 3];
    let mut max: [f32; 3] = [0.; 3];

    let mut c = 0;
    while c < 3 {
        min[c as usize] = temporary_verticies[i_left_in as usize].vert[c as usize];
        max[c as usize] = min[c as usize];
        c += 1;
    }
    let mut l = i_left_in + 1;
    while l <= i_right_in {
        c = 0;
        while c < 3 {
            if min[c as usize] > temporary_verticies[l as usize].vert[c as usize] {
                min[c as usize] = temporary_verticies[l as usize].vert[c as usize];
            }
            if max[c as usize] < temporary_verticies[l as usize].vert[c as usize] {
                max[c as usize] = temporary_verticies[l as usize].vert[c as usize];
            }
            c += 1;
        }
        l += 1;
    }

    let dx = max[0 as usize] - min[0 as usize];
    let dy = max[1 as usize] - min[1 as usize];
    let dz = max[2 as usize] - min[2 as usize];

    let mut channel = 0;
    if dy > dx && dy > dz {
        channel = 1;
    } else if dz > dx {
        channel = 2;
    }

    let sep = 0.5f32 * (max[channel as usize] + min[channel as usize]);

    // stop if all vertices are NaNs
    if sep.is_finite() as i32 == 0 {
        return;
    }

    // terminate recursion when the separation/average value
    // is no longer strictly between fMin and fMax values.
    if sep >= max[channel as usize] || sep <= min[channel as usize] {
        // complete the weld
        l = i_left_in;
        while l <= i_right_in {
            let i: c_int = temporary_verticies[l as usize].index;
            let index: c_int = triangle_verticies[i as usize];
            let position: Vec3<O> = get_position_from_index(context, index);
            let normal: Vec3<O> = get_normal_from_index(context, index);
            let texture_position: Vec3<O> = get_texture_coordinate_from_index(context, index);

            let mut not_found: bool = true;
            let mut l2: c_int = i_left_in;
            let mut i2rec: c_int = -(1);
            while l2 < l && not_found {
                let i2: c_int = temporary_verticies[l2 as usize].index;
                let index2: c_int = triangle_verticies[i2 as usize];
                let position_other: Vec3<O> = get_position_from_index(context, index2);
                let normal_other: Vec3<O> = get_normal_from_index(context, index2);
                let texture_position_other: Vec3<O> =
                    get_texture_coordinate_from_index(context, index2);
                i2rec = i2;

                //if (vP==vP2 && vN==vN2 && vT==vT2)
                if position.x == position_other.x
                    && position.y == position_other.y
                    && position.z == position_other.z
                    && normal.x == normal_other.x
                    && normal.y == normal_other.y
                    && normal.z == normal_other.z
                    && texture_position.x == texture_position_other.x
                    && texture_position.y == texture_position_other.y
                    && texture_position.z == texture_position_other.z
                {
                    not_found = false;
                } else {
                    l2 += 1;
                }
            }

            // merge if previously found
            if !not_found {
                triangle_verticies[i as usize] = triangle_verticies[i2rec as usize];
            }

            l += 1;
        }
    } else {
        let mut i_left: c_int = i_left_in;
        let mut i_right: c_int = i_right_in;
        assert!(i_right_in - i_left_in > 0, "at least 2 entries");

        // separate (by fSep) all points between iL_in and iR_in in pTmpVert[]
        while i_left < i_right {
            let mut ready_left_swap: bool = false;
            let mut ready_right_swap: bool = false;
            while !ready_left_swap && i_left < i_right {
                assert!(i_left >= i_left_in && i_left <= i_right_in);
                ready_left_swap =
                    temporary_verticies[i_left as usize].vert[channel as usize] >= sep;
                if !ready_left_swap {
                    i_left += 1;
                }
            }
            while !ready_right_swap && i_left < i_right {
                assert!(i_right >= i_left_in && i_right <= i_right_in);
                ready_right_swap =
                    temporary_verticies[i_right as usize].vert[channel as usize] < sep;
                if !ready_right_swap {
                    i_right -= 1;
                }
            }
            assert!(i_left < i_right || !(ready_left_swap && ready_right_swap));

            if ready_left_swap && ready_right_swap {
                let temporary_vertex: TemporaryVertex<O> = temporary_verticies[i_left as usize];
                assert!(i_left < i_right);
                temporary_verticies[i_left as usize] = temporary_verticies[i_right as usize];
                temporary_verticies[i_right as usize] = temporary_vertex;
                i_left += 1;
                i_right -= 1;
            }
        }

        assert!(i_left == i_right + 1 || i_left == i_right);
        if i_left == i_right {
            let ready_right_swap: bool =
                temporary_verticies[i_right as usize].vert[channel as usize] < sep;
            if ready_right_swap {
                i_left += 1;
            } else {
                i_right -= 1;
            }
        }

        // only need to weld when there is more than 1 instance of the (x,y,z)
        if i_left_in < i_right {
            // weld all left of fSep
            merge_verts_fast(
                triangle_verticies,
                temporary_verticies,
                context,
                i_left_in,
                i_right,
            );
        }
        if i_left < i_right_in {
            // weld all right of (or equal to) fSep
            merge_verts_fast(
                triangle_verticies,
                temporary_verticies,
                context,
                i_left,
                i_right_in,
            );
        }
    };
}

fn generate_initial_vertices_index_list<I: MikkTSpaceInterface<O>, O: Ops>(
    triangle_info_list: &mut [TriangleInfo<O>],
    triangle_verticies: &mut [c_int],
    context: &I,
    triangle_count: c_int,
) -> c_int {
    let mut tangent_space_offset = 0;
    let mut destination_triangle_info_index: c_int = 0;
    let mut f = 0;
    while f < context.get_num_faces() as c_int {
        let verts = context.get_num_vertices_of_face(f as usize);
        if !(verts != 3 && verts != 4) {
            triangle_info_list[destination_triangle_info_index as usize].original_face_index = f;
            triangle_info_list[destination_triangle_info_index as usize].tangent_spaces_offset =
                tangent_space_offset;
            if verts == 3 {
                let verticies = &mut triangle_info_list[destination_triangle_info_index as usize]
                    .vertex_indices;
                verticies[0] = 0 as c_uchar;
                verticies[1] = 1 as c_uchar;
                verticies[2] = 2 as c_uchar;
                triangle_verticies[(destination_triangle_info_index * 3 + 0) as usize] =
                    as_index(f, 0);
                triangle_verticies[(destination_triangle_info_index * 3 + 1) as usize] =
                    as_index(f, 1);
                triangle_verticies[(destination_triangle_info_index * 3 + 2) as usize] =
                    as_index(f, 2);
                destination_triangle_info_index += 1;
            } else {
                triangle_info_list[(destination_triangle_info_index + 1) as usize]
                    .original_face_index = f;
                triangle_info_list[(destination_triangle_info_index + 1) as usize]
                    .tangent_spaces_offset = tangent_space_offset;

                // need an order independent way to evaluate
                // tspace on quads. This is done by splitting
                // along the shortest diagonal.
                let i0: c_int = as_index(f, 0);
                let i1: c_int = as_index(f, 1);
                let i2: c_int = as_index(f, 2);
                let i3: c_int = as_index(f, 3);
                let tx0: Vec3<O> = get_texture_coordinate_from_index(context, i0);
                let tx1: Vec3<O> = get_texture_coordinate_from_index(context, i1);
                let tx2: Vec3<O> = get_texture_coordinate_from_index(context, i2);
                let tx3: Vec3<O> = get_texture_coordinate_from_index(context, i3);
                let distance_squared_20: f32 = (tx2 - tx0).length_squared();
                let distance_squared_13: f32 = (tx3 - tx1).length_squared();
                let quad_diagonal_is_02 = if distance_squared_20 < distance_squared_13 {
                    true
                } else if distance_squared_13 < distance_squared_20 {
                    false
                } else {
                    let p0: Vec3<O> = get_position_from_index(context, i0);
                    let p1: Vec3<O> = get_position_from_index(context, i1);
                    let p2: Vec3<O> = get_position_from_index(context, i2);
                    let p3: Vec3<O> = get_position_from_index(context, i3);
                    let distance_squared_20: f32 = (p2 - p0).length_squared();
                    let distance_squared_13: f32 = (p3 - p1).length_squared();
                    distance_squared_13 >= distance_squared_20
                };
                if quad_diagonal_is_02 {
                    let verticies_a = &mut triangle_info_list
                        [destination_triangle_info_index as usize]
                        .vertex_indices;
                    verticies_a[0] = 0 as c_uchar;
                    verticies_a[1] = 1 as c_uchar;
                    verticies_a[2] = 2 as c_uchar;
                    triangle_verticies[(destination_triangle_info_index * 3 + 0) as usize] = i0;
                    triangle_verticies[(destination_triangle_info_index * 3 + 1) as usize] = i1;
                    triangle_verticies[(destination_triangle_info_index * 3 + 2) as usize] = i2;
                    destination_triangle_info_index += 1;
                    let verticies_b = &mut triangle_info_list
                        [destination_triangle_info_index as usize]
                        .vertex_indices;
                    verticies_b[0] = 0 as c_uchar;
                    verticies_b[1] = 2 as c_uchar;
                    verticies_b[2] = 3 as c_uchar;
                    triangle_verticies[(destination_triangle_info_index * 3 + 0) as usize] = i0;
                    triangle_verticies[(destination_triangle_info_index * 3 + 1) as usize] = i2;
                    triangle_verticies[(destination_triangle_info_index * 3 + 2) as usize] = i3;
                    destination_triangle_info_index += 1;
                } else {
                    let verticies_a = &mut triangle_info_list
                        [destination_triangle_info_index as usize]
                        .vertex_indices;
                    verticies_a[0] = 0 as c_uchar;
                    verticies_a[1] = 1 as c_uchar;
                    verticies_a[2] = 3 as c_uchar;
                    triangle_verticies[(destination_triangle_info_index * 3 + 0) as usize] = i0;
                    triangle_verticies[(destination_triangle_info_index * 3 + 1) as usize] = i1;
                    triangle_verticies[(destination_triangle_info_index * 3 + 2) as usize] = i3;
                    destination_triangle_info_index += 1;
                    let verticies_b = &mut triangle_info_list
                        [destination_triangle_info_index as usize]
                        .vertex_indices;
                    verticies_b[0] = 1 as c_uchar;
                    verticies_b[1] = 2 as c_uchar;
                    verticies_b[2] = 3 as c_uchar;
                    triangle_verticies[(destination_triangle_info_index * 3 + 0) as usize] = i1;
                    triangle_verticies[(destination_triangle_info_index * 3 + 1) as usize] = i2;
                    triangle_verticies[(destination_triangle_info_index * 3 + 2) as usize] = i3;
                    destination_triangle_info_index += 1;
                }
            }
            tangent_space_offset += verts as c_int;
            assert!(destination_triangle_info_index <= triangle_count);
        }
        f += 1;
    }

    let mut t = 0;
    while t < triangle_count {
        triangle_info_list[t as usize].flags = 0;
        t += 1;
    }

    // return total amount of tspaces
    tangent_space_offset
}

fn get_position_from_index<I: MikkTSpaceInterface<O>, O: Ops>(
    context: &I,
    index: c_int,
) -> Vec3<O> {
    let mut res: Vec3<O> = Vec3::ZERO;
    let (face, vertex) = from_index(index);
    let pos = context.get_position(face as usize, vertex as usize);
    res.x = pos[0 as usize];
    res.y = pos[1 as usize];
    res.z = pos[2 as usize];
    res
}

fn get_normal_from_index<I: MikkTSpaceInterface<O>, O: Ops>(context: &I, index: c_int) -> Vec3<O> {
    let mut res: Vec3<O> = Vec3::ZERO;
    let (face, vertex) = from_index(index);
    let norm = context.get_normal(face as usize, vertex as usize);
    res.x = norm[0 as usize];
    res.y = norm[1 as usize];
    res.z = norm[2 as usize];
    res
}

fn get_texture_coordinate_from_index<I: MikkTSpaceInterface<O>, O: Ops>(
    context: &I,
    index: c_int,
) -> Vec3<O> {
    let mut res: Vec3<O> = Vec3::ZERO;
    let (face, vertex) = from_index(index);
    let texc = context.get_tex_coord(face as usize, vertex as usize);
    res.x = texc[0 as usize];
    res.y = texc[1 as usize];
    res.z = 1.0f32;
    res
}

/// returns the texture area times 2
fn calculate_texture_area<I: MikkTSpaceInterface<O>, O: Ops>(
    context: &I,
    indices: &[c_int],
) -> f32 {
    let t1: Vec3<O> = get_texture_coordinate_from_index(context, indices[0]);
    let t2: Vec3<O> = get_texture_coordinate_from_index(context, indices[1]);
    let t3: Vec3<O> = get_texture_coordinate_from_index(context, indices[2]);

    let t21x: f32 = t2.x - t1.x;
    let t21y: f32 = t2.y - t1.y;
    let t31x: f32 = t3.x - t1.x;
    let t31y: f32 = t3.y - t1.y;

    let signed_area_double: f32 = t21x * t31y - t21y * t31x;
    if signed_area_double < 0 as f32 {
        -signed_area_double
    } else {
        signed_area_double
    }
}

fn initialize_triangle_info<I: MikkTSpaceInterface<O>, O: Ops>(
    triangle_info_list: &mut [TriangleInfo<O>],
    triangle_vertex_list: &[c_int],
    context: &I,
    triangle_count: c_int,
) {
    let mut t: c_int = 0;
    // triangle_info_list[f].iFlag is cleared in GenerateInitialVerticesIndexList() which is called before this function.

    // generate neighbor info list
    let mut f = 0;
    while f < triangle_count {
        let mut i = 0;
        while i < 3 {
            triangle_info_list[f as usize].face_neighbors[i as usize] = -(1);
            let fresh2 = &mut triangle_info_list[f as usize].assigned_group[i as usize];
            *fresh2 = None;
            triangle_info_list[f as usize].s.x = 0.0f32;
            triangle_info_list[f as usize].s.y = 0.0f32;
            triangle_info_list[f as usize].s.z = 0.0f32;
            triangle_info_list[f as usize].t.x = 0.0f32;
            triangle_info_list[f as usize].t.y = 0.0f32;
            triangle_info_list[f as usize].t.z = 0.0f32;
            triangle_info_list[f as usize].s_magnitude = 0 as f32;
            triangle_info_list[f as usize].t_magnitude = 0 as f32;

            // assumed bad
            triangle_info_list[f as usize].flags |= GROUP_WITH_ANY;

            i += 1;
        }
        f += 1;
    }

    // evaluate first order derivatives
    f = 0;
    while f < triangle_count {
        // initial values
        let v1: Vec3<O> =
            get_position_from_index(context, triangle_vertex_list[(f * 3 + 0) as usize]);
        let v2: Vec3<O> =
            get_position_from_index(context, triangle_vertex_list[(f * 3 + 1) as usize]);
        let v3: Vec3<O> =
            get_position_from_index(context, triangle_vertex_list[(f * 3 + 2) as usize]);
        let t1: Vec3<O> =
            get_texture_coordinate_from_index(context, triangle_vertex_list[(f * 3 + 0) as usize]);
        let t2: Vec3<O> =
            get_texture_coordinate_from_index(context, triangle_vertex_list[(f * 3 + 1) as usize]);
        let t3: Vec3<O> =
            get_texture_coordinate_from_index(context, triangle_vertex_list[(f * 3 + 2) as usize]);

        let t21x: f32 = t2.x - t1.x;
        let t21y: f32 = t2.y - t1.y;
        let t31x: f32 = t3.x - t1.x;
        let t31y: f32 = t3.y - t1.y;
        let d1: Vec3<O> = v2 - v1;
        let d2: Vec3<O> = v3 - v1;
        let signed_area_double: f32 = t21x * t31y - t21y * t31x;
        let s: Vec3<O> = (t31y * d1) - (t21y * d2); // eq 18
        let t: Vec3<O> = (-t31x * d1) + (t21x * d2); // eq 19

        triangle_info_list[f as usize].flags |= if signed_area_double > 0 as f32 {
            ORIENT_PRESERVING
        } else {
            0
        };

        if not_zero(signed_area_double) {
            let area_double: f32 = fabsf(signed_area_double);
            let s_magnitude: f32 = s.length();
            let t_magnitude: f32 = t.length();
            let sign: f32 = if triangle_info_list[f as usize].flags & ORIENT_PRESERVING == 0 {
                -1.0f32
            } else {
                1.0f32
            };
            if not_zero(s_magnitude) {
                triangle_info_list[f as usize].s = (sign / s_magnitude) * s;
            }
            if not_zero(t_magnitude) {
                triangle_info_list[f as usize].t = (sign / t_magnitude) * t;
            }

            // evaluate magnitudes prior to normalization of vOs and vOt
            triangle_info_list[f as usize].s_magnitude = s_magnitude / area_double;
            triangle_info_list[f as usize].t_magnitude = t_magnitude / area_double;

            // if this is a good triangle
            if not_zero(triangle_info_list[f as usize].s_magnitude)
                && not_zero(triangle_info_list[f as usize].t_magnitude)
            {
                triangle_info_list[f as usize].flags &= !GROUP_WITH_ANY;
            }
        }
        f += 1;
    }

    // force otherwise healthy quads to a fixed orientation
    while t < triangle_count - 1 {
        let original_face_index_a: c_int = triangle_info_list[t as usize].original_face_index;
        let original_face_index_b: c_int = triangle_info_list[(t + 1) as usize].original_face_index;
        if original_face_index_a == original_face_index_b {
            // this is a quad
            let is_degenerate_a: bool = triangle_info_list[t as usize].flags & MARK_DEGENERATE != 0;
            let is_degenerate_b: bool =
                triangle_info_list[(t + 1) as usize].flags & MARK_DEGENERATE != 0;

            // bad triangles should already have been removed by
            // DegenPrologue(), but just in case check bIsDeg_a and bIsDeg_a are false
            if !(is_degenerate_a || is_degenerate_b) {
                let orientation_preserving_a: bool =
                    triangle_info_list[t as usize].flags & ORIENT_PRESERVING != 0;
                let orientation_preserving_b: bool =
                    triangle_info_list[(t + 1) as usize].flags & ORIENT_PRESERVING != 0;

                // if this happens the quad has extremely bad mapping!!
                if orientation_preserving_a != orientation_preserving_b {
                    let mut choose_orientation_first_triangle: bool = false;
                    if triangle_info_list[(t + 1) as usize].flags & GROUP_WITH_ANY != 0
                        || calculate_texture_area(
                            context,
                            &triangle_vertex_list[{
                                let a = (t * 3 + 0) as usize;
                                let b = a + 3;
                                a..b
                            }],
                        ) >= calculate_texture_area(
                            context,
                            &triangle_vertex_list[{
                                let a = ((t + 1) * 3 + 0) as usize;
                                let b = a + 3;
                                a..b
                            }],
                        )
                    {
                        choose_orientation_first_triangle = true;
                    }

                    // force match
                    let t0: c_int = if choose_orientation_first_triangle {
                        t
                    } else {
                        t + 1
                    };
                    let t1_0: c_int = if choose_orientation_first_triangle {
                        t + 1
                    } else {
                        t
                    };

                    // clear first
                    triangle_info_list[t1_0 as usize].flags &= !ORIENT_PRESERVING;
                    // copy bit
                    triangle_info_list[t1_0 as usize].flags |=
                        triangle_info_list[t0 as usize].flags & ORIENT_PRESERVING;
                }
            }
            t += 2;
        } else {
            t += 1;
        }
    }

    // if /* can't allocate */ {
    //     BuildNeighborsSlow(triangle_info_list, triangle_vertex_list, iNrTrianglesIn);
    // }

    // match up edge pairs
    let mut edges: Vec<Edge> =
        vec![Edge::ZERO; (triangle_count as c_ulong).wrapping_mul(3) as usize];
    let vert_count = edges.len();
    let face_count = vert_count / 3;
    build_neighbors_fast(
        &mut triangle_info_list[..face_count],
        &mut edges,
        &triangle_vertex_list[..vert_count],
        triangle_count,
    );
}

fn build_4_rule_groups<O: Ops>(
    triangle_info_list: &mut [TriangleInfo<O>],
    groups: &mut [Group],
    triangle_vertex_list: &[c_int],
    triangle_count: c_int,
) -> c_int {
    let groups_max_count: c_int = triangle_count * 3;
    let mut groups_active_count: c_int = 0;

    let mut f = 0;
    while f < triangle_count {
        let mut i = 0;
        while i < 3 {
            // if not assigned to a group
            if triangle_info_list[f as usize].flags & GROUP_WITH_ANY == 0
                && triangle_info_list[f as usize].assigned_group[i as usize].is_none()
            {
                let vert_index: c_int = triangle_vertex_list[(f * 3 + i) as usize];
                assert!(groups_active_count < groups_max_count);
                triangle_info_list[f as usize].assigned_group[i as usize] =
                    Some(groups_active_count as usize);
                let this_group = &mut groups[groups_active_count as usize];
                this_group.id = groups_active_count as usize;
                this_group.vertex_representative = vert_index;
                this_group.orientation_preserving =
                    triangle_info_list[f as usize].flags & ORIENT_PRESERVING != 0;
                this_group.face_indices = Vec::new();
                groups_active_count += 1;

                add_triangle_to_group(this_group, f);
                let orientation_preserving_f =
                    triangle_info_list[f as usize].flags & ORIENT_PRESERVING != 0;
                let face_neighbor_index_left =
                    triangle_info_list[f as usize].face_neighbors[i as usize];
                let face_neighbor_index_right = triangle_info_list[f as usize].face_neighbors
                    [(if i > 0 { i - 1 } else { 2 }) as usize];

                if face_neighbor_index_left >= 0 {
                    // neighbor
                    let result: bool = assign_to_group_recursive(
                        triangle_vertex_list,
                        triangle_info_list,
                        face_neighbor_index_left,
                        this_group,
                    );
                    let orientation_preserving_left: bool =
                        triangle_info_list[face_neighbor_index_left as usize].flags
                            & ORIENT_PRESERVING
                            != 0;
                    let different: bool = orientation_preserving_f != orientation_preserving_left;
                    assert!(result || different);
                }
                if face_neighbor_index_right >= 0 {
                    // neighbor
                    let result: bool = assign_to_group_recursive(
                        triangle_vertex_list,
                        triangle_info_list,
                        face_neighbor_index_right,
                        this_group,
                    );
                    let orientation_preserving_right: bool =
                        triangle_info_list[face_neighbor_index_right as usize].flags
                            & ORIENT_PRESERVING
                            != 0;
                    let different: bool = orientation_preserving_f != orientation_preserving_right;
                    assert!(result || different);
                }
            }
            i += 1;
        }
        f += 1;
    }

    groups_active_count
}

fn add_triangle_to_group(group: &mut Group, triangle_index: c_int) {
    group.face_indices.push(triangle_index);
}

fn assign_to_group_recursive<O: Ops>(
    triangle_vertex_list: &[c_int],
    triangle_infos: &mut [TriangleInfo<O>],
    triangle_index: c_int,
    group: &mut Group,
) -> bool {
    let triangle_info = &mut triangle_infos[triangle_index as usize];

    // track down vertex
    let vertex_representative: c_int = group.vertex_representative;
    let vertices = &triangle_vertex_list[{
        let a = (3 * triangle_index + 0) as usize;
        let b = a + 3;
        a..b
    }];
    let mut i: c_int = -(1);
    if vertices[0] == vertex_representative {
        i = 0;
    } else if vertices[1] == vertex_representative {
        i = 1;
    } else if vertices[2] == vertex_representative {
        i = 2;
    }
    assert!(i >= 0 && i < 3);

    // early out
    if triangle_info.assigned_group[i as usize] == Some(group.id) {
        return true;
    } else if (triangle_info.assigned_group[i as usize]).is_some() {
        return false;
    }
    if triangle_info.flags & GROUP_WITH_ANY != 0
        && (triangle_info.assigned_group[0 as usize]).is_none()
        && (triangle_info.assigned_group[1 as usize]).is_none()
        && (triangle_info.assigned_group[2 as usize]).is_none()
    {
        // first to group with a group-with-anything triangle
        // determines it's orientation.
        // This is the only existing order dependency in the code!!
        triangle_info.flags &= !ORIENT_PRESERVING;
        triangle_info.flags |= if group.orientation_preserving {
            ORIENT_PRESERVING
        } else {
            0
        };
    }
    let orientation_preserving: bool = triangle_info.flags & ORIENT_PRESERVING != 0;
    if orientation_preserving != group.orientation_preserving {
        return false;
    }

    add_triangle_to_group(&mut *group, triangle_index);
    triangle_info.assigned_group[i as usize] = Some(group.id);

    let face_neighbor_index_left: c_int = triangle_info.face_neighbors[i as usize];
    let face_neighbor_index_right: c_int =
        triangle_info.face_neighbors[(if i > 0 { i - 1 } else { 2 }) as usize];
    if face_neighbor_index_left >= 0 {
        assign_to_group_recursive(
            triangle_vertex_list,
            triangle_infos,
            face_neighbor_index_left,
            group,
        );
    }
    if face_neighbor_index_right >= 0 {
        assign_to_group_recursive(
            triangle_vertex_list,
            triangle_infos,
            face_neighbor_index_right,
            group,
        );
    }

    true
}

fn generate_tangent_spaces<I: MikkTSpaceInterface<O>, O: Ops>(
    tangent_spaces: &mut [TangentSpace<O>],
    triangle_info_list: &[TriangleInfo<O>],
    groups: &[Group],
    groups_active_count: c_int,
    triangle_vertex_list: &[c_int],
    threshold_cos: f32,
    context: &I,
) -> bool {
    let mut faces_max_count = 0;
    let mut g = 0;
    while g < groups_active_count {
        if faces_max_count < groups[g as usize].face_indices.len() {
            faces_max_count = groups[g as usize].face_indices.len();
        }
        g += 1;
    }

    if faces_max_count == 0 {
        return true;
    }

    // make initial allocations
    let mut sub_group_tangent_spaces: Vec<TangentSpace<O>> =
        vec![TangentSpace::ZERO; faces_max_count as usize];
    let mut unified_sub_groups: Vec<Vec<c_int>> = vec![Vec::new(); faces_max_count as usize];
    let mut g = 0;
    while g < groups_active_count {
        let group = &groups[g as usize];
        let mut unified_sub_groups_count: c_int = 0;

        // triangles
        let mut i = 0;
        while i < group.face_indices.len() {
            // triangle number
            let f: c_int = (group.face_indices)[i as usize];
            let mut tmp_group = Vec::<c_int>::new();
            let index = if triangle_info_list[f as usize].assigned_group[0 as usize]
                == Some(g as usize)
            {
                0
            } else if triangle_info_list[f as usize].assigned_group[1 as usize] == Some(g as usize)
            {
                1
            } else if triangle_info_list[f as usize].assigned_group[2 as usize] == Some(g as usize)
            {
                2
            } else {
                panic!()
            };

            let vertex_index = triangle_vertex_list[(f * 3 + index) as usize];
            assert!(vertex_index == group.vertex_representative);

            // is normalized already
            let n = get_normal_from_index(context, vertex_index);

            // project
            let mut s_f =
                triangle_info_list[f as usize].s - ((n.dot(triangle_info_list[f as usize].s)) * n);
            let mut t_f =
                triangle_info_list[f as usize].t - ((n.dot(triangle_info_list[f as usize].t)) * n);
            s_f.normalize_or_zero();
            t_f.normalize_or_zero();

            // original face number
            let original_face_index_f = triangle_info_list[f as usize].original_face_index;

            let mut j = 0;
            while j < group.face_indices.len() {
                // triangle number
                let t: c_int = (group.face_indices)[j as usize];
                let original_face_index_t: c_int =
                    triangle_info_list[t as usize].original_face_index;

                // project
                let mut s_t: Vec3<O> = triangle_info_list[t as usize].s
                    - ((n.dot(triangle_info_list[t as usize].s)) * n);
                let mut t_t: Vec3<O> = triangle_info_list[t as usize].t
                    - ((n.dot(triangle_info_list[t as usize].t)) * n);
                s_t.normalize_or_zero();
                t_t.normalize_or_zero();

                let any: bool = (triangle_info_list[f as usize].flags
                    | triangle_info_list[t as usize].flags)
                    & GROUP_WITH_ANY
                    != 0;
                // make sure triangles which belong to the same quad are joined.
                let same_original_face: bool = original_face_index_f == original_face_index_t;

                let s_cos: f32 = s_f.dot(s_t);
                let t_cos: f32 = t_f.dot(t_t);

                assert!(f != t || same_original_face, "sanity check");
                if any || same_original_face || s_cos > threshold_cos && t_cos > threshold_cos {
                    tmp_group.push(t);
                }

                j += 1;
            }

            // sort pTmpMembers
            tmp_group.sort();

            // look for an existing match
            let mut found = false;
            let mut l = 0;
            while l < unified_sub_groups_count && !found {
                found = tmp_group == unified_sub_groups[l as usize];
                if !found {
                    l += 1;
                }
            }

            // assign tangent space index
            assert!(found || l == unified_sub_groups_count);

            // if no match was found we allocate a new subgroup
            if !found {
                // insert new subgroup
                sub_group_tangent_spaces[unified_sub_groups_count as usize] =
                    evaluate_tangent_space(
                        &tmp_group,
                        triangle_vertex_list,
                        triangle_info_list,
                        context,
                        group.vertex_representative,
                    );
                unified_sub_groups[unified_sub_groups_count as usize] = tmp_group;
                unified_sub_groups_count += 1;
            }

            // output tspace
            let tangent_space_offset =
                triangle_info_list[f as usize].tangent_spaces_offset as usize;
            let vertex = triangle_info_list[f as usize].vertex_indices[index as usize] as usize;
            let tangent_space = &mut tangent_spaces[(tangent_space_offset + vertex) as usize];
            assert!(tangent_space.counter < 2);
            assert!(
                (triangle_info_list[f as usize].flags & 8 != 0) == group.orientation_preserving
            );
            if tangent_space.counter == 1 {
                *tangent_space =
                    mean_tangent_space(*tangent_space, sub_group_tangent_spaces[l as usize]);
                // update counter
                tangent_space.counter = 2;
                tangent_space.orientation_preserving = group.orientation_preserving;
            } else {
                assert!(tangent_space.counter == 0);
                *tangent_space = sub_group_tangent_spaces[l as usize];
                // update counter
                tangent_space.counter = 1;
                tangent_space.orientation_preserving = group.orientation_preserving;
            }

            i += 1;
        }

        g += 1;
    }

    true
}

fn evaluate_tangent_space<I: MikkTSpaceInterface<O>, O: Ops>(
    face_indices: &[c_int],
    triangle_vertex_list: &[c_int],
    triangle_info_list: &[TriangleInfo<O>],
    context: &I,
    vertex_representative: c_int,
) -> TangentSpace<O> {
    let face_indices_count = face_indices.len();
    let mut res: TangentSpace<O> = TangentSpace {
        s: Vec3::ZERO,
        s_magnitude: 0.,
        t: Vec3::ZERO,
        t_magnitude: 0.,
        counter: 0,
        orientation_preserving: false,
    };
    let mut angle_sum: f32 = 0 as f32;
    res.s.x = 0.0f32;
    res.s.y = 0.0f32;
    res.s.z = 0.0f32;
    res.t.x = 0.0f32;
    res.t.y = 0.0f32;
    res.t.z = 0.0f32;
    res.s_magnitude = 0 as f32;
    res.t_magnitude = 0 as f32;

    let mut face = 0;
    while face < face_indices_count {
        let f: c_int = face_indices[face as usize];

        // only valid triangles get to add their contribution
        if triangle_info_list[f as usize].flags & GROUP_WITH_ANY == 0 {
            let i = if triangle_vertex_list[(3 * f + 0) as usize] == vertex_representative {
                0
            } else if triangle_vertex_list[(3 * f + 1) as usize] == vertex_representative {
                1
            } else if triangle_vertex_list[(3 * f + 2) as usize] == vertex_representative {
                2
            } else {
                panic!()
            };

            // project
            let index = triangle_vertex_list[(3 * f + i) as usize];
            let n = get_normal_from_index(context, index);
            let mut s =
                triangle_info_list[f as usize].s - ((n.dot(triangle_info_list[f as usize].s)) * n);
            let mut t =
                triangle_info_list[f as usize].t - (n.dot(triangle_info_list[f as usize].t) * n);
            s.normalize_or_zero();
            t.normalize_or_zero();

            let i2 = triangle_vertex_list[(3 * f + (if i < 2 { i + 1 } else { 0 })) as usize];
            let i1 = triangle_vertex_list[(3 * f + i) as usize];
            let i0 = triangle_vertex_list[(3 * f + (if i > 0 { i - 1 } else { 2 })) as usize];

            let p0 = get_position_from_index(context, i0);
            let p1 = get_position_from_index(context, i1);
            let p2 = get_position_from_index(context, i2);
            let mut v1 = p0 - p1;
            let mut v2 = p2 - p1;

            // project
            v1 = v1 - ((n.dot(v1)) * n);
            v1.normalize_or_zero();
            v2 = v2 - ((n.dot(v2)) * n);
            v2.normalize_or_zero();

            // weight contribution by the angle
            // between the two edge vectors
            let mut cos = v1.dot(v2);
            cos = if cos > 1 as f32 {
                1 as f32
            } else if cos < -(1) as f32 {
                -(1) as f32
            } else {
                cos
            };
            let angle = O::acos(cos as f64) as f32;
            let s_magnitude = triangle_info_list[f as usize].s_magnitude;
            let t_magnitude = triangle_info_list[f as usize].t_magnitude;

            res.s = res.s + (angle * s);
            res.t = res.t + (angle * t);
            res.s_magnitude += angle * s_magnitude;
            res.t_magnitude += angle * t_magnitude;
            angle_sum += angle;
        }
        face += 1;
    }

    // normalize
    res.s.normalize_or_zero();
    res.t.normalize_or_zero();
    if angle_sum > 0 as f32 {
        res.s_magnitude /= angle_sum;
        res.t_magnitude /= angle_sum;
    }

    res
}

fn build_neighbors_fast<O: Ops>(
    triangle_info_list: &mut [TriangleInfo<O>],
    edges: &mut [Edge],
    triangle_vertex_list: &[c_int],
    triangle_count: c_int,
) {
    // build array of edges
    let seed: c_uint = INTERNAL_RND_SORT_SEED as c_uint;
    let mut f = 0;
    while f < triangle_count {
        let mut i = 0;
        while i < 3 {
            let i0: c_int = triangle_vertex_list[(f * 3 + i) as usize];
            let i1: c_int =
                triangle_vertex_list[(f * 3 + (if i < 2 { i + 1 } else { 0 })) as usize];

            // put minimum index in i0
            edges[(f * 3 + i) as usize].i0 = i0.min(i1);
            // put maximum index in i1
            edges[(f * 3 + i) as usize].i1 = i0.max(i1);
            // record face number
            edges[(f * 3 + i) as usize].f = f;

            i += 1;
        }

        f += 1;
    }

    // sort over all edges by i0, this is the pricy one.
    quick_sort_edges(edges, 0, triangle_count * 3 - 1, 0, seed);
    let entries = triangle_count * 3;
    let mut current_start_index = 0;
    let mut i = 1;
    while i < entries {
        if edges[current_start_index as usize].i0 != edges[i as usize].i0 {
            let index_left: c_int = current_start_index;
            let index_right: c_int = i - 1;
            current_start_index = i;
            quick_sort_edges(edges, index_left, index_right, 1, seed);
        }
        i += 1;
    }
    current_start_index = 0;
    let mut i = 1;
    while i < entries {
        if edges[current_start_index as usize].i0 != edges[i as usize].i0
            || edges[current_start_index as usize].i1 != edges[i as usize].i1
        {
            let index_left: c_int = current_start_index;
            let index_right: c_int = i - 1;
            current_start_index = i;
            quick_sort_edges(edges, index_left, index_right, 2, seed);
        }
        i += 1;
    }

    // pair up, adjacent triangles
    let mut i = 0;
    while i < entries {
        let i0_0: c_int = edges[i as usize].i0;
        let i1_0: c_int = edges[i as usize].i1;
        let f_0: c_int = edges[i as usize].f;

        let mut edgenum_b: c_int = 0;

        // resolve index ordering and edge_num
        let (edgenum_a, i0_a, i1_a) = get_edge(
            &triangle_vertex_list[{
                let a = (f_0 * 3) as usize;
                let b = a + 3;
                a..b
            }],
            i0_0,
            i1_0,
        )
        .unwrap();
        let unassigned_a =
            triangle_info_list[f_0 as usize].face_neighbors[edgenum_a as usize] == -(1);

        if unassigned_a {
            // get true index ordering
            let mut j: c_int = i + 1;
            let mut not_found: bool = true;
            while j < entries
                && i0_0 == edges[j as usize].i0
                && i1_0 == edges[j as usize].i1
                && not_found
            {
                let t = edges[j as usize].f;
                // flip i0_B and i1_B
                // resolve index ordering and edge_num
                let (edgenum, i1_b, i0_b) = get_edge(
                    &triangle_vertex_list[{
                        let a = (t * 3) as usize;
                        let b = a + 3;
                        a..b
                    }],
                    edges[j as usize].i0,
                    edges[j as usize].i1,
                )
                .unwrap();
                edgenum_b = edgenum;
                let unassigned_b =
                    triangle_info_list[t as usize].face_neighbors[edgenum_b as usize] == -(1);

                if i0_a == i0_b && i1_a == i1_b && unassigned_b {
                    not_found = false;
                } else {
                    j += 1;
                }
            }

            if !not_found {
                let t_0: c_int = edges[j as usize].f;
                triangle_info_list[f_0 as usize].face_neighbors[edgenum_a as usize] = t_0;
                triangle_info_list[t_0 as usize].face_neighbors[edgenum_b as usize] = f_0;
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
fn quick_sort_edges(
    sort_buffer: &mut [Edge],
    index_left_in: c_int,
    index_right_in: c_int,
    channel: c_int,
    mut seed: c_uint,
) {
    let elements: c_int = index_right_in - index_left_in + 1;
    if elements < 2 {
        return;
    } else if elements == 2 {
        if sort_buffer[index_left_in as usize][channel as usize]
            > sort_buffer[index_right_in as usize][channel as usize]
        {
            sort_buffer.swap(index_left_in as usize, index_right_in as usize);
        }
        return;
    }
    let mut t = seed & 31 as c_uint;
    t = seed.wrapping_shl(t) | seed.wrapping_shr((32 as c_uint).wrapping_sub(t));
    seed = seed.wrapping_add(t).wrapping_add(3 as c_uint);
    let mut index_left = index_left_in;
    let mut index_right = index_right_in;
    let n = index_right - index_left + 1;
    assert!(n >= 0);
    let index = seed.wrapping_rem(n as c_uint);
    let index_mid = sort_buffer[(index + (index_left as c_uint)) as usize][channel as usize];
    loop {
        while sort_buffer[index_left as usize][channel as usize] < index_mid {
            index_left += 1;
        }
        while sort_buffer[index_right as usize][channel as usize] > index_mid {
            index_right -= 1;
        }
        if index_left <= index_right {
            sort_buffer.swap(index_left as usize, index_right as usize);
            index_left += 1;
            index_right -= 1;
        }
        if index_left > index_right {
            break;
        }
    }
    if index_left_in < index_right {
        quick_sort_edges(sort_buffer, index_left_in, index_right, channel, seed);
    }
    if index_left < index_right_in {
        quick_sort_edges(sort_buffer, index_left, index_right_in, channel, seed);
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

fn degen_prologue<O: Ops>(
    triangle_info_list: &mut [TriangleInfo<O>],
    triangle_vertices: &mut [c_int],
    triangle_count: c_int,
    triangle_count_total: c_int,
) {
    // locate quads with only one good triangle
    let mut t: c_int = 0;
    while t < triangle_count_total - 1 {
        let original_face_index_a: c_int = triangle_info_list[t as usize].original_face_index;
        let original_face_index_b: c_int = triangle_info_list[(t + 1) as usize].original_face_index;
        if original_face_index_a == original_face_index_b {
            // this is a quad
            let is_degenerate_a: bool = triangle_info_list[t as usize].flags & MARK_DEGENERATE != 0;
            let is_degenerate_b: bool =
                triangle_info_list[(t + 1) as usize].flags & MARK_DEGENERATE != 0;
            if is_degenerate_a ^ is_degenerate_b {
                triangle_info_list[t as usize].flags |= QUAD_ONE_DEGEN_TRI;
                triangle_info_list[(t + 1) as usize].flags |= QUAD_ONE_DEGEN_TRI;
            }
            t += 2;
        } else {
            t += 1;
        }
    }

    // reorder list so all degen triangles are moved to the back
    // without reordering the good triangles
    let mut next_good_triangle_search_index = 1;
    let mut t = 0;
    let mut still_finding_good_ones = true;
    while t < triangle_count && still_finding_good_ones {
        let is_good: bool = triangle_info_list[t as usize].flags & MARK_DEGENERATE == 0;
        if is_good {
            if next_good_triangle_search_index < t + 2 {
                next_good_triangle_search_index = t + 2;
            }
        } else {
            // search for the first good triangle.
            let mut just_a_single_degenerate: bool = true;
            while just_a_single_degenerate && next_good_triangle_search_index < triangle_count_total
            {
                let is_good: bool = triangle_info_list[next_good_triangle_search_index as usize]
                    .flags
                    & MARK_DEGENERATE
                    == 0;
                if is_good {
                    just_a_single_degenerate = false;
                } else {
                    next_good_triangle_search_index += 1;
                }
            }

            let t0 = t;
            let t1 = next_good_triangle_search_index;
            next_good_triangle_search_index += 1;
            assert!(next_good_triangle_search_index > t + 1);

            // swap triangle t0 and t1
            if !just_a_single_degenerate {
                let mut i = 0;
                while i < 3 {
                    triangle_vertices.swap((t0 * 3 + i) as usize, (t1 * 3 + i) as usize);
                    i += 1;
                }
                triangle_info_list.swap(t0 as usize, t1 as usize);
            } else {
                // this is not supposed to happen
                still_finding_good_ones = false;
            }
        }
        if still_finding_good_ones {
            t += 1;
        }
    }

    assert!(still_finding_good_ones, "code will still work");
    assert!(triangle_count == t);
}

fn degen_epilogue<I: MikkTSpaceInterface<O>, O: Ops>(
    tangent_spaces: &mut [TangentSpace<O>],
    triangle_info_list: &[TriangleInfo<O>],
    triangle_vertex_list: &[c_int],
    context: &I,
    triangle_count: c_int,
    triangle_total_count: c_int,
) {
    // deal with degenerate triangles
    // punishment for degenerate triangles is O(N^2)
    let mut t = triangle_count;
    while t < triangle_total_count {
        // degenerate triangles on a quad with one good triangle are skipped
        // here but processed in the next loop
        let skip: bool = triangle_info_list[t as usize].flags & QUAD_ONE_DEGEN_TRI != 0;

        if !skip {
            let mut i = 0;
            while i < 3 {
                let index1: c_int = triangle_vertex_list[(t * 3 + i) as usize];
                // search through the good triangles
                let mut not_found: bool = true;
                let mut j: c_int = 0;
                while not_found && j < 3 * triangle_count {
                    let index2: c_int = triangle_vertex_list[j as usize];
                    if index1 == index2 {
                        not_found = false;
                    } else {
                        j += 1;
                    }
                }

                if !not_found {
                    let face: c_int = j / 3;
                    let vertex: c_int = j % 3;
                    let source_vertex =
                        triangle_info_list[face as usize].vertex_indices[vertex as usize] as usize;
                    let source_tangent_space_offset =
                        triangle_info_list[face as usize].tangent_spaces_offset as usize;
                    let destination_vertex =
                        triangle_info_list[t as usize].vertex_indices[i as usize] as usize;
                    let destination_tangent_space_offset =
                        triangle_info_list[t as usize].tangent_spaces_offset as usize;

                    // copy tspace
                    tangent_spaces
                        [(destination_tangent_space_offset + destination_vertex) as usize] =
                        tangent_spaces[(source_tangent_space_offset + source_vertex) as usize];
                }

                i += 1;
            }
        }

        t += 1;
    }

    // deal with degenerate quads with one good triangle
    t = 0;
    while t < triangle_count {
        // this triangle belongs to a quad where the
        // other triangle is degenerate
        if triangle_info_list[t as usize].flags & QUAD_ONE_DEGEN_TRI != 0 {
            let vertices: [u8; 4] = triangle_info_list[t as usize].vertex_indices;
            let flag: c_int = (1) << vertices[0] | (1) << vertices[1] | (1) << vertices[2];
            let mut missing_index: c_int = 0;
            if flag & 2 == 0 {
                missing_index = 1;
            } else if flag & 4 == 0 {
                missing_index = 2;
            } else if flag & 8 == 0 {
                missing_index = 3;
            }

            let original_face_index = triangle_info_list[t as usize].original_face_index;
            let missing_position =
                get_position_from_index(context, as_index(original_face_index, missing_index));
            let mut not_found = true;
            let mut i_0 = 0;
            while not_found && i_0 < 3 {
                let vertex = vertices[i_0 as usize] as c_int;
                let source_position: Vec3<O> =
                    get_position_from_index(context, as_index(original_face_index, vertex));
                if source_position == missing_position {
                    let tangent_space_offset: c_int =
                        triangle_info_list[t as usize].tangent_spaces_offset;
                    tangent_spaces[(tangent_space_offset + missing_index) as usize] =
                        tangent_spaces[(tangent_space_offset + vertex) as usize];
                    not_found = false;
                } else {
                    i_0 += 1;
                }
            }
            assert!(!not_found);
        }

        t += 1;
    }
}
