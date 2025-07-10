//! Provides [`weld_vertices`]; a method to deduplicate [`FaceVertex`] values in
//! a list.
//! This implementation is overly complex and has poor `NaN` handling _deliberately_
//! to match the original C implementation.

use alloc::vec;

use crate::{
    math::Vec3,
    mikktspace::{
        face_vertex::FaceVertex, get_normal_from_index, get_position_from_index,
        get_texture_coordinate_from_index,
    },
    MikkTSpaceInterface, Ops,
};

pub(super) fn weld_vertices<I: MikkTSpaceInterface<O>, O: Ops>(
    context: &I,
    triangle_vertices: &mut [FaceVertex],
) {
    // Generate bounding box
    let mut min = get_position_from_index(context, FaceVertex::new(0, 0));
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
    triangle_verticies: &mut [FaceVertex],
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
