use arbitrary::{Arbitrary, Unstructured};

#[derive(Debug, Clone, Copy, Arbitrary)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
    pub tangent: [f32; 3],
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Vertex) -> bool {
        if self.position.map(|p| p.to_ne_bytes()) != other.position.map(|p| p.to_ne_bytes()) {
            return false;
        }

        if self.normal.map(|p| p.to_ne_bytes()) != other.normal.map(|p| p.to_ne_bytes()) {
            return false;
        }

        if self.tex_coord.map(|p| p.to_ne_bytes()) != other.tex_coord.map(|p| p.to_ne_bytes()) {
            return false;
        }

        self.tangent.map(|p| p.to_ne_bytes()) == other.tangent.map(|p| p.to_ne_bytes())
    }
}

#[derive(Debug, Clone, PartialEq, Arbitrary)]
pub struct Face {
    pub vertex_indices: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Arbitrary)]
pub struct Triangle {
    pub vertex_indices: [usize; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct TriangulatedGeometry {
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<Triangle>,
}

impl TriangulatedGeometry {
    pub fn validate(&mut self) -> Result<(), arbitrary::Error> {
        let Self {
            vertices,
            triangles,
        } = self;

        // Known failure: no vertices
        if vertices.is_empty() {
            return Err(arbitrary::Error::IncorrectFormat);
        }

        // Known failure: no faces
        if triangles.is_empty() {
            return Err(arbitrary::Error::IncorrectFormat);
        }

        // Known failure: face vertex indices out of range
        for triangle in triangles.iter_mut() {
            triangle.vertex_indices = triangle.vertex_indices.map(|i| i % vertices.len());
        }

        // Known failure: NaN values
        if vertices
            .iter()
            .flat_map(|vertex| {
                vertex
                    .position
                    .iter()
                    .copied()
                    .chain(vertex.normal)
                    .chain(vertex.tex_coord)
            })
            .any(|v| v.is_nan())
        {
            return Err(arbitrary::Error::IncorrectFormat);
        }

        // Known failure: Identical positions within a face
        for face in triangles.iter() {
            let mut iter = face.vertex_indices.iter();
            while let Some(a) = iter.next() {
                for b in iter.clone() {
                    if vertices[*a].position == vertices[*b].position {
                        return Err(arbitrary::Error::IncorrectFormat);
                    }
                }
            }
        }

        // Known failure: non-normal values
        if vertices
            .iter()
            .flat_map(|vertex| {
                vertex
                    .position
                    .iter()
                    .copied()
                    .chain(vertex.normal)
                    .chain(vertex.tex_coord)
            })
            .any(|v| !v.is_normal())
        {
            return Err(arbitrary::Error::IncorrectFormat);
        }

        Ok(())
    }

    /// Compares two geometries totally before panicking _if_ they differ.
    pub fn assert_eq(&self, other: &Self) {
        use core::fmt::Write;

        let mut errors = String::new();

        if self.triangles.len() != other.triangles.len() {
            let _ = writeln!(&mut errors, "- Expected {} triangles; found {}", self.triangles.len(), other.triangles.len());
        }

        for (a, b) in self.triangles.iter().zip(other.triangles.iter()).filter(|(a, b)| a != b) {
            let _ = writeln!(&mut errors, "- Expected {:?} triangle; found {:?}", a, b);
        }

        if self.vertices.len() != other.vertices.len() {
            let _ = writeln!(&mut errors, "- Expected {} vertices; found {}", self.vertices.len(), other.vertices.len());
        }

        for (a, b) in self.vertices.iter().zip(other.vertices.iter()).filter(|(a, b)| a != b) {
            let _ = writeln!(&mut errors, "  - Differing vertex:");
            if a.position != b.position {
                let _ = writeln!(&mut errors, "    - Expected {:?} position; found {:?}", a.position, b.position);
            }
            if a.normal != b.normal {
                let _ = writeln!(&mut errors, "    - Expected {:?} normal; found {:?}", a.normal, b.normal);
            }
            if a.tex_coord != b.tex_coord {
                let _ = writeln!(&mut errors, "    - Expected {:?} texture coordinate; found {:?}", a.tex_coord, b.tex_coord);
            }
            if a.tangent != b.tangent {
                let _ = writeln!(&mut errors, "    - Expected {:?} tangent; found {:?}", a.tangent, b.tangent);
            }
        }

        if !errors.is_empty() {
            panic!("Difference Summary:\n{}", errors);
        }
    }
}

impl Arbitrary<'_> for TriangulatedGeometry {
    fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self, arbitrary::Error> {
        let mut value = Self {
            vertices: Vec::<Vertex>::arbitrary(u)?,
            triangles: Vec::<Triangle>::arbitrary(u)?,
        };

        value.validate()?;

        Ok(value)
    }
}

impl mikktspace_sys::MikkTSpaceInterface for TriangulatedGeometry {
    fn get_num_faces(&self) -> usize {
        self.triangles.len()
    }

    fn get_num_vertices_of_face(&self, _face: usize) -> usize {
        3
    }

    fn get_position(&self, face: usize, vert: usize) -> [f32; 3] {
        self.vertices[self.triangles[face].vertex_indices[vert]].position
    }

    fn get_normal(&self, face: usize, vert: usize) -> [f32; 3] {
        self.vertices[self.triangles[face].vertex_indices[vert]].normal
    }

    fn get_tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        self.vertices[self.triangles[face].vertex_indices[vert]].tex_coord
    }

    fn set_tspace(
        &mut self,
        tangent: [f32; 3],
        _bi_tangent: [f32; 3],
        _mag_s: f32,
        _mag_t: f32,
        _is_orientation_preserving: bool,
        face: usize,
        vert: usize,
    ) {
        self.vertices[self.triangles[face].vertex_indices[vert]].tangent = tangent;
    }
}

impl mikktspace_rs::MikkTSpaceInterface for TriangulatedGeometry {
    fn get_num_faces(&self) -> usize {
        self.triangles.len()
    }

    fn get_num_vertices_of_face(&self, _face: usize) -> usize {
        3
    }

    fn get_position(&self, face: usize, vert: usize) -> [f32; 3] {
        self.vertices[self.triangles[face].vertex_indices[vert]].position
    }

    fn get_normal(&self, face: usize, vert: usize) -> [f32; 3] {
        self.vertices[self.triangles[face].vertex_indices[vert]].normal
    }

    fn get_tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        self.vertices[self.triangles[face].vertex_indices[vert]].tex_coord
    }

    fn set_tangent_space(
        &mut self,
        tangent_space: mikktspace_rs::TangentSpace,
        face: usize,
        vert: usize,
    ) {
        self.vertices[self.triangles[face].vertex_indices[vert]].tangent = tangent_space.tangent();
    }
}
