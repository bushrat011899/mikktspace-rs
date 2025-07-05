extern crate alloc;

use alloc::collections::BTreeMap;

use wavefront_obj::obj::{parse, ObjSet, Primitive};

#[derive(Clone, Copy, PartialEq, Debug)]
struct TangentSpace {
    tangent: [f32; 3],
    bi_tangent: [f32; 3],
    tangent_magnitude: f32,
    bi_tangent_magnitude: f32,
    is_orientation_preserving: bool,
}

struct WavefrontObject<'o> {
    object: &'o wavefront_obj::obj::Object,
    result: BTreeMap<(usize, usize), TangentSpace>,
}

impl mikktspace_rs::MikkTSpaceInterface for WavefrontObject<'_> {
    fn get_num_faces(&self) -> usize {
        self.object
            .geometry
            .iter()
            .flat_map(|g| g.shapes.iter())
            .count()
    }

    fn get_num_vertices_of_face(&self, face: usize) -> usize {
        let face = self
            .object
            .geometry
            .iter()
            .flat_map(|g| g.shapes.iter())
            .nth(face)
            .unwrap();
        match face.primitive {
            Primitive::Point(_) => 1,
            Primitive::Line(_, _) => 2,
            Primitive::Triangle(_, _, _) => 3,
        }
    }

    fn get_position(&self, face: usize, vert: usize) -> [f32; 3] {
        let shape = self
            .object
            .geometry
            .iter()
            .flat_map(|g| g.shapes.iter())
            .nth(face)
            .unwrap();

        let indices = match shape.primitive {
            Primitive::Point(a) => &[a][..],
            Primitive::Line(a, b) => &[a, b][..],
            Primitive::Triangle(a, b, c) => &[a, b, c][..],
        };

        let index = indices.iter().map(|(v, _t, _n)| *v).nth(vert).unwrap();

        let vertex = self.object.vertices[index];

        [vertex.x as f32, vertex.y as f32, vertex.z as f32].into()
    }

    fn get_normal(&self, face: usize, vert: usize) -> [f32; 3] {
        let shape = self
            .object
            .geometry
            .iter()
            .flat_map(|g| g.shapes.iter())
            .nth(face)
            .unwrap();

        let indices = match shape.primitive {
            Primitive::Point(a) => &[a][..],
            Primitive::Line(a, b) => &[a, b][..],
            Primitive::Triangle(a, b, c) => &[a, b, c][..],
        };

        let index = indices
            .iter()
            .map(|(_v, _t, n)| *n)
            .nth(vert)
            .unwrap()
            .unwrap();

        let normal = self.object.normals[index];

        [normal.x as f32, normal.y as f32, normal.z as f32].into()
    }

    fn get_tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        let shape = self
            .object
            .geometry
            .iter()
            .flat_map(|g| g.shapes.iter())
            .nth(face)
            .unwrap();

        let indices = match shape.primitive {
            Primitive::Point(a) => &[a][..],
            Primitive::Line(a, b) => &[a, b][..],
            Primitive::Triangle(a, b, c) => &[a, b, c][..],
        };

        let index = indices
            .iter()
            .map(|(_v, t, _n)| *t)
            .nth(vert)
            .unwrap()
            .unwrap();

        let tvertex = self.object.tex_vertices[index];

        [tvertex.u as f32, tvertex.v as f32]
    }

    fn set_tangent_space(
        &mut self,
        tangent_space: mikktspace_rs::TangentSpace,
        face: usize,
        vert: usize,
    ) {
        self.result.insert(
            (face, vert),
            TangentSpace {
                tangent: tangent_space.tangent(),
                bi_tangent: tangent_space.bi_tangent(),
                tangent_magnitude: tangent_space.tangent_magnitude(),
                bi_tangent_magnitude: tangent_space.bi_tangent_magnitude(),
                is_orientation_preserving: tangent_space.is_orientation_preserving(),
            },
        );
    }
}

impl mikktspace_sys::MikkTSpaceInterface for WavefrontObject<'_> {
    fn get_num_faces(&self) -> usize {
        <Self as mikktspace_rs::MikkTSpaceInterface>::get_num_faces(&self)
    }

    fn get_num_vertices_of_face(&self, face: usize) -> usize {
        <Self as mikktspace_rs::MikkTSpaceInterface>::get_num_vertices_of_face(&self, face)
    }

    fn get_position(&self, face: usize, vert: usize) -> [f32; 3] {
        <Self as mikktspace_rs::MikkTSpaceInterface>::get_position(&self, face, vert)
    }

    fn get_normal(&self, face: usize, vert: usize) -> [f32; 3] {
        <Self as mikktspace_rs::MikkTSpaceInterface>::get_normal(&self, face, vert)
    }

    fn get_tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        <Self as mikktspace_rs::MikkTSpaceInterface>::get_tex_coord(&self, face, vert)
    }

    fn set_tspace(
        &mut self,
        tangent: [f32; 3],
        bi_tangent: [f32; 3],
        mag_s: f32,
        mag_t: f32,
        is_orientation_preserving: bool,
        face: usize,
        vert: usize,
    ) {
        self.result.insert(
            (face, vert),
            TangentSpace {
                tangent,
                bi_tangent,
                tangent_magnitude: mag_s,
                bi_tangent_magnitude: mag_t,
                is_orientation_preserving: is_orientation_preserving,
            },
        );
    }
}

macro_rules! generate_tests {
    () => {};
    ($name:ident: $file:expr,$($rest:tt)*) => {
        #[test]
        fn $name() {
            let file = include_str!($file);
            let ObjSet { objects, .. } = parse(file).expect("must be able to parse sample data");
            for object in &objects {
                let mut reference_object = WavefrontObject { object, result: BTreeMap::new() };
                let reference_succeeded = mikktspace_sys::gen_tang_space_default(&mut reference_object);

                let mut object = WavefrontObject { object, result: BTreeMap::new() };
                let succeeded = mikktspace_rs::gen_tang_space_default(&mut object);

                assert_eq!(reference_succeeded, succeeded);
                assert_eq!(reference_object.result, object.result);
            }
        }

        generate_tests! {
            $($rest)*
        }
    };
}

generate_tests! {
    crangeract: "../data/crangeract.obj",
    doomcone_smooth: "../data/doomcone_smooth.obj",
    doomcone: "../data/doomcone.obj",
    obliterated: "../data/obliterated.obj",
    rancid_geometry: "../data/rancid_geometry.obj",
    tangeract: "../data/tangeract.obj",
}
