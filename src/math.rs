use core::ops::{Add, Mul, Sub};

use crate::libc::{c_double, c_float, c_int};

pub const M_PI: c_double = 3.141_592_653_589_793;
const __FLT_MIN__: c_float = 1.175_494_4e-38;
const FLT_MIN: c_float = __FLT_MIN__;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SVec3 {
    pub x: c_float,
    pub y: c_float,
    pub z: c_float,
}

impl SVec3 {
    pub fn dot(self, rhs: Self) -> c_float {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn normalize_or_zero(&mut self) {
        if not_zero(self.x) != 0 || not_zero(self.y) != 0 || not_zero(self.z) != 0 {
            *self = (1 as c_int as c_float / length(*self)) * *self;
        }
    }
}

impl Add for SVec3 {
    type Output = SVec3;

    fn add(self, rhs: Self) -> Self::Output {
        SVec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for SVec3 {
    type Output = SVec3;

    fn sub(self, rhs: Self) -> Self::Output {
        SVec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<f32> for SVec3 {
    type Output = SVec3;

    fn mul(self, rhs: f32) -> Self::Output {
        SVec3 {
            x: rhs * self.x,
            y: rhs * self.y,
            z: rhs * self.z,
        }
    }
}

impl Mul<SVec3> for f32 {
    type Output = SVec3;

    fn mul(self, rhs: SVec3) -> Self::Output {
        SVec3 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

impl PartialEq for SVec3 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

pub fn acos(x: c_double) -> c_double {
    unsafe {
        extern "C" {
            fn acos(_: c_double) -> c_double;
        }
        acos(x)
    }
}

pub fn cos(x: c_double) -> c_double {
    unsafe {
        extern "C" {
            fn cos(_: c_double) -> c_double;
        }
        cos(x)
    }
}

fn sqrtf(x: c_float) -> c_float {
    unsafe {
        extern "C" {
            fn sqrtf(_: c_float) -> c_float;
        }
        sqrtf(x)
    }
}

pub fn fabsf(x: c_float) -> c_float {
    unsafe {
        extern "C" {
            fn fabsf(_: c_float) -> c_float;
        }
        fabsf(x)
    }
}

pub fn length_squared(v: SVec3) -> c_float {
    v.dot(v)
}

pub fn length(v: SVec3) -> c_float {
    sqrtf(length_squared(v))
}

pub fn not_zero(x: c_float) -> c_int {
    (fabsf(x) > FLT_MIN) as c_int
}
