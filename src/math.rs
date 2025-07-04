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

use core::ffi::{c_double, c_float, c_int};
use core::ops::{Add, Index, Mul, Sub};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec3 {
    pub x: c_float,
    pub y: c_float,
    pub z: c_float,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    };

    pub fn dot(self, rhs: Self) -> c_float {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn normalize_or_zero(&mut self) {
        // might change this to an epsilon based test
        if not_zero(self.x) || not_zero(self.y) || not_zero(self.z) {
            *self = (1 as c_int as c_float / self.length()) * *self
        }
    }

    pub fn length_squared(self) -> c_float {
        self.dot(self)
    }

    pub fn length(self) -> c_float {
        sqrtf(self.length_squared())
    }
}

impl Index<usize> for Vec3 {
    type Output = c_float;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!(),
        }
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec3 {
            x: rhs * self.x,
            y: rhs * self.y,
            z: rhs * self.z,
        }
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}

impl PartialEq for Vec3 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

pub fn acos(x: c_double) -> c_double {
    extern crate std;
    x.acos()
}

pub fn cos(x: c_double) -> c_double {
    extern crate std;
    x.cos()
}

fn sqrtf(x: c_float) -> c_float {
    extern crate std;
    x.sqrt()
}

pub fn fabsf(x: c_float) -> c_float {
    if x.is_sign_negative() {
        -x
    } else {
        x
    }
}

pub fn not_zero(x: c_float) -> bool {
    // could possibly use FLT_EPSILON instead
    fabsf(x) > f32::MIN_POSITIVE
}

pub fn deg_to_rad(x: c_float) -> c_float {
    x * core::f32::consts::PI as c_float / 180.0f32
}
