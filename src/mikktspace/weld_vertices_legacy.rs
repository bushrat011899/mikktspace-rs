//! Provides [`weld_vertices`]; a method to deduplicate [`FaceVertex`] values in
//! a list.
//! This implementation is overly complex and has poor `NaN` handling _deliberately_
//! to match the original C implementation.

use alloc::vec::Vec;

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
    // make bbox
    let Some((min, max)) = triangle_vertices
        .iter()
        .map(|&i| get_position_from_index(context, i))
        .fold(None, |state, v| {
            let (mut min, mut max) = state.unwrap_or((v, v));

            for c in 0..3 {
                min[c] = min[c].min(v[c]);
                max[c] = max[c].max(v[c]);
            }

            Some((min, max))
        })
    else {
        // If we cannot generate a bounding box then there are no vertices to weld.
        return;
    };

    let d = max - min;

    let c_max = if d.y > d.x && d.y > d.z {
        1
    } else if d.z > d.x {
        2
    } else {
        0
    };

    let mut temporary_vertices = triangle_vertices
        .iter()
        .map(|&v| get_position_from_index(context, v))
        .enumerate()
        .map(|(index, position)| TemporaryVertex {
            group: {
                const GROUPS: u16 = 2048;
                let t = (position[c_max] - min[c_max]) / d[c_max];
                let group = (GROUPS as f32 * t.clamp(0., 1.)) as u16;
                group.clamp(0, GROUPS - 1)
            },
            position,
            original_index: index,
        })
        .collect::<Vec<_>>();

    temporary_vertices.sort_by_key(|v| v.group);

    for chunk in temporary_vertices.chunk_by_mut(|a, b| a.group == b.group) {
        merge_verts_fast(context, triangle_vertices, chunk);
    }
}

fn merge_verts_fast<I: MikkTSpaceInterface<O>, O: Ops>(
    context: &I,
    vertices: &mut [FaceVertex],
    buffer: &mut [TemporaryVertex<O>],
) {
    // If there is only a single element (or no elements), merging is complete.
    if buffer.len() < 2 {
        return;
    }

    // make bbox
    let (min, max) = buffer
        .iter()
        .map(|t| t.position)
        .fold(None, |state, v| {
            let (mut min, mut max) = state.unwrap_or((v, v));

            for c in 0..3 {
                min[c] = min[c].min(v[c]);
                max[c] = max[c].max(v[c]);
            }

            Some((min, max))
        })
        .unwrap();

    let d = max - min;

    let c = if d.y > d.x && d.y > d.z {
        1
    } else if d.z > d.x {
        2
    } else {
        0
    };

    let sep = 0.5f32 * (max[c] + min[c]);

    // stop if all vertices are NaNs
    if !sep.is_finite() {
        return;
    }

    // terminate recursion when the separation/average value
    // is no longer strictly between fMin and fMax values.
    if !(min[c] < sep && sep < max[c]) {
        // complete the weld
        for (l, v_a) in buffer.iter().enumerate() {
            let i = v_a.original_index;
            let index = vertices[i];

            let a = (
                v_a.position,
                get_normal_from_index(context, index),
                get_texture_coordinate_from_index(context, index),
            );

            let j = buffer.iter().take(l).find_map(|v_b| {
                let j = v_b.original_index;
                let index = vertices[j];

                let b = (
                    v_b.position,
                    get_normal_from_index(context, index),
                    get_texture_coordinate_from_index(context, index),
                );

                (a == b).then_some(j)
            });

            // merge if previously found
            if let Some(j) = j {
                vertices[i] = vertices[j];
            }
        }

        return;
    }

    // separate into vertices either left or right of the separation plane by
    // swapping pairs.
    let mut unsorted = 0..buffer.len();
    while unsorted.len() >= 2 {
        let l = unsorted.find(|&i| buffer[i].position[c] >= sep);
        let r = (&mut unsorted).rev().find(|&i| buffer[i].position[c] < sep);

        unsorted = match (l, r) {
            (Some(l), Some(r)) => {
                buffer.swap(l, r);
                (l + 1)..r
            }
            (None, Some(r)) => unsorted.start..(r + 1),
            (Some(l), None) => l..(l + 1),
            (None, None) => unsorted,
        };
    }

    // separation above only operates on pairs, so there may be a single unsorted
    // entry left.
    if !unsorted.is_empty() && buffer[unsorted.start].position[c] < sep {
        unsorted.start += 1;
    }

    // merge vertices in each separated buffer
    let (left, right) = buffer.split_at_mut(unsorted.start);
    for part in [left, right] {
        merge_verts_fast(context, vertices, part);
    }
}

struct TemporaryVertex<O: Ops> {
    position: Vec3<O>,
    original_index: usize,
    group: u16,
}

impl<O: Ops> Copy for TemporaryVertex<O> {}

impl<O: Ops> Clone for TemporaryVertex<O> {
    fn clone(&self) -> Self {
        *self
    }
}
