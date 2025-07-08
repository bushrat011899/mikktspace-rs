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

use crate::{math::*, MikkTSpaceInterface};

pub(crate) fn generate_tangent_space<I: MikkTSpaceInterface<O>, O: Ops>(
    context: &mut I,
    angular_threshold: f32,
) -> Result<(), GenerateTangentSpaceError> {
    let threshold_cos = O::cos(deg_to_rad(angular_threshold) as f64) as f32;
    let faces_total = context.get_num_faces();

    // count triangles on supported faces
    let triangles_total_count = (0..faces_total)
        .map(|f| match context.get_num_vertices_of_face(f) {
            3 => 1,
            4 => 2,
            _ => 0,
        })
        .sum::<usize>();

    if triangles_total_count == 0 {
        return Err(GenerateTangentSpaceError::InsufficientTriangles);
    }

    // make an initial triangle --> face index list
    let (mut triangle_info_list, mut triangle_vertex_list, tangent_spaces_total) =
        generate_initial_vertices_index_list(context, triangles_total_count);

    // make a welded index list of identical positions and attributes (pos, norm, texc)
    generate_shared_vertices_index_list(context, &mut triangle_vertex_list);

    // mark all triangle pairs that belong to a quad with only one
    // good triangle. These need special treatment in DegenEpilogue().
    // Additionally, move all good triangles to the start of
    // triangle_info_list[] and triangle_vertex_list[] without changing order and
    // put the degenerate triangles last.
    let triangles_degenerate_count =
        degen_prologue(context, &mut triangle_info_list, &mut triangle_vertex_list);

    let triangles_count = triangles_total_count - triangles_degenerate_count;

    // evaluate triangle level attributes and neighbor list
    initialize_triangle_info(
        &mut triangle_info_list[..triangles_count],
        &triangle_vertex_list,
        context,
    );

    // based on the 4 rules, identify groups based on connectivity
    let groups = build_4_rule_groups(
        &mut triangle_info_list[..triangles_count],
        &triangle_vertex_list,
    );

    // make tspaces, each group is split up into subgroups if necessary
    // based on fAngularThreshold. Finally a tangent space is made for
    // every resulting subgroup
    let mut tangent_spaces = generate_tangent_spaces(
        tangent_spaces_total,
        &triangle_info_list,
        &groups,
        &triangle_vertex_list,
        threshold_cos,
        context,
    );

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
    );

    let mut tangent_spaces_iter = tangent_spaces.iter();
    for f in 0..faces_total {
        let vertices = context.get_num_vertices_of_face(f);
        if vertices == 3 || vertices == 4 {
            // I've decided to let degenerate triangles and group-with-anythings
            // vary between left/right hand coordinate systems at the vertices.
            // All healthy triangles on the other hand are built to always be either or.
            // set data
            for v in 0..vertices {
                context.set_tangent_space(tangent_spaces_iter.next().unwrap().into(), f, v);
            }
        }
    }

    Ok(())
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum GenerateTangentSpaceError {
    InsufficientTriangles,
}

impl core::fmt::Display for GenerateTangentSpaceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GenerateTangentSpaceError::InsufficientTriangles => {
                f.write_str("Insufficient Triangles")
            }
        }
    }
}

impl core::error::Error for GenerateTangentSpaceError {}

struct RawTangentSpace<O: Ops> {
    /// normalized first order face derivative
    s: Vec3<O>,
    /// normalized first order face derivative
    t: Vec3<O>,
    /// original magnitude of vOs
    s_magnitude: f32,
    /// original magnitude of vOs
    t_magnitude: f32,
}

impl<O: Ops> Copy for RawTangentSpace<O> {}

impl<O: Ops> Clone for RawTangentSpace<O> {
    fn clone(&self) -> Self {
        *self
    }
}

struct TangentSpace<O: Ops> {
    inner: RawTangentSpace<O>,
    /// this is to average back into quads.
    counter: u8,
    orientation_preserving: bool,
}

impl<O: Ops> Copy for TangentSpace<O> {}

impl<O: Ops> Clone for TangentSpace<O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, O: Ops> From<&'a TangentSpace<O>> for crate::TangentSpace {
    fn from(value: &'a TangentSpace<O>) -> Self {
        crate::TangentSpace {
            tangent: [value.inner.s.x, value.inner.s.y, value.inner.s.z],
            bi_tangent: [value.inner.t.x, value.inner.t.y, value.inner.t.z],
            mag_s: value.inner.s_magnitude,
            mag_t: value.inner.t_magnitude,
            is_orientation_preserving: value.orientation_preserving,
        }
    }
}

impl<O: Ops> From<TangentSpace<O>> for crate::TangentSpace {
    fn from(value: TangentSpace<O>) -> Self {
        crate::TangentSpace::from(&value)
    }
}

struct TriangleInfo<O: Ops> {
    face_neighbors: [Option<usize>; 3],
    assigned_group: [Option<usize>; 3],

    tangent_space: RawTangentSpace<O>,

    /// determines if the current and the next triangle are a quad.
    original_face_index: usize,

    flags: usize,
    tangent_spaces_offset: usize,
    vertex_indices: [u8; 4],
}

impl<O: Ops> Copy for TriangleInfo<O> {}

impl<O: Ops> Clone for TriangleInfo<O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<O: Ops> TriangleInfo<O> {
    const ZERO: TriangleInfo<O> = TriangleInfo {
        face_neighbors: [None; 3],
        assigned_group: [None; 3],
        tangent_space: RawTangentSpace {
            s: Vec3::ZERO,
            t: Vec3::ZERO,
            s_magnitude: 0.,
            t_magnitude: 0.,
        },
        original_face_index: 0,
        flags: 0,
        tangent_spaces_offset: 0,
        vertex_indices: [0; 4],
    };
}

#[derive(Clone)]
struct Group {
    id: usize,
    face_indices: Vec<usize>,
    vertex_representative: usize,
    orientation_preserving: bool,
}

