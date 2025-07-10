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

    reference.assert_eq(&value);
});
