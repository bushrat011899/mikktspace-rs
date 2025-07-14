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

mod face_vertex;
#[cfg_attr(
    not(feature = "corrected-edge-sorting"),
    path = "mikktspace/quick_sort_legacy.rs"
)]
mod quick_sort;
#[cfg_attr(
    not(feature = "corrected-vertex-welding"),
    path = "mikktspace/weld_vertices_legacy.rs"
)]
mod weld_vertices;

use alloc::{vec, vec::Vec};

use self::{face_vertex::FaceVertex, quick_sort::quick_sort_edges, weld_vertices::weld_vertices};
use crate::{math::*, MikkTSpaceInterface};

pub(crate) fn generate_tangent_space<I: MikkTSpaceInterface<O>, O: Ops>(
    context: &mut I,
    angular_threshold: f32,
) -> Result<(), GenerateTangentSpaceError> {
    let threshold_cos = O::cos(deg_to_rad(angular_threshold) as f64) as f32;
    let faces_total = context.get_num_faces();

    // make an initial triangle --> face index list
    let mut triangle_info_list = generate_triangle_info_list(context, faces_total);

    // mark all triangles where one or more vertices are coincident.
    mark_degenerate_triangles(context, &mut triangle_info_list);

    // mark all triangle pairs that belong to a quad with only one
    // good triangle.
    mark_partially_degenerate_quads(&mut triangle_info_list);

    // make a welded index list of identical positions and attributes (pos, norm, texc)
    let mut triangle_vertex_list = triangle_info_list
        .iter()
        .flat_map(|info| {
            info.vertex_indices
                .map(|t| FaceVertex::new(info.original_face_index, t))
        })
        .collect::<Vec<_>>();
    weld_vertices(context, &mut triangle_vertex_list);

    // `generate_triangle_info_list` generates `triangle_info_list` such that the
    // last value will have the largest `tangent_spaces_offset`.
    debug_assert!(triangle_info_list.is_sorted_by_key(|info| info.tangent_spaces_offset));

    // Get the total number of tangent space values to be computed
    let tangent_spaces_total = triangle_info_list
        .last()
        .map(|info| {
            info.tangent_spaces_offset + context.get_num_vertices_of_face(info.original_face_index)
        })
        .unwrap_or(0);

    // Move all good triangles to the start of triangle_info_list[] and triangle_vertex_list[]
    // without changing order and put the degenerate triangles last.
    let ((triangles_good, triangles_degenerate), (vertices_good, vertices_degenerate)) =
        segregate_degenerate_triangles(&mut triangle_info_list, &mut triangle_vertex_list);

    evaluate_first_order_derivatives(triangles_good, context);

    fix_quad_orientation(triangles_good, context);

    build_neighbors(triangles_good, &*vertices_good);

    // based on the 4 rules, identify groups based on connectivity
    let groups = build_4_rule_groups(triangles_good, &*vertices_good);

    // make tspaces, each group is split up into subgroups if necessary
    // based on fAngularThreshold. Finally a tangent space is made for
    // every resulting subgroup
    let mut tangent_spaces = generate_tangent_spaces(
        tangent_spaces_total,
        &*triangles_good,
        &groups,
        &*vertices_good,
        threshold_cos,
        context,
    );

    // all other degenerate triangles will just copy a space from any good triangle
    // with the same welded index in triangle_vertex_list[].
    generate_tangent_spaces_for_degenerate_triangles(
        &mut tangent_spaces,
        &*triangles_good,
        &*triangles_degenerate,
        &*vertices_good,
        &*vertices_degenerate,
    );

    // degenerate quads with one good triangle will be fixed by copying a space from
    // the good triangle to the coinciding vertex.
    generate_tangent_spaces_for_partially_degenerate_quads(
        &mut tangent_spaces,
        &*triangles_good,
        context,
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
pub(crate) enum GenerateTangentSpaceError {}

impl core::fmt::Display for GenerateTangentSpaceError {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unreachable!()
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

impl<O: Ops> PartialEq for RawTangentSpace<O> {
    fn eq(&self, other: &Self) -> bool {
        self.s == other.s
            && self.t == other.t
            && self.s_magnitude == other.s_magnitude
            && self.t_magnitude == other.t_magnitude
    }
}

impl<O: Ops> Copy for RawTangentSpace<O> {}

impl<O: Ops> Clone for RawTangentSpace<O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<O: Ops> RawTangentSpace<O> {
    fn mean(self, rhs: Self) -> Self {
        // this if is important. Due to floating point precision
        // averaging when ts0==ts1 will cause a slight difference
        // which results in tangent space splits later on
        if self == rhs {
            self
        } else {
            // Note that the magnitudes here may not be the value you're expecting.
            // If `lhs.s` and `rhs.t` are `a` and `-a` (for some arbitrary vector `a`),
            // the expected mean would be `0`.
            // However, this method will instead return `a` (as `(a + a) / 2 == a`).
            //
            // TODO: Determine if `s_magnitude` and `t_magnitude` are being calculated incorrectly.
            //       This matches the C implementation, but may be incorrect.
            RawTangentSpace {
                s_magnitude: 0.5f32 * (self.s_magnitude + rhs.s_magnitude),
                t_magnitude: 0.5f32 * (self.t_magnitude + rhs.t_magnitude),
                s: (self.s + rhs.s).normalized_or_zero(),
                t: (self.t + rhs.t).normalized_or_zero(),
            }
        }
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

    flags: u8,
    tangent_spaces_offset: usize,
    vertex_indices: [u8; 3],
}

impl<O: Ops> Copy for TriangleInfo<O> {}

impl<O: Ops> Clone for TriangleInfo<O> {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Clone)]
struct Group {
    id: usize,
    face_indices: Vec<usize>,
    vertex_representative: FaceVertex,
    orientation_preserving: bool,
}

const MARK_DEGENERATE: u8 = 1;
const QUAD_ONE_DEGEN_TRI: u8 = 2;
const GROUP_WITH_ANY: u8 = 4;
const ORIENT_PRESERVING: u8 = 8;

fn generate_triangle_info_list<I: MikkTSpaceInterface<O>, O: Ops>(
    context: &I,
    faces_total: usize,
) -> Vec<TriangleInfo<O>> {
    (0..faces_total)
        .filter_map(|f| match context.get_num_vertices_of_face(f) {
            verts @ (3 | 4) => Some((f, verts)),
            _ => None,
        })
        .scan(0, |tangent_space_offset, (f, verts)| {
            let result = (f, verts, *tangent_space_offset);
            *tangent_space_offset += verts;
            Some(result)
        })
        .flat_map(|(f, verts, tangent_space_offset)| {
            let mut info = TriangleInfo {
                face_neighbors: [None; 3],
                assigned_group: [None; 3],
                tangent_space: RawTangentSpace {
                    s: Vec3::<O>::ZERO,
                    t: Vec3::<O>::ZERO,
                    s_magnitude: 0.,
                    t_magnitude: 0.,
                },
                original_face_index: f,
                flags: 0,
                tangent_spaces_offset: tangent_space_offset,
                vertex_indices: [0; 3],
            };

            if verts == 3 {
                info.vertex_indices = [0, 1, 2];

                [Some(info), None]
            } else if verts == 4 {
                let mut info_a = info;
                let mut info_b = info;

                // need an order independent way to evaluate
                // tspace on quads. This is done by splitting
                // along the shortest diagonal.
                let i = [0, 1, 2, 3].map(|i| FaceVertex::new(f, i));
                let tx = i.map(|i| get_texture_coordinate_from_index(context, i));
                let d = [(2, 0), (3, 1)].map(|(a, b)| [0, 1].map(|i| tx[a][i] - tx[b][i]));
                let l = d.map(|d| d[0] * d[0] + d[1] * d[1]);

                let quad_diagonal_is_02 =
                    match l[0].partial_cmp(&l[1]) {
                        Some(core::cmp::Ordering::Less) => true,
                        Some(core::cmp::Ordering::Greater) => false,
                        _ => {
                            let p = i.map(|i| get_position_from_index(context, i));
                            let d = [(2, 0), (3, 1)].map(|(a, b)| (p[a] - p[b]).length_squared());
                            d[1] >= d[0]
                        }
                    };

                if quad_diagonal_is_02 {
                    info_a.vertex_indices = [0, 1, 2];
                    info_b.vertex_indices = [0, 2, 3];
                } else {
                    info_a.vertex_indices = [0, 1, 3];
                    info_b.vertex_indices = [1, 2, 3];
                }

                [Some(info_a), Some(info_b)]
            } else {
                unreachable!()
            }
        })
        .flatten()
        .collect::<Vec<_>>()
}

fn get_position_from_index<I: MikkTSpaceInterface<O>, O: Ops>(
    context: &I,
    index: FaceVertex,
) -> Vec3<O> {
    context
        .get_position(index.face(), index.vertex() as usize)
        .into()
}

fn get_normal_from_index<I: MikkTSpaceInterface<O>, O: Ops>(
    context: &I,
    index: FaceVertex,
) -> Vec3<O> {
    context
        .get_normal(index.face(), index.vertex() as usize)
        .into()
}

fn get_texture_coordinate_from_index<I: MikkTSpaceInterface<O>, O: Ops>(
    context: &I,
    index: FaceVertex,
) -> [f32; 2] {
    context.get_tex_coord(index.face(), index.vertex() as usize)
}

/// returns the texture area times 2
fn calculate_texture_area<I: MikkTSpaceInterface<O>, O: Ops>(
    context: &I,
    indices: &[FaceVertex],
) -> f32 {
    let tx = [0, 1, 2]
        .map(|i| indices[i])
        .map(|i| get_texture_coordinate_from_index(context, i));

    let d_tx = [1, 2].map(|t| [0, 1].map(|i| tx[t][i] - tx[0][i]));

    let signed_area_double = d_tx[0][0] * d_tx[1][1] - d_tx[0][1] * d_tx[1][0];

    fabsf(signed_area_double)
}

fn fix_quad_orientation<I: MikkTSpaceInterface<O>, O: Ops>(
    triangle_info_list: &mut [TriangleInfo<O>],
    context: &I,
) {
    // force otherwise healthy quads to a fixed orientation
    triangle_info_list
        .chunk_by_mut(|a, b| a.original_face_index == b.original_face_index)
        // this is a quad
        .filter_map(|chunk| match chunk {
            [a, b] => Some([a, b]),
            _ => None,
        })
        // bad triangles should already have been removed by
        // DegenPrologue(), but just in case check bIsDeg_a and bIsDeg_a are false
        .filter(|quad| quad.iter().all(|a| a.flags & MARK_DEGENERATE == 0))
        // if this happens the quad has extremely bad mapping!!
        .filter(|[a, b]| (a.flags & ORIENT_PRESERVING != 0) != (b.flags & ORIENT_PRESERVING != 0))
        .map(|[a, b]| {
            let tx_area_a = calculate_texture_area(
                context,
                &a.vertex_indices
                    .map(|i| FaceVertex::new(a.original_face_index, i)),
            );

            let tx_area_b = calculate_texture_area(
                context,
                &b.vertex_indices
                    .map(|i| FaceVertex::new(b.original_face_index, i)),
            );

            // force match
            if b.flags & GROUP_WITH_ANY != 0 || tx_area_a >= tx_area_b {
                (a, b)
            } else {
                (b, a)
            }
        })
        .for_each(|(a, b)| {
            // clear first
            b.flags &= !ORIENT_PRESERVING;
            // copy bit
            b.flags |= a.flags & ORIENT_PRESERVING;
        });
}

fn evaluate_first_order_derivatives<I: MikkTSpaceInterface<O>, O: Ops>(
    triangle_info_list: &mut [TriangleInfo<O>],
    context: &I,
) {
    // triangle_info_list[f].iFlag is cleared in GenerateInitialVerticesIndexList() which is called before this function.
    // generate neighbor info list
    // evaluate first order derivatives
    for info in triangle_info_list.iter_mut() {
        // assumed bad
        info.flags |= GROUP_WITH_ANY;

        // initial values
        let v = info
            .vertex_indices
            .map(|i| context.get_position(info.original_face_index, i as usize))
            .map(Vec3::<O>::from);
        let tx = info
            .vertex_indices
            .map(|i| context.get_tex_coord(info.original_face_index, i as usize));

        let d_tx = [1, 2].map(|t| [0, 1].map(|i| tx[t][i] - tx[0][i]));
        let d_v = [1, 2].map(|i| v[i] - v[0]);

        let signed_area_double = d_tx[0][0] * d_tx[1][1] - d_tx[0][1] * d_tx[1][0];
        let s = (d_tx[1][1] * d_v[0]) - (d_tx[0][1] * d_v[1]); // eq 18
        let t = (-d_tx[1][0] * d_v[0]) + (d_tx[0][0] * d_v[1]); // eq 19

        info.flags |= if signed_area_double > 0f32 {
            ORIENT_PRESERVING
        } else {
            0
        };

        if not_zero(signed_area_double) {
            let area_double = fabsf(signed_area_double);
            let s_magnitude = s.length();
            let t_magnitude = t.length();
            let sign = if info.flags & ORIENT_PRESERVING == 0 {
                -1.0f32
            } else {
                1.0f32
            };
            if not_zero(s_magnitude) {
                info.tangent_space.s = (sign / s_magnitude) * s;
            }
            if not_zero(t_magnitude) {
                info.tangent_space.t = (sign / t_magnitude) * t;
            }

            // evaluate magnitudes prior to normalization of vOs and vOt
            info.tangent_space.s_magnitude = s_magnitude / area_double;
            info.tangent_space.t_magnitude = t_magnitude / area_double;

            // if this is a good triangle
            if not_zero(info.tangent_space.s_magnitude) && not_zero(info.tangent_space.t_magnitude)
            {
                info.flags &= !GROUP_WITH_ANY;
            }
        }
    }
}

fn build_4_rule_groups<O: Ops>(
    triangle_info_list: &mut [TriangleInfo<O>],
    triangle_vertex_list: &[FaceVertex],
) -> Vec<Group> {
    let mut ids = 0..;
    (0..triangle_info_list.len())
        .zip(triangle_vertex_list.chunks_exact(3))
        .flat_map(|(f, chunk)| chunk.iter().enumerate().map(move |(i, &vert)| (f, i, vert)))
        .filter_map(|(f, i, vertex_representative)| {
            if triangle_info_list[f].flags & GROUP_WITH_ANY != 0
                || triangle_info_list[f].assigned_group[i].is_some()
            {
                // if not assigned to a group
                return None;
            }

            let id = ids.next().unwrap();

            triangle_info_list[f].assigned_group[i] = Some(id);

            let mut group = Group {
                id,
                vertex_representative,
                orientation_preserving: triangle_info_list[f].flags & ORIENT_PRESERVING != 0,
                face_indices: vec![f],
            };

            let orientation_preserving_f = triangle_info_list[f].flags & ORIENT_PRESERVING != 0;

            let neighbors = [2, 0]
                .map(|t| (i + t) % 3)
                .map(|i| triangle_info_list[f].face_neighbors[i]);

            for &neighbor in neighbors.iter().flatten() {
                // neighbor
                let result = assign_to_group_recursive(
                    triangle_vertex_list,
                    triangle_info_list,
                    neighbor,
                    &mut group,
                );

                assert!({
                    let orientation_preserving_left =
                        triangle_info_list[neighbor].flags & ORIENT_PRESERVING != 0;
                    let different = orientation_preserving_f != orientation_preserving_left;

                    result || different
                });
            }

            Some(group)
        })
        .collect::<Vec<_>>()
}

fn assign_to_group_recursive<O: Ops>(
    triangle_vertex_list: &[FaceVertex],
    triangle_infos: &mut [TriangleInfo<O>],
    triangle_index: usize,
    group: &mut Group,
) -> bool {
    let triangle_info = &mut triangle_infos[triangle_index];

    // track down vertex
    let i = triangle_vertex_list
        .chunks_exact(3)
        .nth(triangle_index)
        .unwrap()
        .iter()
        .position(|&v| v == group.vertex_representative)
        .unwrap();

    // early out
    if let Some(id) = triangle_info.assigned_group[i] {
        return id == group.id;
    }

    if triangle_info.flags & GROUP_WITH_ANY != 0
        && triangle_info.assigned_group[0].is_none()
        && triangle_info.assigned_group[1].is_none()
        && triangle_info.assigned_group[2].is_none()
    {
        // first to group with a group-with-anything triangle
        // determines it's orientation.
        // This is the only existing order dependency in the code!!
        triangle_info.flags &= !ORIENT_PRESERVING;
        if group.orientation_preserving {
            triangle_info.flags |= ORIENT_PRESERVING
        }
    }

    let orientation_preserving = triangle_info.flags & ORIENT_PRESERVING != 0;
    if orientation_preserving != group.orientation_preserving {
        return false;
    }

    group.face_indices.push(triangle_index);
    triangle_info.assigned_group[i] = Some(group.id);

    let neighbors = [2, 0]
        .map(|t| (i + t) % 3)
        .map(|i| triangle_info.face_neighbors[i]);
    for &neighbor in neighbors.iter().flatten() {
        assign_to_group_recursive(triangle_vertex_list, triangle_infos, neighbor, group);
    }

    true
}

fn generate_tangent_spaces<I: MikkTSpaceInterface<O>, O: Ops>(
    tangent_spaces_total: usize,
    triangle_info_list: &[TriangleInfo<O>],
    groups: &[Group],
    triangle_vertex_list: &[FaceVertex],
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
                1 => tangent_space.inner.mean(sub_group_tangent_spaces[l]),
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
    triangle_vertex_list: &[FaceVertex],
    triangle_info_list: &[TriangleInfo<O>],
    context: &I,
    vertex_representative: FaceVertex,
) -> RawTangentSpace<O> {
    let (angle_sum, mut res) = face_indices
        .iter()
        .map(|&f| (&triangle_vertex_list[3 * f..][..3], &triangle_info_list[f]))
        // only valid triangles get to add their contribution
        .filter(|(_vertices, info)| info.flags & GROUP_WITH_ANY == 0)
        .fold(
            (
                0f32,
                RawTangentSpace {
                    s: Vec3::ZERO,
                    s_magnitude: 0.,
                    t: Vec3::ZERO,
                    t_magnitude: 0.,
                },
            ),
            |(angle_sum, mut res), (vertices, info)| {
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

                (angle_sum + angle, res)
            },
        );

    // normalize
    res.s.normalize_or_zero();
    res.t.normalize_or_zero();
    if angle_sum > 0f32 {
        res.s_magnitude /= angle_sum;
        res.t_magnitude /= angle_sum;
    }

    res
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Edge {
    i0: FaceVertex,
    i1: FaceVertex,
    f: usize,
}

fn build_neighbors<O: Ops>(triangles: &mut [TriangleInfo<O>], vertices: &[FaceVertex]) {
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

    // Sort over all edges by i0, i1, then f.
    // This is the pricy one.
    quick_sort_edges(&mut edges);

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

/// Finds the index of the edge `(i0_in, i1_in)` within `indices`, additionally
/// returning `i0_in` and `i1_in` in the same order as they are stored within `indices`.
fn get_edge(
    indices: &[FaceVertex],
    i0: FaceVertex,
    i1: FaceVertex,
) -> Option<(usize, FaceVertex, FaceVertex)> {
    indices
        .iter()
        .copied()
        .zip(indices.iter().copied().cycle().skip(1))
        .enumerate()
        .find(|&(_, (a, b))| (a.min(b), a.max(b)) == (i0.min(i1), i0.max(i1)))
        .map(|(edgenum, (a, b))| (edgenum, a, b))
}

fn mark_degenerate_triangles<I: MikkTSpaceInterface<O>, O: Ops>(
    context: &I,
    faces: &mut [TriangleInfo<O>],
) {
    faces
        .iter_mut()
        .filter(|face| {
            let p = face
                .vertex_indices
                .map(|i| context.get_position(face.original_face_index, i as usize));
            let iter = p.iter().cycle();
            iter.clone().zip(iter.skip(1)).take(3).any(|(a, b)| a == b)
        })
        .for_each(|face| face.flags |= MARK_DEGENERATE);
}

fn mark_partially_degenerate_quads<O: Ops>(faces: &mut [TriangleInfo<O>]) {
    faces
        .chunk_by_mut(|a, b| a.original_face_index == b.original_face_index)
        .filter(|faces| faces.len() == 2)
        .filter(|faces| {
            faces
                .iter()
                .filter(|f| f.flags & MARK_DEGENERATE != 0)
                .count()
                == 1
        })
        .flatten()
        .for_each(|face| face.flags |= QUAD_ONE_DEGEN_TRI);
}

/// Sort `faces` and `vertices` into a "good" first section, and a degenerate second section.
#[expect(clippy::type_complexity)]
fn segregate_degenerate_triangles<'faces, 'vertices, O: Ops>(
    faces: &'faces mut [TriangleInfo<O>],
    vertices: &'vertices mut [FaceVertex],
) -> (
    (&'faces mut [TriangleInfo<O>], &'faces mut [TriangleInfo<O>]),
    (&'vertices mut [FaceVertex], &'vertices mut [FaceVertex]),
) {
    // reorder list so all degen triangles are moved to the back
    // without reordering the good triangles
    let (mut proper, mut degenerate) = (0..faces.len(), 0..faces.len());
    loop {
        // search for the first degenerate triangle.
        let Some(a) = proper.find(|&a| faces[a].flags & MARK_DEGENERATE != 0) else {
            break;
        };

        // To preserve local ordering, only swap with a good triangle after this degenerate
        degenerate.start = degenerate.start.max(a + 1);

        // search for the first good triangle.
        let Some(b) = degenerate.find(|&b| faces[b].flags & MARK_DEGENERATE == 0) else {
            // If there are no more good triangles, the sorting is complete.
            break;
        };

        // swap triangle a and b
        for i in 0..3 {
            vertices.swap(a * 3 + i, b * 3 + i);
        }
        faces.swap(a, b);
    }

    let good_triangles_count = faces.partition_point(|face| face.flags & MARK_DEGENERATE == 0);
    let good_vertices_count = 3 * good_triangles_count;

    (
        faces.split_at_mut(good_triangles_count),
        vertices.split_at_mut(good_vertices_count),
    )
}

/// Populate tangent space values for degenerate triangles from adjacent "good" triangles.
fn generate_tangent_spaces_for_degenerate_triangles<O: Ops>(
    // Using `impl Copy` to highlight this function doesn't interact with tangent
    // space values.
    tangent_spaces: &mut [impl Copy],
    triangles_good: &[TriangleInfo<O>],
    triangle_degenerate: &[TriangleInfo<O>],
    vertices_good: &[FaceVertex],
    vertices_degenerate: &[FaceVertex],
) {
    // deal with degenerate triangles
    // punishment for degenerate triangles is O(N^2)
    triangle_degenerate
        .iter()
        .zip(vertices_degenerate.chunks_exact(3))
        .filter(|(triangle, _chunk)| {
            // degenerate triangles on a quad with one good triangle are skipped
            // here but processed in the next loop
            triangle.flags & QUAD_ONE_DEGEN_TRI == 0
        })
        .flat_map(|(a, chunk)| chunk.iter().enumerate().map(move |(i, av)| (a, av, i)))
        .filter_map(|(a, av, i)| {
            // search through the good triangles
            triangles_good
                .iter()
                .zip(vertices_good.chunks_exact(3))
                .flat_map(|(b, chunk)| chunk.iter().enumerate().map(move |(j, bv)| (b, bv, j)))
                .find_map(|(b, bv, j)| (av == bv).then_some((a, i, b, j)))
        })
        .map(|(a, i, b, j)| {
            let vertex_dst = a.vertex_indices[i] as usize;
            let vertex_src = b.vertex_indices[j] as usize;

            let dst = a.tangent_spaces_offset + vertex_dst;
            let src = b.tangent_spaces_offset + vertex_src;

            (dst, src)
        })
        .for_each(|(dst, src)| {
            tangent_spaces[dst] = tangent_spaces[src];
        });
}

fn generate_tangent_spaces_for_partially_degenerate_quads<I: MikkTSpaceInterface<O>, O: Ops>(
    // Using `impl Copy` to highlight this function doesn't interact with tangent
    // space values.
    tangent_spaces: &mut [impl Copy],
    triangle_info_list: &[TriangleInfo<O>],
    context: &I,
) {
    // deal with degenerate quads with one good triangle
    triangle_info_list
        .iter()
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

            let src = triangle_info
                .vertex_indices
                .iter()
                .find_map(|&vertex| {
                    let vertex = vertex as usize;
                    let source_position = context.get_position(face, vertex);
                    (source_position == missing_position).then_some(vertex)
                })
                .unwrap();

            (dst + offset, src + offset)
        })
        .for_each(|(dst, src)| {
            tangent_spaces[dst] = tangent_spaces[src];
        });
}
