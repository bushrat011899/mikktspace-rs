#![no_main]

#[path = "../src/lib.rs"]
mod geometry;

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;

use crate::geometry::TriangulatedGeometry;

fuzz_target!(|value: TriangulatedGeometry| {
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

    if reference != value {
        let r = reference.vertices.iter();
        let v = value.vertices.iter();

        r.zip(v).for_each(|(r, v)| assert_eq!(r, v));

        assert_eq!(
            reference, value,
            "tangents equal, but something else was changed improperly!"
        );
    }
});
