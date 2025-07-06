/*!
 *  Copyright (C) 2011 by Morten S. Mikkelsen
 *
 *  This software is provided 'as-is', without any express or implied
 *  warranty.  In no event will the authors be held liable for any damages
 *  arising from the use of this software.
 *
 *  Permission is granted to anyone to use this software for any purpose,
 *  including commercial applications, and to alter it and redistribute it
 *  freely, subject to the following restrictions:
 *
 *  1. The origin of this software must not be misrepresented; you must not
 *     claim that you wrote the original software. If you use this software
 *     in a product, an acknowledgment in the product documentation would be
 *     appreciated but is not required.
 *  2. Altered source versions must be plainly marked as such, and must not be
 *     misrepresented as being the original software.
 *  3. This notice may not be removed or altered from any source distribution.
 */

/*!
 * Author: Morten S. Mikkelsen
 * Version: 1.0
 *
 * The files mikktspace.h and mikktspace.c are designed to be
 * stand-alone files and it is important that they are kept this way.
 * Not having dependencies on structures/classes/libraries specific
 * to the program, in which they are used, allows them to be copied
 * and used as is into any tool, program or plugin.
 * The code is designed to consistently generate the same
 * tangent spaces, for a given mesh, in any tool in which it is used.
 * This is done by performing an internal welding step and subsequently an order-independent evaluation
 * of tangent space for meshes consisting of triangles and quads.
 * This means faces can be received in any order and the same is true for
 * the order of vertices of each face. The generated result will not be affected
 * by such reordering. Additionally, whether degenerate (vertices or texture coordinates)
 * primitives are present or not will not affect the generated results either.
 * Once tangent space calculation is done the vertices of degenerate primitives will simply
 * inherit tangent space from neighboring non degenerate primitives.
 * The analysis behind this implementation can be found in my master's thesis
 * which is available for download --> http://image.diku.dk/projects/media/morten.mikkelsen.08.pdf
 * Note that though the tangent spaces at the vertices are generated in an order-independent way,
 * by this implementation, the interpolated tangent space is still affected by which diagonal is
 * chosen to split each quad. A sensible solution is to have your tools pipeline always
 * split quads by the shortest diagonal. This choice is order-independent and works with mirroring.
 * If these have the same length then compare the diagonals defined by the texture coordinates.
 * XNormal which is a tool for baking normal maps allows you to write your own tangent space plugin
 * and also quad triangulator plugin.
 */

//! To avoid visual errors (distortions/unwanted hard edges in lighting), when using sampled normal maps, the
//! normal map sampler must use the exact inverse of the pixel shader transformation.
//! The most efficient transformation we can possibly do in the pixel shader is
//! achieved by using, directly, the "unnormalized" interpolated tangent, bitangent and vertex normal: vT, vB and vN.
//! pixel shader (fast transform out)
//! vNout = normalize( vNt.x * vT + vNt.y * vB + vNt.z * vN );
//! where vNt is the tangent space normal. The normal map sampler must likewise use the
//! interpolated and "unnormalized" tangent, bitangent and vertex normal to be compliant with the pixel shader.
//! sampler does (exact inverse of pixel shader):
//! float3 row0 = cross(vB, vN);
//! float3 row1 = cross(vN, vT);
//! float3 row2 = cross(vT, vB);
//! float fSign = dot(vT, row0)<0 ? -1 : 1;
//! vNt = normalize( fSign * float3(dot(vNout,row0), dot(vNout,row1), dot(vNout,row2)) );
//! where vNout is the sampled normal in some chosen 3D space.
//!
//! Should you choose to reconstruct the bitangent in the pixel shader instead
//! of the vertex shader, as explained earlier, then be sure to do this in the normal map sampler also.
//! Finally, beware of quad triangulations. If the normal map sampler doesn't use the same triangulation of
//! quads as your renderer then problems will occur since the interpolated tangent spaces will differ
//! eventhough the vertex level tangent spaces match. This can be solved either by triangulating before
//! sampling/exporting or by using the order-independent choice of diagonal for splitting quads suggested earlier.
//! However, this must be used both by the sampler and your tools/rendering pipeline.

#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

mod math;
mod mikktspace;

use mikktspace::generate_tangent_space;

#[cfg(feature = "std")]
mod std {
    extern crate std;

    /// Implements [`Ops`](crate::Ops) using the standard library.
    /// This is the recommended default when the `std` feature is enabled, as it
    /// it designed to identically match the results provided by the original
    /// mikktspace C library.
    pub struct StdOps;