impl Group {
    const ZERO: Group = Group {
        id: 0,
        face_indices: Vec::new(),
        vertex_representative: 0,
        orientation_preserving: false,
    };
}

struct TemporaryVertex<O: Ops> {
    vert: Vec3<O>,
    index: usize,
}

impl<O: Ops> Copy for TemporaryVertex<O> {}

impl<O: Ops> Clone for TemporaryVertex<O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<O: Ops> TemporaryVertex<O> {
    const ZERO: TemporaryVertex<O> = TemporaryVertex {
        vert: Vec3::ZERO,
        index: 0,
    };
}

const INTERNAL_RND_SORT_SEED: u32 = 39871946;
const MARK_DEGENERATE: usize = 1;
const QUAD_ONE_DEGEN_TRI: usize = 2;
const GROUP_WITH_ANY: usize = 4;
const ORIENT_PRESERVING: usize = 8;

fn as_index(face: usize, vertex: usize) -> usize {
    assert!((0..4).contains(&vertex));
    face << 2 | vertex & 0x3
}

fn from_index(index: usize) -> (usize, usize) {
    (index >> 2, index & 0x3)
}

fn mean_tangent_space<O: Ops>(
    lhs: RawTangentSpace<O>,
    rhs: RawTangentSpace<O>,
) -> RawTangentSpace<O> {
    let mut ts_res = RawTangentSpace {
        s: Vec3::ZERO,
        s_magnitude: 0.,
        t: Vec3::ZERO,
        t_magnitude: 0.,
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

const CELLS: usize = 2048;

// it is IMPORTANT that this function is called to evaluate the hash since
// inlining could potentially reorder instructions and generate different
// results for the same effective input value fVal.
#[inline(never)]
fn find_grid_cell(min: f32, max: f32, val: f32) -> usize {
    let face = CELLS as f32 * ((val - min) / (max - min));
    let vertex = face as usize;
    if vertex < CELLS {
        vertex
    } else {
        CELLS - 1
    }
}

fn generate_shared_vertices_index_list<I: MikkTSpaceInterface<O>, O: Ops>(
    context: &I,
    triangle_vertices: &mut [usize],
) {
    // Generate bounding box
    let mut min = get_position_from_index(context, 0);
    let mut max = min;
    for index in triangle_vertices.iter().skip(1) {
        let position = get_position_from_index(context, *index);
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
    let mut hash_table = vec![0usize; triangle_vertices.len()];
    let mut hash_count = vec![0usize; CELLS];
    let mut hash_offsets = vec![0usize; CELLS];
    let mut hash_count_2 = vec![0usize; CELLS];

    // count amount of elements in each cell unit
    for index_0 in triangle_vertices.iter() {
        let position = get_position_from_index(context, *index_0);
        let val = if channel == 0 {
            position.x
        } else if channel == 1 {
            position.y
        } else {
            position.z
        };
        let cell = find_grid_cell(min_channel, max_channel, val);
        let fresh0 = &mut hash_count[cell];
        *fresh0 += 1;
    }

    // evaluate start index of each cell.
    hash_offsets[0_usize] = 0;
    for k in 1..CELLS {
        hash_offsets[k] = hash_offsets[k - 1] + hash_count[k - 1];
    }

    // insert vertices
    for (i, index_1) in triangle_vertices.iter().enumerate() {
        let position = get_position_from_index(context, *index_1);
        let val = if channel == 0 {
            position.x
        } else if channel == 1 {
            position.y
        } else {
            position.z
        };
        let cell = find_grid_cell(min_channel, max_channel, val);
        assert!(hash_count_2[cell] < hash_count[cell]);
        let entry = &mut hash_table[hash_offsets[cell] + hash_count_2[cell]];
        *entry = i; // vertex i has been inserted.
        let fresh1 = &mut hash_count_2[cell];
        *fresh1 += 1;
    }

    // verify the count
    for k in 0..CELLS {
        assert!(hash_count_2[k] == hash_count[k]);
    }

    // find maximum amount of entries in any hash entry
    let max_count = *hash_count.iter().max().unwrap();

    // complete the merge
    let mut temporary_vertices = vec![TemporaryVertex::<O>::ZERO; max_count];
    for k in 0..CELLS {
        let entries = hash_count[k];
        if entries >= 2 {
            // if /* couldn't allocate pTmpVert? */ {
            //     MergeVertsSlow(
            //         piTriList_in_and_out,
            //         context,
            //         pTable_0 as *const usize,
            //         iEntries,
            //     );
            // }
            for e in 0..entries {
                let i_0 = hash_table[hash_offsets[k] + e];
                let position = get_position_from_index(context, triangle_vertices[i_0]);
                temporary_vertices[e].vert = position;
                temporary_vertices[e].index = i_0;
            }
            merge_verts_fast(
                triangle_vertices,
                &mut temporary_vertices,
                context,
                0,
                entries - 1,
            );
        }
    }
}

fn merge_verts_fast<I: MikkTSpaceInterface<O>, O: Ops>(
    triangle_verticies: &mut [usize],
    temporary_verticies: &mut [TemporaryVertex<O>],
    context: &I,
    i_left_in: usize,
    i_right_in: usize,
) {
    // make bbox
    let (min, max) = temporary_verticies
        .iter()
        .take(i_right_in + 1)
        .skip(i_left_in)
        .fold(None, |state, t| {
            let (mut min, mut max) = state.unwrap_or_else(|| {
                let v = [t.vert.x, t.vert.y, t.vert.z];
                (v, v)
            });

            for c in 0..3 {
                min[c] = min[c].min(t.vert[c]);
                max[c] = max[c].max(t.vert[c]);
            }

            Some((min, max))
        })
        .unwrap();

    let dx = max[0] - min[0];
    let dy = max[1] - min[1];
    let dz = max[2] - min[2];

    let mut channel = 0;
    if dy > dx && dy > dz {
        channel = 1;
    } else if dz > dx {
        channel = 2;
    }

    let sep = 0.5f32 * (max[channel] + min[channel]);

    // stop if all vertices are NaNs
    if !sep.is_finite() {
        return;
    }

    // terminate recursion when the separation/average value
    // is no longer strictly between fMin and fMax values.
    if sep >= max[channel] || sep <= min[channel] {
        // complete the weld
        for l in i_left_in..=i_right_in {
            let i = temporary_verticies[l].index;
            let index = triangle_verticies[i];

            let a = (
                get_position_from_index(context, index),
                get_normal_from_index(context, index),
                get_texture_coordinate_from_index(context, index),
            );

            let i2 = temporary_verticies
                .iter()
                .take(l)
                .skip(i_left_in)
                .find_map(|t| {
                    let i2 = t.index;
                    let index = triangle_verticies[i2];

                    let b = (
                        get_position_from_index(context, index),
                        get_normal_from_index(context, index),
                        get_texture_coordinate_from_index(context, index),
                    );

                    (a == b).then_some(i2)
                });

            // merge if previously found
            if let Some(i2) = i2 {
                triangle_verticies[i] = triangle_verticies[i2];
            }
        }
    } else {
        let mut i_left = i_left_in;
        let mut i_right = i_right_in;
        assert!(i_right_in - i_left_in > 0, "at least 2 entries");

        // separate (by fSep) all points between iL_in and iR_in in pTmpVert[]
        while i_left < i_right {
            let mut ready_left_swap = false;
            let mut ready_right_swap = false;
            while !ready_left_swap && i_left < i_right {
                assert!(i_left >= i_left_in && i_left <= i_right_in);
                ready_left_swap = temporary_verticies[i_left].vert[channel] >= sep;
                if !ready_left_swap {
                    i_left += 1;
                }
            }
            while !ready_right_swap && i_left < i_right {
                assert!(i_right >= i_left_in && i_right <= i_right_in);
                ready_right_swap = temporary_verticies[i_right].vert[channel] < sep;
                if !ready_right_swap {
                    i_right -= 1;
                }
            }
            assert!(i_left < i_right || !(ready_left_swap && ready_right_swap));

            if ready_left_swap && ready_right_swap {
                let temporary_vertex = temporary_verticies[i_left];
                assert!(i_left < i_right);
                temporary_verticies[i_left] = temporary_verticies[i_right];
                temporary_verticies[i_right] = temporary_vertex;
                i_left += 1;
                i_right -= 1;
            }
        }

        assert!(i_left == i_right + 1 || i_left == i_right);
        if i_left == i_right {
            let ready_right_swap = temporary_verticies[i_right].vert[channel] < sep;
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
    context: &I,
    triangles_total_count: usize,
) -> (Vec<TriangleInfo<O>>, Vec<usize>, usize) {
    let mut triangle_info_list = vec![TriangleInfo::ZERO; triangles_total_count];
    let mut triangle_verticies = vec![0; triangles_total_count * 3];

    let triangle_count = triangles_total_count;
    let mut tangent_space_offset = 0;
    let mut destination_triangle_info_index = 0;
    for f in 0..context.get_num_faces() {
        let verts = context.get_num_vertices_of_face(f);
        if !(verts != 3 && verts != 4) {
            triangle_info_list[destination_triangle_info_index].original_face_index = f;
            triangle_info_list[destination_triangle_info_index].tangent_spaces_offset =
                tangent_space_offset;
            if verts == 3 {
                let verticies =
                    &mut triangle_info_list[destination_triangle_info_index].vertex_indices;
                verticies[0] = 0_u8;
                verticies[1] = 1_u8;
                verticies[2] = 2_u8;
                triangle_verticies[destination_triangle_info_index * 3] = as_index(f, 0);
                triangle_verticies[destination_triangle_info_index * 3 + 1] = as_index(f, 1);
                triangle_verticies[destination_triangle_info_index * 3 + 2] = as_index(f, 2);
                destination_triangle_info_index += 1;
            } else {
                triangle_info_list[destination_triangle_info_index + 1].original_face_index = f;
                triangle_info_list[destination_triangle_info_index + 1].tangent_spaces_offset =
                    tangent_space_offset;

                // need an order independent way to evaluate
                // tspace on quads. This is done by splitting
                // along the shortest diagonal.
                let i0 = as_index(f, 0);
                let i1 = as_index(f, 1);
                let i2 = as_index(f, 2);
                let i3 = as_index(f, 3);
                let tx0 = get_texture_coordinate_from_index(context, i0);
                let tx1 = get_texture_coordinate_from_index(context, i1);
                let tx2 = get_texture_coordinate_from_index(context, i2);
                let tx3 = get_texture_coordinate_from_index(context, i3);
                let distance_squared_20 = (tx2 - tx0).length_squared();
                let distance_squared_13 = (tx3 - tx1).length_squared();
                let quad_diagonal_is_02 = if distance_squared_20 < distance_squared_13 {
                    true
                } else if distance_squared_13 < distance_squared_20 {
                    false
                } else {
                    let p0 = get_position_from_index(context, i0);
                    let p1 = get_position_from_index(context, i1);
                    let p2 = get_position_from_index(context, i2);
                    let p3 = get_position_from_index(context, i3);
                    let distance_squared_20 = (p2 - p0).length_squared();
                    let distance_squared_13 = (p3 - p1).length_squared();
                    distance_squared_13 >= distance_squared_20
                };
                if quad_diagonal_is_02 {
                    let verticies_a =
                        &mut triangle_info_list[destination_triangle_info_index].vertex_indices;
                    verticies_a[0] = 0_u8;
                    verticies_a[1] = 1_u8;
                    verticies_a[2] = 2_u8;
                    triangle_verticies[destination_triangle_info_index * 3] = i0;
                    triangle_verticies[destination_triangle_info_index * 3 + 1] = i1;
                    triangle_verticies[destination_triangle_info_index * 3 + 2] = i2;
                    destination_triangle_info_index += 1;
                    let verticies_b =
                        &mut triangle_info_list[destination_triangle_info_index].vertex_indices;
                    verticies_b[0] = 0_u8;
                    verticies_b[1] = 2_u8;
                    verticies_b[2] = 3_u8;
                    triangle_verticies[destination_triangle_info_index * 3] = i0;
                    triangle_verticies[destination_triangle_info_index * 3 + 1] = i2;
                    triangle_verticies[destination_triangle_info_index * 3 + 2] = i3;
                    destination_triangle_info_index += 1;
                } else {
                    let verticies_a =
                        &mut triangle_info_list[destination_triangle_info_index].vertex_indices;
                    verticies_a[0] = 0_u8;
                    verticies_a[1] = 1_u8;
                    verticies_a[2] = 3_u8;
                    triangle_verticies[destination_triangle_info_index * 3] = i0;
                    triangle_verticies[destination_triangle_info_index * 3 + 1] = i1;
                    triangle_verticies[destination_triangle_info_index * 3 + 2] = i3;
                    destination_triangle_info_index += 1;
                    let verticies_b =
                        &mut triangle_info_list[destination_triangle_info_index].vertex_indices;
                    verticies_b[0] = 1_u8;
                    verticies_b[1] = 2_u8;
                    verticies_b[2] = 3_u8;
                    triangle_verticies[destination_triangle_info_index * 3] = i1;
                    triangle_verticies[destination_triangle_info_index * 3 + 1] = i2;
                    triangle_verticies[destination_triangle_info_index * 3 + 2] = i3;
                    destination_triangle_info_index += 1;
                }
            }
            tangent_space_offset += verts;
            assert!(destination_triangle_info_index <= triangle_count);
        }
    }

    for info in triangle_info_list.iter_mut().take(triangle_count) {
        info.flags = 0;
    }

    // return total amount of tspaces
    (triangle_info_list, triangle_verticies, tangent_space_offset)
}

fn get_position_from_index<I: MikkTSpaceInterface<O>, O: Ops>(
    context: &I,
    index: usize,
) -> Vec3<O> {
    let mut res = Vec3::ZERO;
    let (face, vertex) = from_index(index);
    let pos = context.get_position(face, vertex);
    res.x = pos[0_usize];
    res.y = pos[1_usize];
    res.z = pos[2_usize];
    res
}

fn get_normal_from_index<I: MikkTSpaceInterface<O>, O: Ops>(context: &I, index: usize) -> Vec3<O> {
    let mut res = Vec3::ZERO;
    let (face, vertex) = from_index(index);
    let norm = context.get_normal(face, vertex);
    res.x = norm[0_usize];
    res.y = norm[1_usize];
    res.z = norm[2_usize];
    res
}

fn get_texture_coordinate_from_index<I: MikkTSpaceInterface<O>, O: Ops>(
    context: &I,
    index: usize,
) -> Vec3<O> {
    let mut res = Vec3::ZERO;
    let (face, vertex) = from_index(index);
    let texc = context.get_tex_coord(face, vertex);
    res.x = texc[0_usize];
    res.y = texc[1_usize];
    res.z = 1.0f32;
    res
}

/// returns the texture area times 2
fn calculate_texture_area<I: MikkTSpaceInterface<O>, O: Ops>(
    context: &I,
    indices: &[usize],
) -> f32 {
    let t1 = get_texture_coordinate_from_index(context, indices[0]);
    let t2 = get_texture_coordinate_from_index(context, indices[1]);
    let t3 = get_texture_coordinate_from_index(context, indices[2]);

    let t21x = t2.x - t1.x;
    let t21y = t2.y - t1.y;
    let t31x = t3.x - t1.x;
    let t31y = t3.y - t1.y;

    let signed_area_double = t21x * t31y - t21y * t31x;
    if signed_area_double < 0f32 {
        -signed_area_double
    } else {
        signed_area_double
    }
}

fn initialize_triangle_info<I: MikkTSpaceInterface<O>, O: Ops>(
    triangle_info_list: &mut [TriangleInfo<O>],
    triangle_vertex_list: &[usize],
    context: &I,
) {
    let triangle_count = triangle_info_list.len();
    // triangle_info_list[f].iFlag is cleared in GenerateInitialVerticesIndexList() which is called before this function.

    // generate neighbor info list
    for info in triangle_info_list.iter_mut().take(triangle_count) {
        // assumed bad
        info.flags |= GROUP_WITH_ANY;
    }

    // evaluate first order derivatives
    for f in 0..triangle_count {
        // initial values
        let v1 = get_position_from_index(context, triangle_vertex_list[f * 3]);
        let v2 = get_position_from_index(context, triangle_vertex_list[f * 3 + 1]);
        let v3 = get_position_from_index(context, triangle_vertex_list[f * 3 + 2]);
        let t1 = get_texture_coordinate_from_index(context, triangle_vertex_list[f * 3]);
        let t2 = get_texture_coordinate_from_index(context, triangle_vertex_list[f * 3 + 1]);
        let t3 = get_texture_coordinate_from_index(context, triangle_vertex_list[f * 3 + 2]);

        let t21x = t2.x - t1.x;
        let t21y = t2.y - t1.y;
        let t31x = t3.x - t1.x;
        let t31y = t3.y - t1.y;
        let d1 = v2 - v1;
        let d2 = v3 - v1;
        let signed_area_double = t21x * t31y - t21y * t31x;
        let s = (t31y * d1) - (t21y * d2); // eq 18
        let t = (-t31x * d1) + (t21x * d2); // eq 19

        triangle_info_list[f].flags |= if signed_area_double > 0f32 {
            ORIENT_PRESERVING
        } else {
            0
        };

        if not_zero(signed_area_double) {
            let area_double = fabsf(signed_area_double);
            let s_magnitude = s.length();
            let t_magnitude = t.length();
            let sign = if triangle_info_list[f].flags & ORIENT_PRESERVING == 0 {
                -1.0f32
            } else {
                1.0f32
            };
            if not_zero(s_magnitude) {
                triangle_info_list[f].tangent_space.s = (sign / s_magnitude) * s;
            }
            if not_zero(t_magnitude) {
                triangle_info_list[f].tangent_space.t = (sign / t_magnitude) * t;
            }

            // evaluate magnitudes prior to normalization of vOs and vOt
            triangle_info_list[f].tangent_space.s_magnitude = s_magnitude / area_double;
            triangle_info_list[f].tangent_space.t_magnitude = t_magnitude / area_double;

            // if this is a good triangle
            if not_zero(triangle_info_list[f].tangent_space.s_magnitude)
                && not_zero(triangle_info_list[f].tangent_space.t_magnitude)
            {
                triangle_info_list[f].flags &= !GROUP_WITH_ANY;
            }
        }
    }

    // force otherwise healthy quads to a fixed orientation
    let mut t = 0;
    while t < triangle_count - 1 {
        let original_face_index_a = triangle_info_list[t].original_face_index;
        let original_face_index_b = triangle_info_list[t + 1].original_face_index;
        if original_face_index_a == original_face_index_b {
            // this is a quad
            let is_degenerate_a = triangle_info_list[t].flags & MARK_DEGENERATE != 0;
            let is_degenerate_b = triangle_info_list[t + 1].flags & MARK_DEGENERATE != 0;

            // bad triangles should already have been removed by
            // DegenPrologue(), but just in case check bIsDeg_a and bIsDeg_a are false
            if !(is_degenerate_a || is_degenerate_b) {
                let orientation_preserving_a = triangle_info_list[t].flags & ORIENT_PRESERVING != 0;
                let orientation_preserving_b =
                    triangle_info_list[t + 1].flags & ORIENT_PRESERVING != 0;

                // if this happens the quad has extremely bad mapping!!
                if orientation_preserving_a != orientation_preserving_b {
                    let mut choose_orientation_first_triangle = false;
                    if triangle_info_list[t + 1].flags & GROUP_WITH_ANY != 0
                        || calculate_texture_area(
                            context,
                            &triangle_vertex_list[{
                                let a = t * 3;
                                let b = a + 3;
                                a..b
                            }],
                        ) >= calculate_texture_area(
                            context,
                            &triangle_vertex_list[{
                                let a = (t + 1) * 3;
                                let b = a + 3;
                                a..b
                            }],
                        )
                    {
                        choose_orientation_first_triangle = true;
                    }

                    // force match
                    let t0 = if choose_orientation_first_triangle {
                        t
                    } else {
                        t + 1
                    };
                    let t1_0 = if choose_orientation_first_triangle {
                        t + 1
                    } else {
                        t
                    };

                    // clear first
                    triangle_info_list[t1_0].flags &= !ORIENT_PRESERVING;
                    // copy bit
                    triangle_info_list[t1_0].flags |=
                        triangle_info_list[t0].flags & ORIENT_PRESERVING;
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
    build_neighbors_fast(triangle_info_list, triangle_vertex_list);
}

fn build_4_rule_groups<O: Ops>(
    triangle_info_list: &mut [TriangleInfo<O>],
    triangle_vertex_list: &[usize],
) -> Vec<Group> {
    let triangle_count = triangle_info_list.len();

    let groups_max_count = triangle_count * 3;
    let mut groups = vec![Group::ZERO; groups_max_count];

    let groups_max_count = triangle_count * 3;
    let mut groups_active_count = 0;

    for f in 0..triangle_count {
        for i in 0..3 {
            // if not assigned to a group
            if triangle_info_list[f].flags & GROUP_WITH_ANY == 0
                && triangle_info_list[f].assigned_group[i].is_none()
            {
                let vert_index = triangle_vertex_list[f * 3 + i];
                assert!(groups_active_count < groups_max_count);
                triangle_info_list[f].assigned_group[i] = Some(groups_active_count);
                let this_group = &mut groups[groups_active_count];
                this_group.id = groups_active_count;
                this_group.vertex_representative = vert_index;
                this_group.orientation_preserving =
                    triangle_info_list[f].flags & ORIENT_PRESERVING != 0;
                this_group.face_indices = Vec::new();
                groups_active_count += 1;

                this_group.face_indices.push(f);
                let orientation_preserving_f = triangle_info_list[f].flags & ORIENT_PRESERVING != 0;
                let face_neighbor_index_left = triangle_info_list[f].face_neighbors[i];
                let face_neighbor_index_right =
                    triangle_info_list[f].face_neighbors[if i > 0 { i - 1 } else { 2 }];

                if let Some(face_neighbor_index_left) = face_neighbor_index_left {
                    // neighbor
                    let result = assign_to_group_recursive(
                        triangle_vertex_list,
                        triangle_info_list,
                        face_neighbor_index_left,
                        this_group,
                    );
                    let orientation_preserving_left =
                        triangle_info_list[face_neighbor_index_left].flags & ORIENT_PRESERVING != 0;
                    let different = orientation_preserving_f != orientation_preserving_left;
                    assert!(result || different);
                }
                if let Some(face_neighbor_index_right) = face_neighbor_index_right {
                    // neighbor
                    let result = assign_to_group_recursive(
                        triangle_vertex_list,
                        triangle_info_list,
                        face_neighbor_index_right,
                        this_group,
                    );
                    let orientation_preserving_right =
                        triangle_info_list[face_neighbor_index_right].flags & ORIENT_PRESERVING
                            != 0;
                    let different = orientation_preserving_f != orientation_preserving_right;
                    assert!(result || different);
                }
            }
        }
    }

    groups.truncate(groups_active_count);

    groups
}

fn assign_to_group_recursive<O: Ops>(
    triangle_vertex_list: &[usize],
    triangle_infos: &mut [TriangleInfo<O>],
    triangle_index: usize,
    group: &mut Group,
) -> bool {
    let triangle_info = &mut triangle_infos[triangle_index];

    // track down vertex
    let vertex_representative = group.vertex_representative;
    let vertices = &triangle_vertex_list[{
        let a = 3 * triangle_index;
        let b = a + 3;
        a..b
    }];
    let i = if vertices[0] == vertex_representative {
        0
    } else if vertices[1] == vertex_representative {
        1
    } else if vertices[2] == vertex_representative {
        2
    } else {
        panic!()
    };

    // early out
    if triangle_info.assigned_group[i] == Some(group.id) {
        return true;
    } else if (triangle_info.assigned_group[i]).is_some() {
        return false;
    }
    if triangle_info.flags & GROUP_WITH_ANY != 0
        && (triangle_info.assigned_group[0_usize]).is_none()
        && (triangle_info.assigned_group[1_usize]).is_none()
        && (triangle_info.assigned_group[2_usize]).is_none()
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
    let orientation_preserving = triangle_info.flags & ORIENT_PRESERVING != 0;
    if orientation_preserving != group.orientation_preserving {
        return false;
    }

    group.face_indices.push(triangle_index);
    triangle_info.assigned_group[i] = Some(group.id);

    let face_neighbor_index_left = triangle_info.face_neighbors[i];
    let face_neighbor_index_right = triangle_info.face_neighbors[if i > 0 { i - 1 } else { 2 }];
    if let Some(face_neighbor_index_left) = face_neighbor_index_left {
        assign_to_group_recursive(
            triangle_vertex_list,
            triangle_infos,
            face_neighbor_index_left,
            group,
        );
    }
    if let Some(face_neighbor_index_right) = face_neighbor_index_right {
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
    tangent_spaces_total: usize,
    triangle_info_list: &[TriangleInfo<O>],
    groups: &[Group],
    triangle_vertex_list: &[usize],
    threshold_cos: f32,
    context: &I,
) -> Vec<TangentSpace<O>> {
    let mut tangent_spaces = (0..tangent_spaces_total)
        .map(|_| TangentSpace {
            inner: RawTangentSpace {
                s: Vec3::<O> {
                    x: 1.0,
                    ..Vec3::ZERO
                },
                s_magnitude: 1.0,
                t: Vec3::<O> {
                    y: 1.0,
                    ..Vec3::ZERO
                },
                t_magnitude: 1.0,
            },
            counter: 0,
            orientation_preserving: false,
        })
        .collect::<Vec<_>>();

    let faces_max_count = match groups.iter().map(|group| group.face_indices.len()).max() {
        Some(count) => count,
        None => return tangent_spaces,
    };

    // make initial allocations
    let mut sub_group_tangent_spaces = Vec::<RawTangentSpace<O>>::with_capacity(faces_max_count);
    let mut unified_sub_groups = Vec::<Vec<usize>>::with_capacity(faces_max_count);
    for (g, group) in groups.iter().enumerate() {
        // triangles
        for &f in group.face_indices.iter() {
            let a = &triangle_info_list[f];

            // triangle number
            let index = a
                .assigned_group
                .iter()
                .position(|group| group == &Some(g))
                .unwrap();

            let vertex_index = triangle_vertex_list[f * 3 + index];
            assert!(vertex_index == group.vertex_representative);

            // is normalized already
            let n = get_normal_from_index(context, vertex_index);

            // project
            let mut s_f = a.tangent_space.s - ((n.dot(a.tangent_space.s)) * n);
            let mut t_f = a.tangent_space.t - ((n.dot(a.tangent_space.t)) * n);
            s_f.normalize_or_zero();
            t_f.normalize_or_zero();

            let mut tmp_group = group
                .face_indices
                .iter()
                .copied()
                .filter(|&t| {
                    let b = &triangle_info_list[t];

                    // project
                    let mut s_t = b.tangent_space.s - ((n.dot(b.tangent_space.s)) * n);
                    let mut t_t = b.tangent_space.t - ((n.dot(b.tangent_space.t)) * n);
                    s_t.normalize_or_zero();
                    t_t.normalize_or_zero();

                    let any = (a.flags | b.flags) & GROUP_WITH_ANY != 0;

                    // make sure triangles which belong to the same quad are joined.
                    let same_original_face = a.original_face_index == b.original_face_index;

                    let s_cos = s_f.dot(s_t);
                    let t_cos = t_f.dot(t_t);

                    assert!(f != t || same_original_face, "sanity check");

                    any || same_original_face || s_cos > threshold_cos && t_cos > threshold_cos
                })
                .collect::<Vec<_>>();

            // sort pTmpMembers
            tmp_group.sort();

            // look for an existing match
            let found = unified_sub_groups.iter().position(|g| g == &tmp_group);

            let l = match found {
                Some(l) => l,
                None => {
                    // if no match was found we allocate a new subgroup
                    let l = sub_group_tangent_spaces.len();
                    sub_group_tangent_spaces.push(evaluate_tangent_space(
                        &tmp_group,
                        triangle_vertex_list,
                        triangle_info_list,
                        context,
                        group.vertex_representative,
                    ));
                    unified_sub_groups.push(tmp_group);
                    l
                }
            };

            // output tspace
            let index = a.tangent_spaces_offset + a.vertex_indices[index] as usize;
            let tangent_space = &mut tangent_spaces[index];

            assert!((a.flags & ORIENT_PRESERVING != 0) == group.orientation_preserving);

            tangent_space.inner = match tangent_space.counter {
                0 => sub_group_tangent_spaces[l],
                1 => mean_tangent_space(tangent_space.inner, sub_group_tangent_spaces[l]),
                _ => panic!("counter should always be zero or one at this stage"),
            };

            tangent_space.counter += 1;
            tangent_space.orientation_preserving = group.orientation_preserving;
        }

        sub_group_tangent_spaces.clear();
        unified_sub_groups.clear();
    }

    tangent_spaces
}

fn evaluate_tangent_space<I: MikkTSpaceInterface<O>, O: Ops>(
    face_indices: &[usize],
    triangle_vertex_list: &[usize],
    triangle_info_list: &[TriangleInfo<O>],
    context: &I,
    vertex_representative: usize,
) -> RawTangentSpace<O> {
    let mut res = RawTangentSpace {
        s: Vec3::ZERO,
        s_magnitude: 0.,
        t: Vec3::ZERO,
        t_magnitude: 0.,
    };

    let angle_sum = face_indices
        .iter()
        .map(|&f| (&triangle_vertex_list[3 * f..][..3], &triangle_info_list[f]))
        // only valid triangles get to add their contribution
        .filter(|(_vertices, info)| info.flags & GROUP_WITH_ANY == 0)
        .fold(0f32, |angle_sum, (vertices, info)| {
            let i = (0..=2)
                .find(|&i| vertices[i] == vertex_representative)
                .unwrap();

            let n = get_normal_from_index(context, vertices[i]);
            let p = [(i + 1) % 3, i, (i + 2) % 3]
                .map(|i| vertices[i])
                .map(|i| get_position_from_index(context, i));
            let v = [p[0] - p[1], p[2] - p[1]]
                .map(|v| v - ((n.dot(v)) * n))
                .map(Vec3::normalized_or_zero);

            // weight contribution by the angle
            // between the two edge vectors
            let cos = v[0].dot(v[1]).clamp(-1f32, 1f32);
            let angle = O::acos(cos as f64) as f32;

            let t = [info.tangent_space.s, info.tangent_space.t]
                .map(|t| t - (n.dot(t) * n))
                .map(Vec3::normalized_or_zero)
                .map(|t| angle * t);
            let t_mag = [
                info.tangent_space.s_magnitude,
                info.tangent_space.t_magnitude,
            ]
            .map(|t| angle * t);

            res.s = res.s + t[0];
            res.t = res.t + t[1];
            res.s_magnitude += t_mag[0];
            res.t_magnitude += t_mag[1];

            angle_sum + angle
        });

    // normalize
    res.s.normalize_or_zero();
    res.t.normalize_or_zero();
    if angle_sum > 0f32 {
        res.s_magnitude /= angle_sum;
        res.t_magnitude /= angle_sum;
    }

    res
}

fn build_neighbors_fast<O: Ops>(triangles: &mut [TriangleInfo<O>], vertices: &[usize]) {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
    struct Edge {
        i0: usize,
        i1: usize,
        f: usize,
    }

    // build array of edges
    let mut edges = vertices
        .chunks_exact(3)
        .take(triangles.len())
        .enumerate()
        .flat_map(|(f, chunk)| {
            chunk
                .iter()
                .zip(chunk.iter().cycle().skip(1))
                .take(3)
                .map(move |(a, b)| (f, a.min(b), a.max(b)))
        })
        .map(|(f, &i0, &i1)| Edge { i0, i1, f })
        .collect::<Vec<_>>();

    // Sort over all edges by i0, this is the pricy one.
    // Sorts using the `Ord` implementation from `Edge` and `[Edge]::sort`.
    // This is a correct and typical sort, but differs from the original C
    // library.
    #[cfg(feature = "corrected-edge-sorting")]
    edges.sort();

    // Sorts using the original quicksort implementation from the C library.
    // Note that this includes an off-by-one error which can cause the last
    // step in sorting to fail.
    // This is typically observed as the verticies in the last face being
    // out of order.
    #[cfg(not(feature = "corrected-edge-sorting"))]
    {
        quick_sort_by_key_with_seed(&mut edges, |e| e.i0, INTERNAL_RND_SORT_SEED);

        let mut s = 0;
        for i in 1..edges.len() {
            if edges[s].i0 == edges[i].i0 {
                continue;
            }

            quick_sort_by_key_with_seed(&mut edges[s..i], |e| e.i1, INTERNAL_RND_SORT_SEED);
            s = i;
        }

        let mut s = 0;
        for i in 1..edges.len() {
            if edges[s].i0 == edges[i].i0 && edges[s].i1 == edges[i].i1 {
                continue;
            }

            quick_sort_by_key_with_seed(&mut edges[s..i], |e| e.f, INTERNAL_RND_SORT_SEED);
            s = i;
        }
    }

    // pair up, adjacent triangles
    let mut iter = edges.iter();
    while let Some(a) = iter.next() {
        // resolve index ordering and edge_num
        let (n_a, i0_a, i1_a) = get_edge(&vertices[a.f * 3..][..3], a.i0, a.i1).unwrap();

        if triangles[a.f].face_neighbors[n_a].is_some() {
            continue;
        }

        // get true index ordering
        let found = iter
            .clone()
            .take_while(|b| (a.i0, a.i1) == (b.i0, b.i1))
            .find_map(|b| {
                // flip i0_B and i1_B
                // resolve index ordering and edge_num
                let (n_b, i1_b, i0_b) = get_edge(&vertices[b.f * 3..][..3], b.i0, b.i1).unwrap();
                let unassigned_b = triangles[b.f].face_neighbors[n_b].is_none();

                ((i0_a, i1_a) == (i0_b, i1_b) && unassigned_b).then_some((b, n_b))
            });

        let Some((b, n_b)) = found else {
            continue;
        };

        triangles[a.f].face_neighbors[n_a] = Some(b.f);
        triangles[b.f].face_neighbors[n_b] = Some(a.f);
    }
}

/// Note that this method _should_ be able to be replaced with `[T]::sort` and an
/// appropriate implementation of [`Ord`] for [`SEdge`].
/// However, in initial testing this caused incorrect results, indicating this sort
/// may not be implemented correctly.
/// Further testing is required.
fn quick_sort_by_key_with_seed<T, K: Ord>(sort_buffer: &mut [T], key: fn(&T) -> K, seed: u32) {
    match sort_buffer.len() {
        0 | 1 => return,
        2 => {
            sort_buffer.sort_by_key(key);
            return;
        }
        _ => {}
    }

    let seed = {
        let t = seed & 31;
        let t = seed.wrapping_shl(t) | seed.wrapping_shr(32_u32.wrapping_sub(t));
        seed.wrapping_add(t).wrapping_add(3)
    };

    let pivot = key(&sort_buffer[seed.wrapping_rem(sort_buffer.len() as u32) as usize]);

    let (mut l, mut r) = (0, sort_buffer.len().saturating_sub(1));
    while l <= r {
        l = (l..sort_buffer.len())
            .find(|&left| key(&sort_buffer[left]) >= pivot)
            .unwrap();

        r = (0..=r)
            .rev()
            .find(|&right| key(&sort_buffer[right]) <= pivot)
            .unwrap();

        if l <= r {
            sort_buffer.swap(l, r);
            l = l.saturating_add(1);
            r = r.saturating_sub(1);
        }
    }

    quick_sort_by_key_with_seed(&mut sort_buffer[..=r], key, seed);
    quick_sort_by_key_with_seed(&mut sort_buffer[l..], key, seed);
}

/// Finds the index of the edge `(i0_in, i1_in)` within `indices`, additionally
/// returning `i0_in` and `i1_in` in the same order as they are stored within `indices`.
fn get_edge(indices: &[usize], i0: usize, i1: usize) -> Option<(usize, usize, usize)> {
    indices
        .iter()
        .copied()
        .zip(indices.iter().copied().cycle().skip(1))
        .enumerate()
        .find(|&(_, (a, b))| (a.min(b), a.max(b)) == (i0.min(i1), i0.max(i1)))
        .map(|(edgenum, (a, b))| (edgenum, a, b))
}

fn degen_prologue<I: MikkTSpaceInterface<O>, O: Ops>(
    context: &I,
    faces: &mut [TriangleInfo<O>],
    vertices: &mut [usize],
) -> usize {
    // Mark & count all degenerate triangles
    let triangles_degenerate_count = (0..faces.len())
        .zip(vertices.chunks_exact(3))
        .filter(|(_, i)| {
            let iter = i
                .iter()
                .cycle()
                .map(|&i| get_position_from_index(context, i));
            iter.clone().zip(iter.skip(1)).take(3).any(|(a, b)| a == b)
        })
        .inspect(|(t, _)| faces[*t].flags |= MARK_DEGENERATE)
        .count();

    let triangle_count = faces.len() - triangles_degenerate_count;

    // locate quads with only one good triangle
    for chunk in faces.chunk_by_mut(|a, b| a.original_face_index == b.original_face_index) {
        let [a, b] = chunk else { continue };

        // this is a quad
        let is_degenerate_a = a.flags & MARK_DEGENERATE != 0;
        let is_degenerate_b = b.flags & MARK_DEGENERATE != 0;
        if is_degenerate_a ^ is_degenerate_b {
            a.flags |= QUAD_ONE_DEGEN_TRI;
            b.flags |= QUAD_ONE_DEGEN_TRI;
        }
    }

    // reorder list so all degen triangles are moved to the back
    // without reordering the good triangles
    let mut sorted = 0..triangle_count;
    let mut unsorted = 0..faces.len();
    // search for the first degenerate triangle.
    while let Some(a) = sorted.find(|&a| faces[a].flags & MARK_DEGENERATE != 0) {
        unsorted.start = unsorted.start.max(a + 1);

        // search for the first good triangle.
        let b = unsorted
            .find(|&b| faces[b].flags & MARK_DEGENERATE == 0)
            .expect("this is not supposed to happen");

        // swap triangle a and b
        for i in 0..3 {
            vertices.swap(a * 3 + i, b * 3 + i);
        }
        faces.swap(a, b);
    }

    triangles_degenerate_count
}

fn degen_epilogue<I: MikkTSpaceInterface<O>, O: Ops>(
    // Using `impl Copy` to highlight this function doesn't interact with tangent
    // space values.
    tangent_spaces: &mut [impl Copy],
    triangle_info_list: &[TriangleInfo<O>],
    triangle_vertex_list: &[usize],
    context: &I,
    triangle_count: usize,
) {
    // deal with degenerate triangles
    // punishment for degenerate triangles is O(N^2)
    let full_bad_fixes = (triangle_count..triangle_info_list.len())
        .filter(|&t| {
            // degenerate triangles on a quad with one good triangle are skipped
            // here but processed in the next loop
            triangle_info_list[t].flags & QUAD_ONE_DEGEN_TRI == 0
        })
        .flat_map(|t| (0..3).map(move |i| (t, i)))
        .filter_map(|(t, i)| {
            // search through the good triangles
            (0..(3 * triangle_count))
                .find(|&j| triangle_vertex_list[t * 3 + i] == triangle_vertex_list[j])
                .map(|j| (t, i, j / 3, j % 3))
        })
        .map(|(dst, v_dst, src, v_src)| {
            let vertex_dst = triangle_info_list[dst].vertex_indices[v_dst] as usize;
            let vertex_src = triangle_info_list[src].vertex_indices[v_src] as usize;

            let dst = triangle_info_list[dst].tangent_spaces_offset + vertex_dst;
            let src = triangle_info_list[src].tangent_spaces_offset + vertex_src;

            (dst, src)
        });

    // deal with degenerate quads with one good triangle
    let partial_bad_fixes = triangle_info_list
        .iter()
        .take(triangle_count)
        // this triangle belongs to a quad where the
        // other triangle is degenerate
        .filter(|triangle_info| triangle_info.flags & QUAD_ONE_DEGEN_TRI != 0)
        .map(|triangle_info| {
            let dst = (0..=3)
                .find(|v| !triangle_info.vertex_indices.contains(v))
                .unwrap() as usize;

            let offset = triangle_info.tangent_spaces_offset;
            let face = triangle_info.original_face_index;
            let missing_position = context.get_position(face, dst);

            let src = (0..3)
                .find_map(|i| {
                    let vertex = triangle_info.vertex_indices[i] as usize;
                    let source_position = context.get_position(face, vertex);
                    (source_position == missing_position).then_some(vertex)
                })
                .unwrap();

            (dst + offset, src + offset)
        });

    for (dst, src) in full_bad_fixes.chain(partial_bad_fixes) {
        tangent_spaces[dst] = tangent_spaces[src]
    }
}
