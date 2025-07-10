#![no_main]

#[path = "../src/lib.rs"]
mod geometry;

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;

use crate::geometry::{Triangle, TriangulatedGeometry, Vertex};

#[derive(Debug)]
struct OneTriangle(TriangulatedGeometry);

impl Arbitrary<'_> for OneTriangle {
    fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self, arbitrary::Error> {
        let vertices = vec![
            Vertex::arbitrary(u)?,
            Vertex::arbitrary(u)?,
            Vertex::arbitrary(u)?,
        ];
        let triangles = vec![Triangle {
            vertex_indices: [0, 1, 2],
        }];
        let mut value = TriangulatedGeometry {
            vertices,
            triangles,
        };

        value.validate()?;

        Ok(Self(value))
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        let (min, max) = Vertex::size_hint(depth);
        (3 * min, max.map(|max| 3 * max))
    }
}

fuzz_target!(|value: OneTriangle| {
    let OneTriangle(value) = value;

    let reference = {
        let mut value = value.clone();
        mikktspace_sys::gen_tang_space_default(&mut value);
        value
    };

    let value = {
        let mut value = value;
        mikktspace_rs::gen_tang_space_default(&mut value);
        value
    };

    reference.assert_eq(&value);
});