    impl crate::Ops for StdOps {
        #[inline]
        fn sqrtf(x: f32) -> f32 {
            x.sqrt()
        }

        #[inline]
        fn cos(x: f64) -> f64 {
            x.cos()
        }

        #[inline]
        fn acos(x: f64) -> f64 {
            x.acos()
        }
    }
}

pub use math::Ops;

#[cfg(feature = "std")]
pub use std::StdOps;

/// Provides an interface for reading vertex information from geometry, and writing
/// back out the calculated tangent space information.
///
/// Without the `std` feature, there is no default implementation for [`Ops`]
/// provided.
/// Instead, you must also provide a type implementing [`Ops`] using an alternative
/// math backend, such as [`libm`].
///
/// [libm]: https://docs.rs/libm
pub trait MikkTSpaceInterface<
    #[cfg(not(feature = "std"))] O: Ops,
    #[cfg(feature = "std")] O: Ops = std::StdOps,
>
{
    /// Returns the number of faces (triangles/quads) on the mesh to be processed.
    fn get_num_faces(&self) -> usize;

    /// Returns the number of vertices on face number `face`.
    /// `face` is a number in the range `0..get_num_faces()`.
    fn get_num_vertices_of_face(&self, face: usize) -> usize;

    /// Returns the position of the referenced `face` of vertex number `vert`.
    /// `vert` is in the range `0..=2` for triangles and `0..=3` for quads.
    fn get_position(&self, face: usize, vert: usize) -> [f32; 3];

    /// Returns the normal of the referenced `face` of vertex number `vert`.
    /// `vert` is in the range `0..=2` for triangles and `0..=3` for quads.
    fn get_normal(&self, face: usize, vert: usize) -> [f32; 3];

    /// Returns the texture coordinate of the referenced `face` of vertex number `vert`.
    /// `vert` is in the range `0..=2` for triangles and `0..=3` for quads.
    fn get_tex_coord(&self, face: usize, vert: usize) -> [f32; 2];

    /// This function is used to return tangent space results to the application.
    ///
    /// Note that the results are returned unindexed.
    /// It is possible to generate a new index list, but averaging/overwriting
    /// tangent spaces by using an already existing index list **WILL** produce
    /// **INCORRECT** results.
    /// **DO NOT** use an already existing index list.
    fn set_tangent_space(&mut self, tangent_space: TangentSpace, face: usize, vert: usize);
}

/// Wraps the relevant results generated when calculating the tangent space for
/// a particular vertex on a particular face.
pub struct TangentSpace {
    tangent: [f32; 3],
    bi_tangent: [f32; 3],
    mag_s: f32,
    mag_t: f32,
    is_orientation_preserving: bool,
}

impl TangentSpace {
    /// Returns the normalized tangent as an `[x, y, z]` array.
    #[inline]
    pub const fn tangent(&self) -> [f32; 3] {
        self.tangent
    }

    /// Returns the normalized bi-tangent as an `[x, y, z]` array.
    #[inline]
    pub const fn bi_tangent(&self) -> [f32; 3] {
        self.bi_tangent
    }

    /// Returns the magnitude of the tangent.
    #[inline]
    pub const fn tangent_magnitude(&self) -> f32 {
        self.mag_s
    }

    /// Returns the magnitude of the bi-tangent.
    #[inline]
    pub const fn bi_tangent_magnitude(&self) -> f32 {
        self.mag_t
    }

    /// Indicates if this generated tangent preserves the original orientation of
    /// the face.
    #[inline]
    pub const fn is_orientation_preserving(&self) -> bool {
        self.is_orientation_preserving
    }

    /// Returns an encoded summary of the tangent and bi-tangent as an `[x, y, z, w]`
    /// array.
    #[inline]
    pub const fn tangent_encoded(&self) -> [f32; 4] {
        let sign = if self.is_orientation_preserving {
            1.0
        } else {
            -1.0
        };
        [self.tangent[0], self.tangent[1], self.tangent[2], sign]
    }
}

/// Default (recommended) fAngularThreshold is 180 degrees (which means threshold disabled)
pub fn gen_tang_space_default<I, O>(interface: &mut I) -> bool
where
    I: MikkTSpaceInterface<O>,
    O: Ops,
{
    generate_tangent_space(interface, 180.0f32).is_ok()
}

pub fn gen_tang_space<I, O>(interface: &mut I, angular_threshold: f32) -> bool
where
    I: MikkTSpaceInterface<O>,
    O: Ops,
{
    generate_tangent_space(interface, angular_threshold).is_ok()
}
