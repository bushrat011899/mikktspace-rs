//! Implements edge sorting using `[T]::sort`.

use super::Edge;

pub(super) fn quick_sort_edges(edges: &mut [Edge]) {
    edges.sort();
}
