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
