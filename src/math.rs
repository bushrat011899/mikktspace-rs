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

use core::{
    marker::PhantomData,
    ops::{Add, Index, Mul, Sub},
};

/// Provides the math operations required by the tangent space algorithm but which
/// aren't included in Rust's [`core`] crate.
/// With the `std` feature enabled, a (default) implementation is provided.
pub trait Ops {
    /// Provides a [`sqrt`] implementation for [`f32`].
    ///
    /// [`sqrt`]: https://doc.rust-lang.org/stable/std/primitive.f32.html#method.sqrt
    fn sqrtf(x: f32) -> f32;

    /// Provides a [`cos`] implementation for [`f64`].
    ///
    /// [`cos`]: https://doc.rust-lang.org/stable/std/primitive.f64.html#method.cos
    fn cos(x: f64) -> f64;

    /// Provides a [`acos`] implementation for [`f64`].
    ///
    /// [`acos`]: https://doc.rust-lang.org/stable/std/primitive.f64.html#method.acos
    fn acos(x: f64) -> f64;
}

pub(crate) struct Vec3<O: Ops> {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) z: f32,
    pub(crate) _phantom: PhantomData<O>,
}

impl<O: Ops> Copy for Vec3<O> {}

impl<O: Ops> Clone for Vec3<O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<O: Ops> Vec3<O> {
    pub(crate) const ZERO: Vec3<O> = Vec3 {
        x: 0.,
        y: 0.,
        z: 0.,
        _phantom: PhantomData,
    };

    pub(crate) fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub(crate) fn normalize_or_zero(&mut self) {
        // might change this to an epsilon based test
        if not_zero(self.x) || not_zero(self.y) || not_zero(self.z) {
            *self = (1f32 / self.length()) * *self
        }
    }

    pub(crate) fn normalized_or_zero(mut self) -> Self {
        self.normalize_or_zero();
        self
    }

    pub(crate) fn length_squared(self) -> f32 {
        self.dot(self)
    }

    pub(crate) fn length(self) -> f32 {
        O::sqrtf(self.length_squared())
    }
}

impl<O: Ops> Index<usize> for Vec3<O> {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!(),
        }
    }
}

impl<O: Ops> Add for Vec3<O> {
    type Output = Vec3<O>;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            _phantom: PhantomData,
        }
    }
}

impl<O: Ops> Sub for Vec3<O> {
    type Output = Vec3<O>;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            _phantom: PhantomData,
        }
    }
}

impl<O: Ops> Mul<f32> for Vec3<O> {
    type Output = Vec3<O>;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec3 {
            x: rhs * self.x,
            y: rhs * self.y,
            z: rhs * self.z,
            _phantom: PhantomData,
        }
    }
}

impl<O: Ops> Mul<Vec3<O>> for f32 {
    type Output = Vec3<O>;

    fn mul(self, rhs: Vec3<O>) -> Self::Output {
        rhs * self
    }
}

impl<O: Ops> PartialEq for Vec3<O> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

pub(crate) fn fabsf(x: f32) -> f32 {
    if x.is_sign_negative() {
        -x
    } else {
        x
    }
}

pub(crate) fn not_zero(x: f32) -> bool {
    // could possibly use FLT_EPSILON instead
    fabsf(x) > f32::MIN_POSITIVE
}

pub(crate) fn deg_to_rad(x: f32) -> f32 {
    x * core::f32::consts::PI / 180.0f32
}
