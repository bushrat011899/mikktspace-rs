use super::Edge;

pub(super) fn quick_sort_edges(edges: &mut [Edge]) {
    quick_sort_by_key_with_seed(edges, |e| e.i0, INTERNAL_RND_SORT_SEED);

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

const INTERNAL_RND_SORT_SEED: u32 = 39871946;
